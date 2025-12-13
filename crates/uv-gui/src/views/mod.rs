//! View implementations for the uv GUI.
//!
//! This module contains all the main views used in the application,
//! including the project overview, package browser, and settings views.

mod environments;
mod main_window;
mod packages;
mod project;
mod python;
mod settings;

pub use environments::EnvironmentsView;
pub use main_window::MainWindow;
pub use packages::PackagesView;
pub use project::ProjectView;
pub use python::PythonView;
pub use settings::SettingsView;
