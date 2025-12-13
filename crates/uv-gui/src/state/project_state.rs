//! Project-specific state management.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// State for a loaded Python project.
#[derive(Clone, Debug, Default)]
pub struct ProjectState {
    /// The project name.
    pub name: String,
    /// The project version.
    pub version: Option<String>,
    /// The project description.
    pub description: Option<String>,
    /// The project root directory.
    pub root: PathBuf,
    /// The path to pyproject.toml, if present.
    pub pyproject_path: Option<PathBuf>,
    /// The path to requirements.txt, if present.
    pub requirements_path: Option<PathBuf>,
    /// Dependencies declared in the project.
    pub dependencies: Vec<Package>,
    /// Development dependencies.
    pub dev_dependencies: Vec<Package>,
    /// Optional dependencies by group.
    pub optional_dependencies: Vec<(String, Vec<Package>)>,
    /// The active virtual environment, if any.
    pub active_environment: Option<Environment>,
    /// Available virtual environments.
    pub environments: Vec<Environment>,
    /// The Python version used by the project.
    pub python_version: Option<String>,
    /// Whether the project has a lockfile.
    pub has_lockfile: bool,
    /// Whether the project is out of sync with the lockfile.
    pub needs_sync: bool,
}

impl ProjectState {
    /// Create a new project state from a directory.
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            root: path,
            ..Default::default()
        }
    }

    /// Check if this is a valid uv project.
    pub fn is_valid(&self) -> bool {
        self.pyproject_path.is_some() || self.requirements_path.is_some()
    }

    /// Get all dependencies including dev dependencies.
    pub fn all_dependencies(&self) -> Vec<&Package> {
        self.dependencies
            .iter()
            .chain(self.dev_dependencies.iter())
            .collect()
    }

    /// Get the total number of dependencies.
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len() + self.dev_dependencies.len()
    }
}

/// Information about a Python package.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    /// The package name.
    pub name: String,
    /// The installed version, if installed.
    pub installed_version: Option<String>,
    /// The required version specifier.
    pub required_version: Option<String>,
    /// The latest available version.
    pub latest_version: Option<String>,
    /// Whether this is a development dependency.
    pub is_dev: bool,
    /// Whether an update is available.
    pub update_available: bool,
    /// The package description.
    pub description: Option<String>,
    /// The package homepage URL.
    pub homepage: Option<String>,
    /// The package documentation URL.
    pub documentation: Option<String>,
    /// The package repository URL.
    pub repository: Option<String>,
    /// Package authors.
    pub authors: Vec<String>,
    /// Package license.
    pub license: Option<String>,
    /// Package keywords.
    pub keywords: Vec<String>,
    /// Direct dependencies of this package.
    pub dependencies: Vec<String>,
}

impl Package {
    /// Create a new package with just a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Create a new package with name and version.
    pub fn with_version(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            installed_version: Some(version.into()),
            ..Default::default()
        }
    }

    /// Check if the package is installed.
    pub fn is_installed(&self) -> bool {
        self.installed_version.is_some()
    }

    /// Check if an update is available.
    pub fn has_update(&self) -> bool {
        self.update_available
    }
}

/// Information about a virtual environment.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    /// The environment name.
    pub name: String,
    /// The path to the environment.
    pub path: PathBuf,
    /// The Python version in this environment.
    pub python_version: String,
    /// Whether this environment is currently active.
    pub is_active: bool,
    /// The number of installed packages.
    pub package_count: usize,
    /// When the environment was created.
    pub created_at: Option<String>,
    /// The size of the environment on disk.
    pub size_bytes: Option<u64>,
}

impl Environment {
    /// Create a new environment.
    pub fn new(name: impl Into<String>, path: PathBuf) -> Self {
        Self {
            name: name.into(),
            path,
            ..Default::default()
        }
    }

    /// Get a display-friendly size string.
    pub fn size_display(&self) -> String {
        match self.size_bytes {
            Some(bytes) if bytes >= 1_073_741_824 => {
                format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
            }
            Some(bytes) if bytes >= 1_048_576 => {
                format!("{:.1} MB", bytes as f64 / 1_048_576.0)
            }
            Some(bytes) if bytes >= 1024 => format!("{:.1} KB", bytes as f64 / 1024.0),
            Some(bytes) => format!("{bytes} B"),
            None => "Unknown".to_string(),
        }
    }
}

/// Information about an installed Python version.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonInstallation {
    /// The Python version (e.g., "3.12.0").
    pub version: String,
    /// The path to the Python executable.
    pub path: PathBuf,
    /// Whether this is the default Python version.
    pub is_default: bool,
    /// Whether this is a system Python or managed by uv.
    pub is_managed: bool,
    /// The implementation (CPython, PyPy, etc.).
    pub implementation: String,
    /// The architecture (x86_64, arm64, etc.).
    pub architecture: Option<String>,
}

impl PythonInstallation {
    /// Create a new Python installation.
    pub fn new(version: impl Into<String>, path: PathBuf) -> Self {
        Self {
            version: version.into(),
            path,
            implementation: "CPython".to_string(),
            ..Default::default()
        }
    }

    /// Get a display string for this installation.
    pub fn display(&self) -> String {
        let mut s = format!("{} {}", self.implementation, self.version);
        if let Some(arch) = &self.architecture {
            s.push_str(&format!(" ({arch})"));
        }
        if self.is_default {
            s.push_str(" [default]");
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_new() {
        let pkg = Package::new("requests");
        assert_eq!(pkg.name, "requests");
        assert!(!pkg.is_installed());
    }

    #[test]
    fn test_package_with_version() {
        let pkg = Package::with_version("requests", "2.31.0");
        assert_eq!(pkg.name, "requests");
        assert_eq!(pkg.installed_version, Some("2.31.0".to_string()));
        assert!(pkg.is_installed());
    }

    #[test]
    fn test_environment_size_display() {
        let mut env = Environment::new("test", PathBuf::from("/tmp/test"));

        env.size_bytes = Some(500);
        assert_eq!(env.size_display(), "500 B");

        env.size_bytes = Some(2048);
        assert_eq!(env.size_display(), "2.0 KB");

        env.size_bytes = Some(5_242_880);
        assert_eq!(env.size_display(), "5.0 MB");

        env.size_bytes = Some(2_147_483_648);
        assert_eq!(env.size_display(), "2.0 GB");
    }

    #[test]
    fn test_python_installation_display() {
        let mut py = PythonInstallation::new("3.12.0", PathBuf::from("/usr/bin/python3"));
        assert_eq!(py.display(), "CPython 3.12.0");

        py.architecture = Some("x86_64".to_string());
        assert_eq!(py.display(), "CPython 3.12.0 (x86_64)");

        py.is_default = true;
        assert_eq!(py.display(), "CPython 3.12.0 (x86_64) [default]");
    }
}
