//! GUI actions and commands.
//!
//! This module defines all the actions that can be triggered in the GUI,
//! such as adding packages, switching tabs, and refreshing data.

// These action types are defined for future use with GPUI's action system.
#![allow(dead_code, unreachable_pub)]

use serde::{Deserialize, Serialize};

use crate::state::Tab;

/// Action to switch between tabs in the main window.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct SwitchTab {
    pub tab: Tab,
}

/// Action to search for packages.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SearchPackages {
    pub query: String,
}

/// Action to add a package to the project.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AddPackage {
    pub name: String,
    pub version: Option<String>,
    pub dev: bool,
}

/// Action to remove a package from the project.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RemovePackage {
    pub name: String,
}

/// Action to refresh the package list.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RefreshPackages;

/// Action to open a project directory.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct OpenProject {
    pub path: Option<String>,
}

/// Action to create a new virtual environment.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct CreateEnvironment {
    pub name: String,
    pub python_version: Option<String>,
}

/// Action to delete a virtual environment.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DeleteEnvironment {
    pub path: String,
}

/// Action to activate a virtual environment.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ActivateEnvironment {
    pub path: String,
}

/// Action to install a Python version.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct InstallPython {
    pub version: String,
}

/// Action to uninstall a Python version.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UninstallPython {
    pub version: String,
}

/// Action to set the default Python version.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SetDefaultPython {
    pub version: String,
}

/// Action to update settings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdateSetting {
    pub key: String,
    pub value: String,
}

/// Action to sync the project.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SyncProject;

/// Action to lock the project dependencies.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LockProject;

/// Action to run a command in the project context.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct RunCommand {
    pub command: String,
    pub args: Vec<String>,
}

/// Action to show package details.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ShowPackageDetails {
    pub name: String,
}

/// Action to update a package.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdatePackage {
    pub name: String,
    pub version: Option<String>,
}

/// Action to update all packages.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct UpdateAllPackages;
