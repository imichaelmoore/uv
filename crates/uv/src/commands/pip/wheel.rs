use std::collections::BTreeSet;
use std::fmt::Write;
use std::io::Write as IoWrite;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use owo_colors::OwoColorize;
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use uv_cache::Cache;
use uv_client::{BaseClientBuilder, FlatIndexClient, RegistryClientBuilder};
use uv_configuration::{
    BuildIsolation, BuildOptions, Concurrency, Constraints, ExtrasSpecification, HashCheckingMode,
    IndexStrategy, SourceStrategy, Upgrade,
};
use uv_configuration::{KeyringProviderType, TargetTriple};
use uv_dispatch::{BuildDispatch, SharedState};
use uv_distribution::LoweredExtraBuildDependencies;
use uv_distribution_types::{
    CachedDist, ConfigSettings, DependencyMetadata, Dist, ExtraBuildVariables, Index,
    IndexLocations, NameRequirementSpecification, Origin, PackageConfigSettings, Requirement,
    ResolvedDist, Resolution, UnresolvedRequirementSpecification,
};
use uv_fs::Simplified;
use uv_install_wheel::LinkMode;
use uv_installer::Preparer;
use uv_preview::{Preview, PreviewFeatures};
use uv_pypi_types::Conflicts;
use uv_python::{
    EnvironmentPreference, PythonInstallation, PythonPreference, PythonRequest, PythonVersion,
};
use uv_requirements::{RequirementsSource, RequirementsSpecification};
use uv_resolver::{
    DependencyMode, ExcludeNewer, FlatIndex, OptionsBuilder, PrereleaseMode, PythonRequirement,
    ResolutionMode, ResolverEnvironment,
};
use uv_settings::PythonInstallMirrors;
use uv_torch::{TorchMode, TorchSource, TorchStrategy};
use uv_types::{HashStrategy, InFlight};
use uv_warnings::warn_user_once;
use uv_workspace::pyproject::ExtraBuildDependencies;
use uv_workspace::WorkspaceCache;

use crate::commands::diagnostics;
use crate::commands::pip::loggers::DefaultResolveLogger;
use crate::commands::pip::{operations, resolution_markers, resolution_tags};
use crate::commands::reporters::{PrepareReporter, PythonDownloadReporter};
use crate::commands::{elapsed, ExitStatus};
use crate::printer::Printer;

