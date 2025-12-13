use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;

use crate::common::{TestContext, uv_snapshot};

#[test]
fn download_single_package() -> Result<()> {
    let context = TestContext::new("3.12")
        .with_filtered_python_names()
        .with_filtered_virtualenv_bin()
        .with_filtered_exe_suffix();
    let download_dir = context.temp_dir.child("downloads");
    download_dir.create_dir_all()?;

    uv_snapshot!(context.filters(), context.pip_download()
        .arg("iniconfig==2.0.0")
        .arg("-d")
        .arg(download_dir.path()), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Using CPython 3.12.[X] interpreter at: .venv/[BIN]/[PYTHON]
    Resolved 1 package in [TIME]
     Downloaded iniconfig-2.0.0-py3-none-any.whl
    Downloaded 1 package in [TIME]
    "###
    );

    // Verify the wheel was downloaded
    assert!(download_dir.child("iniconfig-2.0.0-py3-none-any.whl").exists());

    Ok(())
}

#[test]
fn download_no_deps() -> Result<()> {
    let context = TestContext::new("3.12")
        .with_filtered_python_names()
        .with_filtered_virtualenv_bin()
        .with_filtered_exe_suffix();
    let download_dir = context.temp_dir.child("downloads");
    download_dir.create_dir_all()?;

    uv_snapshot!(context.filters(), context.pip_download()
        .arg("flask==3.0.0")
        .arg("--no-deps")
        .arg("-d")
        .arg(download_dir.path()), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Using CPython 3.12.[X] interpreter at: .venv/[BIN]/[PYTHON]
    Resolved 1 package in [TIME]
     Downloaded flask-3.0.0-py3-none-any.whl
    Downloaded 1 package in [TIME]
    "###
    );

    // Verify only the main wheel was downloaded (no dependencies)
    assert!(download_dir.child("flask-3.0.0-py3-none-any.whl").exists());
    assert!(!download_dir.child("werkzeug-3.0.1-py3-none-any.whl").exists());
    assert!(!download_dir.child("jinja2-3.1.3-py3-none-any.whl").exists());

    Ok(())
}

#[test]
fn download_from_requirements_file() -> Result<()> {
    let context = TestContext::new("3.12")
        .with_filtered_python_names()
        .with_filtered_virtualenv_bin()
        .with_filtered_exe_suffix();
    let download_dir = context.temp_dir.child("downloads");
    download_dir.create_dir_all()?;

    let requirements_txt = context.temp_dir.child("requirements.txt");
    requirements_txt.write_str("iniconfig==2.0.0\ntomli==2.0.1")?;

    uv_snapshot!(context.filters(), context.pip_download()
        .arg("-r")
        .arg(requirements_txt.path())
        .arg("-d")
        .arg(download_dir.path()), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Using CPython 3.12.[X] interpreter at: .venv/[BIN]/[PYTHON]
    Resolved 2 packages in [TIME]
     Downloaded iniconfig-2.0.0-py3-none-any.whl
     Downloaded tomli-2.0.1-py3-none-any.whl
    Downloaded 2 packages in [TIME]
    "###
    );

    // Verify the wheels were downloaded
    assert!(download_dir.child("iniconfig-2.0.0-py3-none-any.whl").exists());
    assert!(download_dir.child("tomli-2.0.1-py3-none-any.whl").exists());

    Ok(())
}

#[test]
fn download_to_current_directory() -> Result<()> {
    let context = TestContext::new("3.12")
        .with_filtered_python_names()
        .with_filtered_virtualenv_bin()
        .with_filtered_exe_suffix();

    uv_snapshot!(context.filters(), context.pip_download()
        .arg("iniconfig==2.0.0")
        .current_dir(context.temp_dir.path()), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Using CPython 3.12.[X] interpreter at: .venv/[BIN]/[PYTHON]
    Resolved 1 package in [TIME]
     Downloaded iniconfig-2.0.0-py3-none-any.whl
    Downloaded 1 package in [TIME]
    "###
    );

    // Verify the wheel was downloaded to the current directory
    assert!(context.temp_dir.child("iniconfig-2.0.0-py3-none-any.whl").exists());

    Ok(())
}

