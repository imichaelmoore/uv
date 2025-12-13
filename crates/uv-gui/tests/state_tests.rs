//! Tests for the application state management.

use std::path::PathBuf;

use uv_gui::state::{
    AppState, Environment, LoadingState, Notification, NotificationType, Package, ProjectState,
    PythonInstallation, Tab,
};

#[test]
fn test_app_state_initialization() {
    let state = AppState::new();
    assert_eq!(state.current_tab(), Tab::Project);
    assert!(state.project().is_none());
    assert!(state.notifications().is_empty());
    assert_eq!(state.loading_state(), LoadingState::Idle);
    assert!(state.is_dark_mode());
}

#[test]
fn test_tab_switching() {
    let mut state = AppState::new();

    state.set_current_tab(Tab::Packages);
    assert_eq!(state.current_tab(), Tab::Packages);

    state.set_current_tab(Tab::Environments);
    assert_eq!(state.current_tab(), Tab::Environments);

    state.set_current_tab(Tab::Python);
    assert_eq!(state.current_tab(), Tab::Python);

    state.set_current_tab(Tab::Settings);
    assert_eq!(state.current_tab(), Tab::Settings);

    state.set_current_tab(Tab::Project);
    assert_eq!(state.current_tab(), Tab::Project);
}

#[test]
fn test_notification_management() {
    let mut state = AppState::new();

    // Add notifications
    state.add_notification(Notification::info("Info message"));
    state.add_notification(Notification::success("Success message"));
    state.add_notification(Notification::warning("Warning message"));
    state.add_notification(Notification::error("Error message"));

    assert_eq!(state.notifications().len(), 4);
    assert_eq!(
        state.notifications()[0].notification_type,
        NotificationType::Info
    );
    assert_eq!(
        state.notifications()[1].notification_type,
        NotificationType::Success
    );
    assert_eq!(
        state.notifications()[2].notification_type,
        NotificationType::Warning
    );
    assert_eq!(
        state.notifications()[3].notification_type,
        NotificationType::Error
    );

    // Remove notification
    state.remove_notification(1);
    assert_eq!(state.notifications().len(), 3);

    // Clear all
    state.clear_notifications();
    assert!(state.notifications().is_empty());
}

#[test]
fn test_loading_state() {
    let mut state = AppState::new();

    state.set_loading_state(LoadingState::Loading);
    assert_eq!(state.loading_state(), LoadingState::Loading);

    state.set_loading_state(LoadingState::Loaded);
    assert_eq!(state.loading_state(), LoadingState::Loaded);

    state.set_loading_state(LoadingState::Error);
    assert_eq!(state.loading_state(), LoadingState::Error);

    state.set_loading_state(LoadingState::Idle);
    assert_eq!(state.loading_state(), LoadingState::Idle);
}

#[test]
fn test_dark_mode_toggle() {
    let mut state = AppState::new();
    assert!(state.is_dark_mode());

    state.toggle_dark_mode();
    assert!(!state.is_dark_mode());

    state.toggle_dark_mode();
    assert!(state.is_dark_mode());
}

#[test]
fn test_project_state() {
    let mut project = ProjectState::from_path(PathBuf::from("/test/project"));
    assert_eq!(project.root, PathBuf::from("/test/project"));
    assert!(!project.is_valid());

    project.pyproject_path = Some(PathBuf::from("/test/project/pyproject.toml"));
    assert!(project.is_valid());

    // Test dependencies
    project.dependencies = vec![Package::new("requests"), Package::new("numpy")];
    project.dev_dependencies = vec![Package::new("pytest")];

    assert_eq!(project.dependency_count(), 3);
    assert_eq!(project.all_dependencies().len(), 3);
}

#[test]
fn test_package_creation() {
    let pkg = Package::new("requests");
    assert_eq!(pkg.name, "requests");
    assert!(!pkg.is_installed());
    assert!(!pkg.has_update());

    let pkg_with_version = Package::with_version("numpy", "1.26.0");
    assert_eq!(pkg_with_version.name, "numpy");
    assert_eq!(
        pkg_with_version.installed_version,
        Some("1.26.0".to_string())
    );
    assert!(pkg_with_version.is_installed());
}

#[test]
fn test_environment() {
    let mut env = Environment::new("test-env", PathBuf::from("/home/user/.venv"));
    assert_eq!(env.name, "test-env");
    assert_eq!(env.path, PathBuf::from("/home/user/.venv"));
    assert!(!env.is_active);

    // Test size display
    env.size_bytes = None;
    assert_eq!(env.size_display(), "Unknown");

    env.size_bytes = Some(500);
    assert_eq!(env.size_display(), "500 B");

    env.size_bytes = Some(2048);
    assert_eq!(env.size_display(), "2.0 KB");

    env.size_bytes = Some(5 * 1024 * 1024);
    assert_eq!(env.size_display(), "5.0 MB");

    env.size_bytes = Some(2 * 1024 * 1024 * 1024);
    assert_eq!(env.size_display(), "2.0 GB");
}

#[test]
fn test_python_installation() {
    let mut py = PythonInstallation::new("3.12.0", PathBuf::from("/usr/bin/python3"));
    assert_eq!(py.version, "3.12.0");
    assert_eq!(py.implementation, "CPython");
    assert!(!py.is_default);
    assert!(!py.is_managed);

    assert_eq!(py.display(), "CPython 3.12.0");

    py.architecture = Some("x86_64".to_string());
    assert_eq!(py.display(), "CPython 3.12.0 (x86_64)");

    py.is_default = true;
    assert_eq!(py.display(), "CPython 3.12.0 (x86_64) [default]");
}

#[test]
fn test_tab_default() {
    let tab: Tab = Default::default();
    assert_eq!(tab, Tab::Project);
}
