//! Integration tests for the uv GUI.

use uv_gui::state::{Package, ProjectState, Tab};

/// Test that the GUI can be imported and basic types work.
#[test]
fn test_gui_module_imports() {
    // Verify the main types are accessible
    let _tab = Tab::Project;
    let _package = Package::new("test");
    let _project = ProjectState::default();
}

/// Test project state with dependencies.
#[test]
fn test_project_with_dependencies() {
    let mut project = ProjectState::default();
    project.name = "test-project".to_string();
    project.version = Some("1.0.0".to_string());

    project.dependencies = vec![
        Package::with_version("requests", "2.31.0"),
        Package::with_version("numpy", "1.26.0"),
    ];

    project.dev_dependencies = vec![
        Package::with_version("pytest", "7.4.0"),
        Package::with_version("black", "23.10.0"),
    ];

    assert_eq!(project.dependency_count(), 4);
    assert_eq!(project.dependencies.len(), 2);
    assert_eq!(project.dev_dependencies.len(), 2);
}

/// Test package update detection.
#[test]
fn test_package_updates() {
    let mut pkg = Package::new("requests");
    pkg.installed_version = Some("2.30.0".to_string());
    pkg.latest_version = Some("2.31.0".to_string());
    pkg.update_available = true;

    assert!(pkg.is_installed());
    assert!(pkg.has_update());
}
