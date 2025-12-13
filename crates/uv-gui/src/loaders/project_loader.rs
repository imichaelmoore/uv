//! High-level project loading that combines all sources.
//!
//! This module provides a unified interface for loading project
//! information from pyproject.toml and uv.lock.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use uv_normalize::PackageName;
use uv_pep440::Version;
use uv_workspace::pyproject::PyProjectToml;

use thiserror::Error;

use super::dependency_loader::{DependencyLoadError, DependencyLoader, LoadedDependency};
use super::lockfile_loader::{LockedPackage, LockfileLoadError, LockfileLoader};
use crate::state::Package;

/// Complete loaded project data.
#[derive(Clone, Debug, Default)]
pub struct LoadedProject {
    /// The project name.
    pub name: String,
    /// The project version.
    pub version: Option<String>,
    /// The project root directory.
    pub root: PathBuf,
    /// Production dependencies.
    pub dependencies: Vec<Package>,
    /// Development dependencies (merged from all dev sources).
    pub dev_dependencies: Vec<Package>,
    /// Whether the project has a lockfile.
    pub has_lockfile: bool,
}

/// Error combining all loading errors.
#[derive(Debug, Error)]
pub enum ProjectLoadError {
    /// Failed to read pyproject.toml.
    #[error("Failed to read pyproject.toml: {0}")]
    ReadError(#[from] std::io::Error),
    /// Failed to load dependencies.
    #[error("Failed to load dependencies: {0}")]
    DependencyError(#[from] DependencyLoadError),
    /// Failed to load lockfile.
    #[error("Failed to load lockfile: {0}")]
    LockfileError(#[from] LockfileLoadError),
    /// Failed to parse pyproject.toml.
    #[error("Failed to parse pyproject.toml: {0}")]
    ParseError(#[from] uv_workspace::pyproject::PyprojectTomlError),
}

/// High-level project loader that combines all data sources.
pub struct ProjectLoader;

impl ProjectLoader {
    /// Load a complete project with all dependency information.
    pub fn load(project_root: &Path) -> Result<LoadedProject, ProjectLoadError> {
        let pyproject_path = project_root.join("pyproject.toml");
        let lock_path = project_root.join("uv.lock");

        // Load and parse pyproject.toml for name/version
        let content = fs_err::read_to_string(&pyproject_path)?;
        let pyproject = PyProjectToml::from_string(content)?;

        let name = pyproject
            .project
            .as_ref()
            .map(|p| p.name.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let version = pyproject
            .project
            .as_ref()
            .and_then(|p| p.version.as_ref())
            .map(|v| v.to_string());

        // Load raw dependencies
        let raw_deps = DependencyLoader::load(&pyproject_path)?;

        // Load locked versions (if available)
        let locked_packages =
            LockfileLoader::try_load(&lock_path).map_err(ProjectLoadError::LockfileError)?;
        let version_map = Self::build_version_map(&locked_packages);

        // Combine into Package structs with versions
        let (dependencies, dev_dependencies) = Self::categorize_and_enrich(raw_deps, &version_map);

        Ok(LoadedProject {
            name,
            version,
            root: project_root.to_path_buf(),
            dependencies,
            dev_dependencies,
            has_lockfile: locked_packages.is_some(),
        })
    }

    /// Build a map from package name to locked version.
    fn build_version_map(locked: &Option<Vec<LockedPackage>>) -> HashMap<PackageName, Version> {
        locked
            .as_ref()
            .map(|packages| {
                packages
                    .iter()
                    .map(|pkg| (pkg.name.clone(), pkg.version.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Categorize dependencies and enrich with version info.
    fn categorize_and_enrich(
        raw_deps: Vec<LoadedDependency>,
        version_map: &HashMap<PackageName, Version>,
    ) -> (Vec<Package>, Vec<Package>) {
        let mut dependencies = Vec::new();
        let mut dev_dependencies = Vec::new();
        let mut seen_dev: HashMap<PackageName, usize> = HashMap::new();

        for dep in raw_deps {
            let installed_version = version_map.get(&dep.name).map(|v| v.to_string());
            let source_label = dep.source.label().to_string();

            let package = Package {
                name: dep.name.to_string(),
                installed_version,
                required_version: Some(dep.requirement_string.clone()),
                is_dev: dep.is_dev(),
                source_label: Some(source_label),
                ..Default::default()
            };

            if dep.is_dev() {
                // Deduplicate dev dependencies by name (keep first occurrence)
                if let std::collections::hash_map::Entry::Vacant(e) =
                    seen_dev.entry(dep.name.clone())
                {
                    e.insert(dev_dependencies.len());
                    dev_dependencies.push(package);
                }
            } else {
                dependencies.push(package);
            }
        }

        (dependencies, dev_dependencies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_version_map_empty() {
        let map = ProjectLoader::build_version_map(&None);
        assert!(map.is_empty());
    }

    #[test]
    fn test_build_version_map_with_packages() {
        use std::str::FromStr;

        let packages = vec![LockedPackage {
            name: PackageName::from_str("requests").unwrap(),
            version: Version::from_str("2.31.0").unwrap(),
        }];

        let map = ProjectLoader::build_version_map(&Some(packages));
        assert_eq!(map.len(), 1);

        let name = PackageName::from_str("requests").unwrap();
        assert_eq!(map.get(&name).unwrap().to_string(), "2.31.0");
    }
}