/// Build wheels from requirements.
#[allow(clippy::fn_params_excessive_bools)]
pub(crate) async fn pip_wheel(
    requirements: &[RequirementsSource],
    constraints: &[RequirementsSource],
    overrides: &[RequirementsSource],
    build_constraints: &[RequirementsSource],
    constraints_from_workspace: Vec<Requirement>,
    overrides_from_workspace: Vec<Requirement>,
    build_constraints_from_workspace: Vec<Requirement>,
    extras: &ExtrasSpecification,
    resolution_mode: ResolutionMode,
    prerelease_mode: PrereleaseMode,
    dependency_mode: DependencyMode,
    upgrade: Upgrade,
    index_locations: IndexLocations,
    index_strategy: IndexStrategy,
    torch_backend: Option<TorchMode>,
    dependency_metadata: DependencyMetadata,
    keyring_provider: KeyringProviderType,
    client_builder: &BaseClientBuilder<'_>,
    link_mode: LinkMode,
    hash_checking: Option<HashCheckingMode>,
    config_settings: &ConfigSettings,
    config_settings_package: &PackageConfigSettings,
    build_isolation: BuildIsolation,
    extra_build_dependencies: &ExtraBuildDependencies,
    extra_build_variables: &ExtraBuildVariables,
    build_options: BuildOptions,
    python_version: Option<PythonVersion>,
    python_platform: Option<TargetTriple>,
    install_mirrors: PythonInstallMirrors,
    strict: bool,
    exclude_newer: ExcludeNewer,
    sources: SourceStrategy,
    python: Option<String>,
    python_preference: PythonPreference,
    concurrency: Concurrency,
    wheel_dir: &Path,
    cache: Cache,
    printer: Printer,
    preview: Preview,
) -> anyhow::Result<ExitStatus> {
    let start = std::time::Instant::now();

    if !preview.is_enabled(PreviewFeatures::EXTRA_BUILD_DEPENDENCIES)
        && !extra_build_dependencies.is_empty()
    {
        warn_user_once!(
            "The `extra-build-dependencies` option is experimental and may change without warning. Pass `--preview-features {}` to disable this warning.",
            PreviewFeatures::EXTRA_BUILD_DEPENDENCIES
        );
    }

    let client_builder = client_builder.clone().keyring(keyring_provider);

    // Read all requirements from the provided sources.
    let RequirementsSpecification {
        project,
        requirements,
        constraints,
        overrides,
        excludes: _,
        pylock: _,
        source_trees,
        groups: _,
        index_url,
        extra_index_urls,
        no_index,
        find_links,
        no_binary,
        no_build,
        extras: _,
    } = operations::read_requirements(
        requirements,
        constraints,
        overrides,
        &[],
        extras,
        None,
        &client_builder,
    )
    .await?;

    let constraints: Vec<NameRequirementSpecification> = constraints
        .iter()
        .cloned()
        .chain(
            constraints_from_workspace
                .into_iter()
                .map(NameRequirementSpecification::from),
        )
        .collect();

    let overrides: Vec<UnresolvedRequirementSpecification> = overrides
        .iter()
        .cloned()
        .chain(
            overrides_from_workspace
                .into_iter()
                .map(UnresolvedRequirementSpecification::from),
        )
        .collect();

    // Read build constraints.
    let build_constraints: Vec<NameRequirementSpecification> =
        operations::read_constraints(build_constraints, &client_builder)
            .await?
            .into_iter()
            .chain(
                build_constraints_from_workspace
                    .iter()
                    .cloned()
                    .map(NameRequirementSpecification::from),
            )
            .collect();

    // Create the wheel directory if it doesn't exist.
    fs_err::tokio::create_dir_all(wheel_dir).await?;

    // Detect the current Python interpreter.
    let python_request = python.as_deref().map(PythonRequest::parse);
    let reporter = PythonDownloadReporter::single(printer);

    let installation = PythonInstallation::find_or_download(
        python_request.as_ref(),
        EnvironmentPreference::OnlySystem,
        python_preference,
        uv_python::PythonDownloads::Automatic,
        &client_builder,
        &cache,
        Some(&reporter),
        install_mirrors.python_install_mirror.as_deref(),
        install_mirrors.pypy_install_mirror.as_deref(),
        install_mirrors.python_downloads_json_url.as_deref(),
        preview,
    )
    .await?;

    // Create a virtual environment for building wheels.
    let temp_dir = tempfile::tempdir_in(cache.root())?;
    let environment = uv_virtualenv::create_venv(
        temp_dir.path(),
        installation.into_interpreter(),
        uv_virtualenv::Prompt::None,
        false,                              // system_site_packages
        uv_virtualenv::OnExisting::Allow,   // on_existing
        false,                              // relocatable
        false,                              // seed
        false,                              // upgradeable
        preview,
    )?;

    // Lower the extra build dependencies, if any.
    let extra_build_requires =
        LoweredExtraBuildDependencies::from_non_lowered(extra_build_dependencies.clone())
            .into_inner();

    // Determine the markers and tags to use for the resolution.
    let interpreter = environment.interpreter();
    let marker_env = resolution_markers(
        python_version.as_ref(),
        python_platform.as_ref(),
        interpreter,
    );
    let tags = resolution_tags(
        python_version.as_ref(),
        python_platform.as_ref(),
        interpreter,
    )?;

    // Determine the Python requirement, if the user requested a specific version.
    let python_requirement = if let Some(python_version) = python_version.as_ref() {
        PythonRequirement::from_python_version(interpreter, python_version)
    } else {
        PythonRequirement::from_interpreter(interpreter)
    };

    // Collect the set of required hashes.
    let hasher = if let Some(hash_checking) = hash_checking {
        HashStrategy::from_requirements(
            requirements
                .iter()
                .chain(overrides.iter())
                .map(|entry| (&entry.requirement, entry.hashes.as_slice())),
            constraints
                .iter()
                .map(|entry| (&entry.requirement, entry.hashes.as_slice())),
            Some(&marker_env),
            hash_checking,
        )?
    } else {
        HashStrategy::None
    };

    // Incorporate any index locations from the provided sources.
    let index_locations = index_locations.combine(
        extra_index_urls
            .into_iter()
            .map(Index::from_extra_index_url)
            .chain(index_url.map(Index::from_index_url))
            .map(|index| index.with_origin(Origin::RequirementsTxt))
            .collect(),
        find_links
            .into_iter()
            .map(Index::from_find_links)
            .map(|index| index.with_origin(Origin::RequirementsTxt))
            .collect(),
        no_index,
    );

    // Determine the PyTorch backend.
    let torch_backend = torch_backend
        .map(|mode| {
            let source = if uv_auth::PyxTokenStore::from_settings()
                .is_ok_and(|store| store.has_credentials())
            {
                TorchSource::Pyx
            } else {
                TorchSource::default()
            };
            TorchStrategy::from_mode(
                mode,
                source,
                python_platform
                    .map(TargetTriple::platform)
                    .as_ref()
                    .unwrap_or(interpreter.platform())
                    .os(),
            )
        })
        .transpose()?;

    // Initialize the registry client.
    let client = RegistryClientBuilder::new(client_builder.clone(), cache.clone())
        .index_locations(index_locations.clone())
        .index_strategy(index_strategy)
        .torch_backend(torch_backend.clone())
        .markers(interpreter.markers())
        .platform(interpreter.platform())
        .build();

    // Combine the `--no-binary` and `--no-build` flags from the requirements files.
    let build_options = build_options.combine(no_binary, no_build);

    // Resolve the flat indexes from `--find-links`.
    let flat_index = {
        let client = FlatIndexClient::new(client.cached_client(), client.connectivity(), &cache);
        let entries = client
            .fetch_all(index_locations.flat_indexes().map(Index::url))
            .await?;
        FlatIndex::from_entries(entries, Some(&tags), &hasher, &build_options)
    };

    // Determine whether to enable build isolation.
    let types_build_isolation = match build_isolation {
        BuildIsolation::Isolate => uv_types::BuildIsolation::Isolated,
        BuildIsolation::Shared => uv_types::BuildIsolation::Shared(&environment),
        BuildIsolation::SharedPackage(ref packages) => {
            uv_types::BuildIsolation::SharedPackage(&environment, packages)
        }
    };

    // Enforce (but never require) the build constraints, if `--require-hashes` or `--verify-hashes`
    // is provided. _Requiring_ hashes would be too strict, and would break with pip.
    let build_hasher = if hash_checking.is_some() {
        HashStrategy::from_requirements(
            std::iter::empty(),
            build_constraints
                .iter()
                .map(|entry| (&entry.requirement, entry.hashes.as_slice())),
            Some(&marker_env),
            HashCheckingMode::Verify,
        )?
    } else {
        HashStrategy::None
    };
    let build_constraints = Constraints::from_requirements(
        build_constraints
            .iter()
            .map(|constraint| constraint.requirement.clone()),
    );

    // Initialize any shared state.
    let state = SharedState::default();

    // Create a build dispatch.
    let build_dispatch = BuildDispatch::new(
        &client,
        &cache,
        &build_constraints,
        interpreter,
        &index_locations,
        &flat_index,
        &dependency_metadata,
        state.clone(),
        index_strategy,
        config_settings,
        config_settings_package,
        types_build_isolation,
        &extra_build_requires,
        extra_build_variables,
        link_mode,
        &build_options,
        &build_hasher,
        exclude_newer.clone(),
        sources,
        WorkspaceCache::default(),
        concurrency,
        preview,
    );

    // When resolving, don't take any external preferences into account.
    let preferences = Vec::default();

    let options = OptionsBuilder::new()
        .resolution_mode(resolution_mode)
        .prerelease_mode(prerelease_mode)
        .dependency_mode(dependency_mode)
        .exclude_newer(exclude_newer.clone())
        .index_strategy(index_strategy)
        .torch_backend(torch_backend)
        .build_options(build_options.clone())
        .build();

    // Resolve the requirements.
    let resolution = match operations::resolve(
        requirements,
        constraints,
        overrides,
        Vec::new(), // excludes
        source_trees,
        project,
        BTreeSet::default(),
        extras,
        &Default::default(),
        preferences,
        uv_types::EmptyInstalledPackages,
        &hasher,
        &Default::default(), // reinstall
        &upgrade,
        Some(&tags),
        ResolverEnvironment::specific(marker_env.clone()),
        python_requirement,
        interpreter.markers(),
        Conflicts::empty(),
        &client,
        &flat_index,
        state.index(),
        &build_dispatch,
        concurrency,
        options,
        Box::new(DefaultResolveLogger),
        printer,
    )
    .await
    {
        Ok(graph) => Resolution::from(graph),
        Err(err) => {
            return diagnostics::OperationDiagnostic::native_tls(client_builder.is_native_tls())
                .report(err)
                .map_or(Ok(ExitStatus::Failure), |err| Err(err.into()));
        }
    };

    // Constrain any build requirements marked as `match-runtime = true`.
    let extra_build_requires = extra_build_requires.match_runtime(&resolution)?;

    // Create a build dispatch.
    let build_dispatch = BuildDispatch::new(
        &client,
        &cache,
        &build_constraints,
        interpreter,
        &index_locations,
        &flat_index,
        &dependency_metadata,
        state.clone(),
        index_strategy,
        config_settings,
        config_settings_package,
        types_build_isolation,
        &extra_build_requires,
        extra_build_variables,
        link_mode,
        &build_options,
        &build_hasher,
        exclude_newer.clone(),
        sources,
        WorkspaceCache::default(),
        concurrency,
        preview,
    );

    // Collect distributions that need to be built/downloaded.
    let distributions: Vec<Arc<Dist>> = resolution
        .distributions()
        .filter_map(|dist| match dist {
            ResolvedDist::Installable { dist, .. } => Some(dist.clone()),
            ResolvedDist::Installed { .. } => None,
        })
        .collect();

    // Prepare the wheels.
    let in_flight = InFlight::default();
    let preparer = Preparer::new(
        &cache,
        &tags,
        &hasher,
        &build_options,
        uv_distribution::DistributionDatabase::new(&client, &build_dispatch, concurrency.downloads),
    )
    .with_reporter(Arc::new(
        PrepareReporter::from(printer).with_length(distributions.len() as u64),
    ));

    let wheels: Vec<CachedDist> = preparer
        .prepare(distributions, &in_flight, &resolution)
        .await?;

    // Pack the wheels and copy them to the output directory.
    let mut built_wheels = Vec::new();
    for wheel in wheels {
        let filename = wheel.filename().to_string();
        let archive_path = wheel.path();
        let dest_path = wheel_dir.join(&filename);

        // Pack the extracted archive directory into a wheel file.
        pack_wheel(archive_path, &dest_path).with_context(|| {
            format!(
                "Failed to pack wheel from {} to {}",
                archive_path.user_display(),
                dest_path.user_display()
            )
        })?;

        built_wheels.push(filename);
    }

    // Sort the wheels for consistent output.
    built_wheels.sort();

    // Print the summary.
    let s = if built_wheels.len() == 1 { "" } else { "s" };
    writeln!(
        printer.stderr(),
        "{}",
        format!(
            "Successfully built {} wheel{s} {}",
            format!("{}", built_wheels.len()).bold(),
            format!("in {}", elapsed(start.elapsed())).dimmed()
        )
        .dimmed()
    )?;

    // Print the wheel filenames.
    for wheel in &built_wheels {
        writeln!(printer.stderr(), " - {wheel}")?;
    }

    // Notify the user of any resolution diagnostics.
    operations::diagnose_resolution(resolution.diagnostics(), printer)?;

    // Notify the user of any environment diagnostics.
    if strict {
        operations::diagnose_environment(&resolution, &environment, &marker_env, &tags, printer)?;
    }

    Ok(ExitStatus::Success)
}

/// Pack a directory into a wheel file.
///
/// This creates a ZIP archive from the extracted wheel directory.
fn pack_wheel(source_dir: &Path, dest_path: &Path) -> anyhow::Result<()> {
    let file = fs_err::File::create(dest_path)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for entry in WalkDir::new(source_dir) {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(source_dir)
            .context("Failed to strip prefix from path")?;

        // Skip the root directory itself.
        if name.as_os_str().is_empty() {
            continue;
        }

        let name_str = name.to_string_lossy();

        if path.is_file() {
            zip.start_file(&*name_str, options)?;
            let contents = fs_err::read(path)?;
            zip.write_all(&contents)?;
        } else if path.is_dir() && !name.as_os_str().is_empty() {
            zip.add_directory(&*name_str, options)?;
        }
    }

    zip.finish()?;
    Ok(())
}
