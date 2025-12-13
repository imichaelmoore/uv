//! Dependency loading from pyproject.toml.
//!
//! This module handles parsing dependencies from all supported locations
//! in a pyproject.toml file.

use std::path::Path;
use std::str::FromStr;

use uv_normalize::{DEV_DEPENDENCIES, ExtraName, GroupName, PackageName};
use uv_workspace::pyproject::PyProjectToml;

use thiserror::Error;

/// The source location of a dependency declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependencySource {
    /// `project.dependencies`
    Project,
    /// `project.optional-dependencies.{extra}`
    OptionalDependency(ExtraName),
    /// `tool.uv.dev-dependencies`
    ToolUvDevDependencies,
    /// `dependency-groups.{group}` (PEP 735)
    DependencyGroup(GroupName),
}

impl DependencySource {
    /// Returns true if this source represents a dev dependency.
    pub fn is_dev(&self) -> bool {
        match self {
            Self::Project => false,
            Self::OptionalDependency(extra) => extra.as_str() == "dev",
            Self::ToolUvDevDependencies => true,
            Self::DependencyGroup(group) => group == &*DEV_DEPENDENCIES,
        }
    }

    /// Returns a human-readable label for this source.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Project => "dependencies",
            Self::OptionalDependency(_) => "optional",
            Self::ToolUvDevDependencies => "tool.uv",
            Self::DependencyGroup(_) => "group",
        }
    }
}

/// A dependency loaded from pyproject.toml.
#[derive(Clone, Debug)]
pub struct LoadedDependency {
    /// The package name (normalized).
    pub name: PackageName,
    /// The raw requirement string (e.g., "requests>=2.28.0").
    pub requirement_string: String,
    /// The source location of this dependency.
    pub source: DependencySource,
}

impl LoadedDependency {
    /// Returns true if this is a dev dependency.
    pub fn is_dev(&self) -> bool {
        self.source.is_dev()
    }
}

