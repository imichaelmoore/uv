use std::fmt::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use tracing::debug;
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipWriter};

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
    ConfigSettings, DependencyMetadata, Dist, ExtraBuildVariables, Index, IndexLocations,
    NameRequirementSpecification, Origin, PackageConfigSettings, Resolution, ResolvedDist,
    UnresolvedRequirementSpecification,
};
use uv_fs::Simplified;
use uv_install_wheel::LinkMode;
use uv_preview::{Preview, PreviewFeatures};
use uv_pypi_types::Conflicts;
use uv_python::{
    EnvironmentPreference, PythonEnvironment, PythonInstallation, PythonPreference, PythonRequest,
    PythonVersion,
};
use uv_requirements::{RequirementsSource, RequirementsSpecification};
use uv_resolver::{
    DependencyMode, ExcludeNewer, FlatIndex, OptionsBuilder, PrereleaseMode, PythonRequirement,
    ResolutionMode, ResolverEnvironment,
};
use uv_settings::PythonInstallMirrors;
use uv_torch::{TorchMode, TorchSource, TorchStrategy};
use uv_types::{EmptyInstalledPackages, HashStrategy};
use uv_warnings::warn_user_once;
use uv_workspace::WorkspaceCache;
use uv_workspace::pyproject::ExtraBuildDependencies;

use crate::commands::pip::loggers::DefaultResolveLogger;
use crate::commands::pip::operations::report_interpreter;
use crate::commands::pip::{operations, resolution_markers, resolution_tags};
use crate::commands::reporters::PythonDownloadReporter;
use crate::commands::{ExitStatus, diagnostics};
use crate::printer::Printer;

