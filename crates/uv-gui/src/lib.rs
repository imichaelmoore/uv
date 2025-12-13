//! # uv-gui
//!
//! A graphical user interface for uv, the Python package manager.
//!
//! This crate provides a native GUI application built with GPUI (Zed's GPU-accelerated
//! UI framework) that allows users to manage Python packages, virtual environments,
//! and Python installations through an intuitive interface.
//!
//! ## Features
//!
//! - **Project Overview**: View and manage project dependencies
//! - **Package Browser**: Search and install packages from PyPI
//! - **Environment Management**: Create and manage virtual environments
//! - **Python Version Manager**: Install and switch Python versions
//! - **Dependency Tree**: Visualize project dependency graphs
//!
//! ## Usage
//!
//! ```no_run
//! use uv_gui::UvGuiApp;
//!
//! fn main() {
//!     UvGuiApp::run();
//! }
//! ```

mod actions;
mod app;
pub mod components;
pub mod state;
pub mod views;

pub use app::UvGuiApp;

/// Error types for the GUI application.
#[derive(Debug, thiserror::Error)]
pub enum GuiError {
    /// Failed to initialize the GUI framework.
    #[error("Failed to initialize GUI: {0}")]
    InitializationError(String),

    /// Failed to load project information.
    #[error("Failed to load project: {0}")]
    ProjectLoadError(String),

    /// Failed to execute a uv command.
    #[error("Command failed: {0}")]
    CommandError(String),

    /// Network error when fetching package information.
    #[error("Network error: {0}")]
    NetworkError(#[from] anyhow::Error),
}

/// Result type for GUI operations.
pub type GuiResult<T> = Result<T, GuiError>;