/// Error type for dependency loading.
#[derive(Debug, Error)]
pub enum DependencyLoadError {
    /// Failed to read the file.
    #[error("Failed to read pyproject.toml: {0}")]
    ReadError(#[from] std::io::Error),
    /// Failed to parse the TOML.
    #[error("Failed to parse pyproject.toml: {0}")]
    ParseError(#[from] uv_workspace::pyproject::PyprojectTomlError),
}

/// Loads dependencies from a pyproject.toml file.
pub struct DependencyLoader;

impl DependencyLoader {
    /// Load all dependencies from a pyproject.toml file.
    pub fn load(path: &Path) -> Result<Vec<LoadedDependency>, DependencyLoadError> {
        let content = fs_err::read_to_string(path)?;
        let pyproject = PyProjectToml::from_string(content)?;

        let mut dependencies = Vec::new();

        // 1. Load project.dependencies
        dependencies.extend(Self::load_project_dependencies(&pyproject));

        // 2. Load project.optional-dependencies (all extras, including "dev")
        dependencies.extend(Self::load_optional_dependencies(&pyproject));

        // 3. Load tool.uv.dev-dependencies
        dependencies.extend(Self::load_tool_uv_dev_dependencies(&pyproject));

        // 4. Load dependency-groups (PEP 735)
        dependencies.extend(Self::load_dependency_groups(&pyproject));

        Ok(dependencies)
    }

    /// Load dependencies from `project.dependencies`.
    fn load_project_dependencies(pyproject: &PyProjectToml) -> Vec<LoadedDependency> {
        let Some(project) = &pyproject.project else {
            return Vec::new();
        };

        let Some(deps) = &project.dependencies else {
            return Vec::new();
        };

        deps.iter()
            .filter_map(|req_str| {
                Self::parse_requirement(req_str, DependencySource::Project).or_else(|| {
                    tracing::warn!("Failed to parse dependency: {req_str}");
                    None
                })
            })
            .collect()
    }

    /// Load dependencies from `project.optional-dependencies`.
    fn load_optional_dependencies(pyproject: &PyProjectToml) -> Vec<LoadedDependency> {
        let Some(project) = &pyproject.project else {
            return Vec::new();
        };

        let Some(opt_deps) = &project.optional_dependencies else {
            return Vec::new();
        };

        opt_deps
            .iter()
            .flat_map(|(extra, deps)| {
                let source = DependencySource::OptionalDependency(extra.clone());
                deps.iter().filter_map(move |req_str| {
                    Self::parse_requirement(req_str, source.clone()).or_else(|| {
                        tracing::warn!("Failed to parse optional dependency [{extra}]: {req_str}");
                        None
                    })
                })
            })
            .collect()
    }

    /// Load dependencies from `tool.uv.dev-dependencies`.
    fn load_tool_uv_dev_dependencies(pyproject: &PyProjectToml) -> Vec<LoadedDependency> {
        let Some(tool) = &pyproject.tool else {
            return Vec::new();
        };

        let Some(uv) = &tool.uv else {
            return Vec::new();
        };

        let Some(dev_deps) = &uv.dev_dependencies else {
            return Vec::new();
        };

        dev_deps
            .iter()
            .filter_map(|req| {
                let name = req.name.clone();
                let requirement_string = req.to_string();
                Some(LoadedDependency {
                    name,
                    requirement_string,
                    source: DependencySource::ToolUvDevDependencies,
                })
            })
            .collect()
    }

    /// Load dependencies from `dependency-groups` (PEP 735).
    fn load_dependency_groups(pyproject: &PyProjectToml) -> Vec<LoadedDependency> {
        let Some(dep_groups) = &pyproject.dependency_groups else {
            return Vec::new();
        };

        dep_groups
            .iter()
            .flat_map(|(group_name, specifiers)| {
                let source = DependencySource::DependencyGroup(group_name.clone());
                specifiers.iter().filter_map(move |specifier| {
                    match specifier {
                        uv_pypi_types::DependencyGroupSpecifier::Requirement(req_str) => {
                            Self::parse_requirement(req_str, source.clone()).or_else(|| {
                                tracing::warn!(
                                    "Failed to parse dependency-group [{group_name}]: {req_str}"
                                );
                                None
                            })
                        }
                        // Skip include-group references - they're resolved elsewhere
                        uv_pypi_types::DependencyGroupSpecifier::IncludeGroup { .. } => None,
                        uv_pypi_types::DependencyGroupSpecifier::Object { .. } => None,
                    }
                })
            })
            .collect()
    }

    /// Parse a requirement string into a LoadedDependency.
    fn parse_requirement(req_str: &str, source: DependencySource) -> Option<LoadedDependency> {
        // Extract package name from the requirement string.
        // PEP 508 format: name [extras] [version] [; markers]
        let name_end = req_str
            .find(|c: char| !c.is_alphanumeric() && c != '-' && c != '_' && c != '.')
            .unwrap_or(req_str.len());

        let name_str = &req_str[..name_end];
        let name = PackageName::from_str(name_str).ok()?;

        Some(LoadedDependency {
            name,
            requirement_string: req_str.to_string(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requirement_simple() {
        let dep = DependencyLoader::parse_requirement("requests", DependencySource::Project);
        assert!(dep.is_some());
        let dep = dep.unwrap();
        assert_eq!(dep.name.as_str(), "requests");
        assert_eq!(dep.requirement_string, "requests");
    }

    #[test]
    fn test_parse_requirement_with_version() {
        let dep =
            DependencyLoader::parse_requirement("requests>=2.28.0", DependencySource::Project);
        assert!(dep.is_some());
        let dep = dep.unwrap();
        assert_eq!(dep.name.as_str(), "requests");
        assert_eq!(dep.requirement_string, "requests>=2.28.0");
    }

    #[test]
    fn test_parse_requirement_with_extras() {
        let dep = DependencyLoader::parse_requirement(
            "requests[security]>=2.28.0",
            DependencySource::Project,
        );
        assert!(dep.is_some());
        let dep = dep.unwrap();
        assert_eq!(dep.name.as_str(), "requests");
    }

    #[test]
    fn test_dependency_source_is_dev() {
        assert!(!DependencySource::Project.is_dev());
        assert!(DependencySource::ToolUvDevDependencies.is_dev());
        assert!(DependencySource::DependencyGroup(DEV_DEPENDENCIES.clone()).is_dev());
    }
}