/// Download packages to a directory.
#[allow(clippy::fn_params_excessive_bools)]
pub(crate) async fn pip_download(
    requirements: &[RequirementsSource],
    constraints: &[RequirementsSource],
    overrides: &[RequirementsSource],
    build_constraints: &[RequirementsSource],
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
    hash_checking: Option<HashCheckingMode>,
    config_settings: &ConfigSettings,
    config_settings_package: &PackageConfigSettings,
    build_isolation: BuildIsolation,
    extra_build_dependencies: &ExtraBuildDependencies,
    extra_build_variables: &ExtraBuildVariables,
    build_options: BuildOptions,
    python_version: Option<PythonVersion>,
    python_platform: Option<TargetTriple>,
    python_downloads: uv_python::PythonDownloads,
    install_mirrors: PythonInstallMirrors,
    exclude_newer: ExcludeNewer,
    sources: SourceStrategy,
    python: Option<String>,
    system: bool,
    dest: Option<PathBuf>,
    python_preference: PythonPreference,
    concurrency: Concurrency,
    cache: Cache,
    printer: Printer,
    preview: Preview,
) -> Result<ExitStatus> {
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
        groups,
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
        &[], // no excludes for download
        extras,
        None,
        &client_builder,
    )
    .await?;

    let constraints: Vec<NameRequirementSpecification> = constraints.iter().cloned().collect();

    let overrides: Vec<UnresolvedRequirementSpecification> = overrides.iter().cloned().collect();

    // Read build constraints.
    let build_constraints: Vec<NameRequirementSpecification> =
        operations::read_constraints(build_constraints, &client_builder)
            .await?
            .into_iter()
            .collect();

    // Detect the current Python interpreter.
    let python_request = python.as_deref().map(PythonRequest::parse);
    let reporter = PythonDownloadReporter::single(printer);

    let installation = PythonInstallation::find_or_download(
        python_request.as_ref(),
        EnvironmentPreference::from_system_flag(system, true),
        python_preference.with_system_flag(system),
        python_downloads,
        &client_builder,
        &cache,
        Some(&reporter),
        install_mirrors.python_install_mirror.as_deref(),
        install_mirrors.pypy_install_mirror.as_deref(),
        install_mirrors.python_downloads_json_url.as_deref(),
        preview,
    )
    .await?;

    report_interpreter(&installation, true, printer)?;

    let environment = PythonEnvironment::from_installation(installation);

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
        LinkMode::default(),
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
        vec![],
        source_trees,
        project,
        std::collections::BTreeSet::default(),
        extras,
        &groups,
        preferences,
        EmptyInstalledPackages,
        &hasher,
        &uv_configuration::Reinstall::None,
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

    // Notify the user of any resolution diagnostics.
    operations::diagnose_resolution(resolution.diagnostics(), printer)?;

    // Determine the destination directory.
    let dest = dest.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Create the destination directory if it doesn't exist.
    fs_err::create_dir_all(&dest)?;

    // Extract the distributions to download.
    let distributions: Vec<Arc<Dist>> = resolution
        .distributions()
        .filter_map(|resolved| match resolved {
            ResolvedDist::Installable { dist, .. } => Some(dist.clone()),
            ResolvedDist::Installed { .. } => None,
        })
        .collect();

    if distributions.is_empty() {
        writeln!(printer.stderr(), "No packages to download")?;
        return Ok(ExitStatus::Success);
    }

    // Create the distribution database.
    let database =
        uv_distribution::DistributionDatabase::new(&client, &build_dispatch, concurrency.downloads);

    // Download the distributions.
    let preparer = uv_installer::Preparer::new(
        &cache,
        &tags,
        &hasher,
        &build_options,
        database,
    );

    // Prepare the distributions (download and build wheels).
    let mut prepared = preparer
        .prepare(distributions, state.in_flight(), &resolution)
        .await
        .with_context(|| "Failed to prepare distributions")?;

    // Sort by filename for consistent output order.
    prepared.sort_by(|a, b| a.filename().cmp(b.filename()));

    // Copy the prepared distributions to the destination directory.
    let mut downloaded_count = 0;
    for wheel in &prepared {
        let filename = wheel.filename().to_string();
        let source_path = wheel.path();
        let dest_path = dest.join(&filename);

        if dest_path.exists() {
            debug!("Skipping existing file: {}", dest_path.user_display());
            writeln!(
                printer.stderr(),
                " {} {} (already exists)",
                "Skipping".yellow(),
                filename
            )?;
        } else {
            // The source_path is a directory (unzipped wheel), so we need to zip it
            zip_directory(source_path, &dest_path)?;
            debug!("Downloaded: {}", dest_path.user_display());
            writeln!(printer.stderr(), " {} {}", "Downloaded".green(), filename)?;
            downloaded_count += 1;
        }
    }

    // Print summary.
    let elapsed = start.elapsed();
    let s = if downloaded_count == 1 { "" } else { "s" };
    writeln!(
        printer.stderr(),
        "{}",
        format!(
            "Downloaded {} package{s} in {:.2}s",
            downloaded_count,
            elapsed.as_secs_f32()
        )
        .dimmed()
    )?;

    Ok(ExitStatus::Success)
}

/// Zip a directory into a wheel file.
fn zip_directory(source_dir: &Path, dest_file: &Path) -> Result<()> {
    let file = fs_err::File::create(dest_file)?;
    let mut zip = ZipWriter::new(file);

    let options =
        zip::write::SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(source_dir)
            .with_context(|| format!("Failed to strip prefix from {}", path.display()))?;

        // Skip the root directory itself
        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let path_str = relative_path
            .to_str()
            .with_context(|| format!("Non-UTF8 path: {}", relative_path.display()))?;

        if path.is_dir() {
            // Add directory entry (with trailing slash)
            let dir_name = format!("{path_str}/");
            zip.add_directory(&dir_name, options)?;
        } else {
            // Add file entry
            zip.start_file(path_str, options)?;
            let mut f = fs_err::File::open(path)?;
            std::io::copy(&mut f, &mut zip)?;
        }
    }

    zip.finish()?;
    Ok(())
}
