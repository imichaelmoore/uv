//! Global application state.

use std::path::PathBuf;

use super::{LoadingState, Notification, ProjectState, Tab};

/// Global application state for the uv GUI.
#[derive(Clone, Debug)]
pub struct AppState {
    /// The currently active tab.
    current_tab: Tab,
    /// The current project state, if a project is loaded.
    project: Option<ProjectState>,
    /// The current working directory.
    working_directory: Option<PathBuf>,
    /// Active notifications.
    notifications: Vec<Notification>,
    /// Global loading state.
    loading_state: LoadingState,
    /// Whether dark mode is enabled.
    dark_mode: bool,
    /// Cache directory path.
    cache_dir: Option<PathBuf>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    /// Create a new application state.
    pub fn new() -> Self {
        Self {
            current_tab: Tab::Project,
            project: None,
            working_directory: std::env::current_dir().ok(),
            notifications: Vec::new(),
            loading_state: LoadingState::Idle,
            dark_mode: true,
            cache_dir: None,
        }
    }

    /// Get the current tab.
    pub fn current_tab(&self) -> Tab {
        self.current_tab
    }

    /// Set the current tab.
    pub fn set_current_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
    }

    /// Get the current project state.
    pub fn project(&self) -> Option<&ProjectState> {
        self.project.as_ref()
    }

    /// Set the project state.
    pub fn set_project(&mut self, project: Option<ProjectState>) {
        self.project = project;
    }

    /// Get the working directory.
    pub fn working_directory(&self) -> Option<&PathBuf> {
        self.working_directory.as_ref()
    }

    /// Set the working directory.
    pub fn set_working_directory(&mut self, path: PathBuf) {
        self.working_directory = Some(path);
    }

    /// Get active notifications.
    pub fn notifications(&self) -> &[Notification] {
        &self.notifications
    }

    /// Add a notification.
    pub fn add_notification(&mut self, notification: Notification) {
        self.notifications.push(notification);
    }

    /// Remove a notification by index.
    pub fn remove_notification(&mut self, index: usize) {
        if index < self.notifications.len() {
            self.notifications.remove(index);
        }
    }

    /// Clear all notifications.
    pub fn clear_notifications(&mut self) {
        self.notifications.clear();
    }

    /// Get the loading state.
    pub fn loading_state(&self) -> LoadingState {
        self.loading_state
    }

    /// Set the loading state.
    pub fn set_loading_state(&mut self, state: LoadingState) {
        self.loading_state = state;
    }

    /// Check if dark mode is enabled.
    pub fn is_dark_mode(&self) -> bool {
        self.dark_mode
    }

    /// Toggle dark mode.
    pub fn toggle_dark_mode(&mut self) {
        self.dark_mode = !self.dark_mode;
    }

    /// Get the cache directory.
    pub fn cache_dir(&self) -> Option<&PathBuf> {
        self.cache_dir.as_ref()
    }

    /// Set the cache directory.
    pub fn set_cache_dir(&mut self, path: PathBuf) {
        self.cache_dir = Some(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.current_tab(), Tab::Project);
        assert!(state.project().is_none());
        assert!(state.is_dark_mode());
    }

    #[test]
    fn test_tab_switching() {
        let mut state = AppState::new();
        state.set_current_tab(Tab::Packages);
        assert_eq!(state.current_tab(), Tab::Packages);
    }

    #[test]
    fn test_notifications() {
        let mut state = AppState::new();
        assert!(state.notifications().is_empty());

        state.add_notification(Notification::info("Test"));
        assert_eq!(state.notifications().len(), 1);

        state.remove_notification(0);
        assert!(state.notifications().is_empty());
    }
}
