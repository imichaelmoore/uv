//! Application state management.
//!
//! This module contains all the state types used by the GUI application,
//! including project state, package information, and UI state.

mod app_state;
mod project_state;

pub use app_state::AppState;
pub use project_state::{Environment, Package, ProjectState, PythonInstallation};

use serde::{Deserialize, Serialize};

/// The available tabs in the main window.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tab {
    /// Project overview showing dependencies and project info.
    #[default]
    Project,
    /// Package browser for searching and installing packages.
    Packages,
    /// Virtual environment management.
    Environments,
    /// Python version management.
    Python,
    /// Application settings.
    Settings,
}

/// Loading state for async operations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoadingState {
    /// Not loading.
    #[default]
    Idle,
    /// Loading data.
    Loading,
    /// Successfully loaded.
    Loaded,
    /// Error occurred during loading.
    Error,
}

/// Notification type for user feedback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotificationType {
    /// Informational message.
    Info,
    /// Success message.
    Success,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
}

/// A notification to display to the user.
#[derive(Clone, Debug)]
pub struct Notification {
    /// The notification message.
    pub message: String,
    /// The notification type.
    pub notification_type: NotificationType,
    /// Whether the notification is dismissible.
    pub dismissible: bool,
}

impl Notification {
    /// Create a new info notification.
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Info,
            dismissible: true,
        }
    }

    /// Create a new success notification.
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Success,
            dismissible: true,
        }
    }

    /// Create a new warning notification.
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Warning,
            dismissible: true,
        }
    }

    /// Create a new error notification.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            notification_type: NotificationType::Error,
            dismissible: true,
        }
    }
}
