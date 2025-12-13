use anyhow::Result;
use assert_fs::prelude::*;
use predicates::path::exists;

use crate::common::{TestContext, uv_snapshot};

#[test]
fn missing_requirements_txt() {
    let context = TestContext::new("3.12");

    uv_snapshot!(context.filters(), context.pip_wheel()
        .arg("-r")
        .arg("requirements.txt"), @r#"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: File not found: `requirements.txt`
    "#
    );
}

#[test]
fn empty_requirements_txt() -> Result<()> {
    let context = TestContext::new("3.12");
    let requirements_txt = context.temp_dir.child("requirements.txt");
    requirements_txt.touch()?;

    uv_snapshot!(context.pip_wheel()
        .arg("-r")
        .arg("requirements.txt"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    warning: Requirements file `requirements.txt` does not contain any dependencies
    Resolved in [TIME]
    Successfully built 0 wheels in [TIME]
    "#
    );

    Ok(())
}

#[test]
fn wheel_single_package() -> Result<()> {
    let context = TestContext::new("3.12");
    let wheelhouse = context.temp_dir.child("wheelhouse");

    uv_snapshot!(context.filters(), context.pip_wheel()
        .arg("iniconfig==2.0.0")
        .arg("-w")
        .arg(wheelhouse.path()), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Resolved 1 package in [TIME]
    Successfully built 1 wheel in [TIME]
     - iniconfig-2.0.0-py3-none-any.whl
    "#
    );

    // Verify the wheel was created.
    wheelhouse.child("iniconfig-2.0.0-py3-none-any.whl").assert(exists());

    Ok(())
}

#[test]
fn wheel_from_requirements_file() -> Result<()> {
    let context = TestContext::new("3.12");
    let requirements_txt = context.temp_dir.child("requirements.txt");
    requirements_txt.write_str("iniconfig==2.0.0")?;
    let wheelhouse = context.temp_dir.child("wheelhouse");

    uv_snapshot!(context.filters(), context.pip_wheel()
        .arg("-r")
        .arg("requirements.txt")
        .arg("-w")
        .arg(wheelhouse.path()), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Resolved 1 package in [TIME]
    Successfully built 1 wheel in [TIME]
     - iniconfig-2.0.0-py3-none-any.whl
    "#
    );

    // Verify the wheel was created.
    wheelhouse.child("iniconfig-2.0.0-py3-none-any.whl").assert(exists());

    Ok(())
}

#[test]
fn wheel_multiple_packages() -> Result<()> {
    let context = TestContext::new("3.12");
    let wheelhouse = context.temp_dir.child("wheelhouse");

    uv_snapshot!(context.filters(), context.pip_wheel()
        .arg("iniconfig==2.0.0")
        .arg("tomli==2.0.1")
        .arg("-w")
        .arg(wheelhouse.path()), @r#"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Resolved 2 packages in [TIME]
    Successfully built 2 wheels in [TIME]
     - iniconfig-2.0.0-py3-none-any.whl
     - tomli-2.0.1-py3-none-any.whl
    "#
    );

    // Verify the wheels were created.
    wheelhouse.child("iniconfig-2.0.0-py3-none-any.whl").assert(exists());
    wheelhouse.child("tomli-2.0.1-py3-none-any.whl").assert(exists());

    Ok(())
}

#[test]
fn wheel_help() {
    let context = TestContext::new("3.12");

    uv_snapshot!(context.filters(), context.pip_wheel().arg("--help"), @r#"
    success: true
    exit_code: 0
    ----- stdout -----
    Build wheels from requirements

    Usage: uv pip wheel [OPTIONS] <PACKAGE|--requirements <REQUIREMENTS>|--editable <EDITABLE>>

    Arguments:
      [PACKAGE]...  Build wheels for all listed packages

    Options:
      -r, --requirements <REQUIREMENTS>
              Build wheels for the packages listed in the given files
      -e, --editable <EDITABLE>
              Build wheels from the editable package based on the provided local file path
      -c, --constraints <CONSTRAINTS>
              Constrain versions using the given requirements files [env: UV_CONSTRAINT=]
          --overrides <OVERRIDES>
              Override versions using the given requirements files [env: UV_OVERRIDE=]
      -b, --build-constraints <BUILD_CONSTRAINTS>
              Constrain build dependencies using the given requirements files when building source
              distributions [env: UV_BUILD_CONSTRAINT=]
          --extra <EXTRA>
              Include optional dependencies from the specified extra name; may be provided more than
              once
          --all-extras
              Include all optional dependencies
          --no-deps
              Ignore package dependencies, instead only building wheels for the packages explicitly
              listed on the command line or in the requirements files
          --require-hashes
              Require a matching hash for each requirement [env: UV_REQUIRE_HASHES=]
          --no-verify-hashes
              Disable validation of hashes in the requirements file [env: UV_NO_VERIFY_HASHES=]
          --no-build
              Don't build source distributions
          --no-binary <NO_BINARY>
              Don't install pre-built wheels
          --only-binary <ONLY_BINARY>
              Only use pre-built wheels; don't build source distributions
          --python-version <PYTHON_VERSION>
              The minimum Python version that should be supported by the requirements (e.g., `3.7` or
              `3.7.9`)
          --python-platform <PYTHON_PLATFORM>
              The platform for which wheels should be built [possible values: windows, linux, macos,
              x86_64-pc-windows-msvc, aarch64-pc-windows-msvc, i686-pc-windows-msvc,
              x86_64-unknown-linux-gnu, aarch64-apple-darwin, x86_64-apple-darwin,
              aarch64-unknown-linux-gnu, aarch64-unknown-linux-musl, x86_64-unknown-linux-musl,
              riscv64-unknown-linux, x86_64-manylinux2014, x86_64-manylinux_2_17, x86_64-manylinux_2_28,
              x86_64-manylinux_2_31, x86_64-manylinux_2_32, x86_64-manylinux_2_33,
              x86_64-manylinux_2_34, x86_64-manylinux_2_35, x86_64-manylinux_2_36,
              x86_64-manylinux_2_37, x86_64-manylinux_2_38, x86_64-manylinux_2_39,
              x86_64-manylinux_2_40, aarch64-manylinux2014, aarch64-manylinux_2_17,
              aarch64-manylinux_2_28, aarch64-manylinux_2_31, aarch64-manylinux_2_32,
              aarch64-manylinux_2_33, aarch64-manylinux_2_34, aarch64-manylinux_2_35,
              aarch64-manylinux_2_36, aarch64-manylinux_2_37, aarch64-manylinux_2_38,
              aarch64-manylinux_2_39, aarch64-manylinux_2_40, aarch64-linux-android,
              x86_64-linux-android, wasm32-pyodide2024, arm64-apple-ios, arm64-apple-ios-simulator,
              x86_64-apple-ios-simulator]
          --strict
              Validate the Python environment after completing the build, to detect packages with
              missing dependencies or other issues
      -w, --wheel-dir <WHEEL_DIR>
              The directory into which built wheels should be placed [default: wheelhouse]
          --torch-backend <TORCH_BACKEND>
              The backend to use when fetching packages in the PyTorch ecosystem (e.g., `cpu`, `cu126`,
              or `auto`) [env: UV_TORCH_BACKEND=] [possible values: auto, cpu, cu130, cu129, cu128,
              cu126, cu125, cu124, cu123, cu122, cu121, cu120, cu118, cu117, cu116, cu115, cu114, cu113,
              cu112, cu111, cu110, cu102, cu101, cu100, cu92, cu91, cu90, cu80, rocm6.4, rocm6.3,
              rocm6.2.4, rocm6.2, rocm6.1, rocm6.0, rocm5.7, rocm5.6, rocm5.5, rocm5.4.2, rocm5.4,
              rocm5.3, rocm5.2, rocm5.1.1, rocm4.2, rocm4.1, rocm4.0.1, xpu]

    Index options:
          --index <INDEX>
              The URLs to use when resolving dependencies, in addition to the default index [env:
              UV_INDEX=]
          --default-index <DEFAULT_INDEX>
              The URL of the default package index (by default: <https://pypi.org/simple>) [env:
              UV_DEFAULT_INDEX=]
      -i, --index-url <INDEX_URL>
              (Deprecated: use `--default-index` instead) The URL of the Python package index (by
              default: <https://pypi.org/simple>) [env: UV_INDEX_URL=]
          --extra-index-url <EXTRA_INDEX_URL>
              (Deprecated: use `--index` instead) Extra URLs of package indexes to use, in addition to
              `--index-url` [env: UV_EXTRA_INDEX_URL=]
      -f, --find-links <FIND_LINKS>
              Locations to search for candidate distributions, in addition to those found in the
              registry indexes [env: UV_FIND_LINKS=]
          --no-index
              Ignore the registry index (e.g., PyPI), instead relying on direct URL dependencies and
              those provided via `--find-links`
          --index-strategy <INDEX_STRATEGY>
              The strategy to use when resolving against multiple index URLs [env: UV_INDEX_STRATEGY=]
              [possible values: first-index, unsafe-first-match, unsafe-best-match]
          --keyring-provider <KEYRING_PROVIDER>
              Attempt to use `keyring` for authentication for index URLs [env: UV_KEYRING_PROVIDER=]
              [possible values: disabled, subprocess]

    Resolver options:
      -U, --upgrade
              Allow package upgrades, ignoring pinned versions in any existing output file. Implies
              `--refresh`
      -P, --upgrade-package <UPGRADE_PACKAGE>
              Allow upgrades for a specific package, ignoring pinned versions in any existing output
              file. Implies `--refresh-package`
          --resolution <RESOLUTION>
              The strategy to use when selecting between the different compatible versions for a given
              package requirement [env: UV_RESOLUTION=] [possible values: highest, lowest,
              lowest-direct]
          --prerelease <PRERELEASE>
              The strategy to use when considering pre-release versions [env: UV_PRERELEASE=] [possible
              values: disallow, allow, if-necessary, explicit, if-necessary-or-explicit]
          --fork-strategy <FORK_STRATEGY>
              The strategy to use when selecting multiple versions of a given package across Python
              versions and platforms [env: UV_FORK_STRATEGY=] [possible values: fewest, requires-python]
          --exclude-newer <EXCLUDE_NEWER>
              Limit candidate packages to those that were uploaded prior to the given date [env:
              UV_EXCLUDE_NEWER=2024-03-25T00:00:00Z]
          --exclude-newer-package <EXCLUDE_NEWER_PACKAGE>
              Limit candidate packages for specific packages to those that were uploaded prior to the
              given date
          --no-sources
              Ignore the `tool.uv.sources` table when resolving dependencies. Used to lock against the
              standards-compliant, publishable package metadata, as opposed to using any workspace, Git,
              URL, or local path sources [env: UV_NO_SOURCES=]

    Installer options:
          --reinstall
              Reinstall all packages, regardless of whether they're already installed. Implies
              `--refresh`
          --reinstall-package <REINSTALL_PACKAGE>
              Reinstall a specific package, regardless of whether it's already installed. Implies
              `--refresh-package`
          --link-mode <LINK_MODE>
              The method to use when installing packages from the global cache [env: UV_LINK_MODE=]
              [possible values: clone, copy, hardlink, symlink]
          --compile-bytecode
              Compile Python files to bytecode after installation [env: UV_COMPILE_BYTECODE=]

    Build options:
      -C, --config-setting <CONFIG_SETTING>
              Settings to pass to the PEP 517 build backend, specified as `KEY=VALUE` pairs
          --config-settings-package <CONFIG_SETTINGS_PACKAGE>
              Settings to pass to the PEP 517 build backend for a specific package, specified as
              `PACKAGE:KEY=VALUE` pairs
          --no-build-isolation
              Disable isolation when building source distributions [env: UV_NO_BUILD_ISOLATION=]
          --no-build-isolation-package <NO_BUILD_ISOLATION_PACKAGE>
              Disable isolation when building source distributions for a specific package

    Cache options:
      -n, --no-cache
              Avoid reading from or writing to the cache, instead using a temporary directory for the
              duration of the operation [env: UV_NO_CACHE=]
          --cache-dir [CACHE_DIR]
              Path to the cache directory [env: UV_CACHE_DIR=]
          --refresh
              Refresh all cached data
          --refresh-package <REFRESH_PACKAGE>
              Refresh cached data for a specific package

    Python options:
      -p, --python <PYTHON>      The Python interpreter to use for building wheels. [env: UV_PYTHON=]
          --managed-python       Require use of uv-managed Python versions [env: UV_MANAGED_PYTHON=]
          --no-managed-python    Disable use of uv-managed Python versions [env: UV_NO_MANAGED_PYTHON=]
          --no-python-downloads  Disable automatic downloads of Python. [env:
                                 "UV_PYTHON_DOWNLOADS=never"]

    Global options:
      -q, --quiet...
              Use quiet output
      -v, --verbose...
              Use verbose output
          --color <COLOR_CHOICE>
              Control the use of color in output [possible values: auto, always, never]
          --native-tls
              Whether to load TLS certificates from the platform's native certificate store [env:
              UV_NATIVE_TLS=]
          --offline
              Disable network access [env: UV_OFFLINE=]
          --allow-insecure-host <ALLOW_INSECURE_HOST>
              Allow insecure connections to a host [env: UV_INSECURE_HOST=]
          --no-progress
              Hide all progress outputs [env: UV_NO_PROGRESS=]
          --directory <DIRECTORY>
              Change to the given directory prior to running the command [env: UV_WORKING_DIR=]
          --project <PROJECT>
              Discover a project in the given directory [env: UV_PROJECT=]
          --config-file <CONFIG_FILE>
              The path to a `uv.toml` file to use for configuration [env: UV_CONFIG_FILE=]
          --no-config
              Avoid discovering configuration files (`pyproject.toml`, `uv.toml`) [env: UV_NO_CONFIG=]
      -h, --help
              Display the concise help for this command

    Use `uv help pip wheel` for more details.

    ----- stderr -----
    "#
    );
}