#[test]
fn download_skip_existing() -> Result<()> {
    let context = TestContext::new("3.12")
        .with_filtered_python_names()
        .with_filtered_virtualenv_bin()
        .with_filtered_exe_suffix();
    let download_dir = context.temp_dir.child("downloads");
    download_dir.create_dir_all()?;

    // First download
    context
        .pip_download()
        .arg("iniconfig==2.0.0")
        .arg("-d")
        .arg(download_dir.path())
        .assert()
        .success();

    // Second download should skip the existing file
    uv_snapshot!(context.filters(), context.pip_download()
        .arg("iniconfig==2.0.0")
        .arg("-d")
        .arg(download_dir.path()), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Using CPython 3.12.[X] interpreter at: .venv/[BIN]/[PYTHON]
    Resolved 1 package in [TIME]
     Skipping iniconfig-2.0.0-py3-none-any.whl (already exists)
    Downloaded 0 packages in [TIME]
    "###
    );

    Ok(())
}

#[test]
fn download_multiple_packages() -> Result<()> {
    let context = TestContext::new("3.12")
        .with_filtered_python_names()
        .with_filtered_virtualenv_bin()
        .with_filtered_exe_suffix();
    let download_dir = context.temp_dir.child("downloads");
    download_dir.create_dir_all()?;

    uv_snapshot!(context.filters(), context.pip_download()
        .arg("iniconfig==2.0.0")
        .arg("tomli==2.0.1")
        .arg("-d")
        .arg(download_dir.path()), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Using CPython 3.12.[X] interpreter at: .venv/[BIN]/[PYTHON]
    Resolved 2 packages in [TIME]
     Downloaded iniconfig-2.0.0-py3-none-any.whl
     Downloaded tomli-2.0.1-py3-none-any.whl
    Downloaded 2 packages in [TIME]
    "###
    );

    // Verify the wheels were downloaded
    assert!(download_dir.child("iniconfig-2.0.0-py3-none-any.whl").exists());
    assert!(download_dir.child("tomli-2.0.1-py3-none-any.whl").exists());

    Ok(())
}

#[test]
fn download_missing_requirements_file() {
    let context = TestContext::new("3.12");
    let download_dir = context.temp_dir.child("downloads");

    uv_snapshot!(context.filters(), context.pip_download()
        .arg("-r")
        .arg("requirements.txt")
        .arg("-d")
        .arg(download_dir.path()), @r###"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: File not found: `requirements.txt`
    "###
    );
}

#[test]
fn download_no_packages_specified() {
    let context = TestContext::new("3.12");

    uv_snapshot!(context.filters(), context.pip_download(), @r###"
    success: false
    exit_code: 2
    ----- stdout -----

    ----- stderr -----
    error: the following required arguments were not provided:
      <PACKAGE|--requirements <REQUIREMENTS>|--editable <EDITABLE>>

    Usage: uv pip download --cache-dir [CACHE_DIR] --exclude-newer <EXCLUDE_NEWER> <PACKAGE|--requirements <REQUIREMENTS>|--editable <EDITABLE>>

    For more information, try '--help'.
    "###
    );
}

#[test]
fn download_with_constraint() -> Result<()> {
    let context = TestContext::new("3.12")
        .with_filtered_python_names()
        .with_filtered_virtualenv_bin()
        .with_filtered_exe_suffix();
    let download_dir = context.temp_dir.child("downloads");
    download_dir.create_dir_all()?;

    let constraints_txt = context.temp_dir.child("constraints.txt");
    constraints_txt.write_str("iniconfig<2.1.0")?;

    uv_snapshot!(context.filters(), context.pip_download()
        .arg("iniconfig")
        .arg("-c")
        .arg(constraints_txt.path())
        .arg("-d")
        .arg(download_dir.path()), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Using CPython 3.12.[X] interpreter at: .venv/[BIN]/[PYTHON]
    Resolved 1 package in [TIME]
     Downloaded iniconfig-2.0.0-py3-none-any.whl
    Downloaded 1 package in [TIME]
    "###
    );

    Ok(())
}

#[test]
fn download_only_binary() -> Result<()> {
    let context = TestContext::new("3.12")
        .with_filtered_python_names()
        .with_filtered_virtualenv_bin()
        .with_filtered_exe_suffix();
    let download_dir = context.temp_dir.child("downloads");
    download_dir.create_dir_all()?;

    uv_snapshot!(context.filters(), context.pip_download()
        .arg("iniconfig==2.0.0")
        .arg("--only-binary")
        .arg(":all:")
        .arg("-d")
        .arg(download_dir.path()), @r###"
    success: true
    exit_code: 0
    ----- stdout -----

    ----- stderr -----
    Using CPython 3.12.[X] interpreter at: .venv/[BIN]/[PYTHON]
    Resolved 1 package in [TIME]
     Downloaded iniconfig-2.0.0-py3-none-any.whl
    Downloaded 1 package in [TIME]
    "###
    );

    Ok(())
}
