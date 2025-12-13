//! PyPI package information loader.
//!
//! This module provides functionality for fetching package metadata
//! from the PyPI JSON API.

use std::collections::HashMap;

use serde::Deserialize;
use thiserror::Error;

use crate::state::Package;

/// Base URL for PyPI JSON API.
const PYPI_JSON_API_BASE: &str = "https://pypi.org/pypi";

/// Error type for PyPI package search operations.
#[derive(Debug, Error)]
pub enum PyPiSearchError {
    /// The requested package was not found on PyPI.
    #[error("Package not found: `{0}`")]
    NotFound(String),

    /// Network or HTTP error occurred.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// The package name is invalid.
    #[error("Invalid package name: `{0}`")]
    InvalidName(String),
}

/// Full response from PyPI JSON API for a package.
#[derive(Debug, Deserialize)]
pub struct PyPiPackageResponse {
    /// Package metadata.
    pub info: PyPiPackageInfo,
    /// All available releases keyed by version.
    #[serde(default)]
    pub releases: HashMap<String, Vec<PyPiReleaseFile>>,
}

/// Package metadata from PyPI.
#[derive(Debug, Clone, Deserialize)]
pub struct PyPiPackageInfo {
    /// The package name (normalized).
    pub name: String,
    /// The latest version.
    pub version: String,
    /// Short description (summary).
    #[serde(default)]
    pub summary: String,
    /// Full description (may be markdown or rst).
    pub description: Option<String>,
    /// Package author.
    pub author: Option<String>,
    /// Author email.
    pub author_email: Option<String>,
    /// Package license.
    pub license: Option<String>,
    /// Homepage URL.
    pub home_page: Option<String>,
    /// Project URLs (Source, Documentation, etc.).
    #[serde(default)]
    pub project_urls: Option<HashMap<String, String>>,
    /// Keywords for the package.
    #[serde(default)]
    pub keywords: Option<String>,
    /// Python version requirement.
    pub requires_python: Option<String>,
    /// Package dependencies.
    #[serde(default)]
    pub requires_dist: Option<Vec<String>>,
}

/// Information about a release file.
#[derive(Debug, Clone, Deserialize)]
pub struct PyPiReleaseFile {
    /// The filename.
    pub filename: String,
    /// File size in bytes.
    pub size: Option<u64>,
    /// Upload timestamp.
    pub upload_time: Option<String>,
}

/// Loader for fetching package information from PyPI.
pub struct PyPiPackageLoader {
    client: reqwest::blocking::Client,
}

impl PyPiPackageLoader {
    /// Create a new PyPI package loader.
    ///
    /// Returns `None` if the HTTP client fails to build (e.g., TLS initialization failure).
    pub fn new() -> Option<Self> {
        let client = reqwest::blocking::Client::builder()
            .user_agent(format!("uv-gui/{}", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;

        Some(Self { client })
    }

    /// Look up a package by exact name on PyPI.
    ///
    /// This uses the PyPI JSON API endpoint: `GET /pypi/{package}/json`
    pub fn lookup(&self, package_name: &str) -> Result<PyPiPackageResponse, PyPiSearchError> {
        let name = package_name.trim();
        if name.is_empty() {
            return Err(PyPiSearchError::InvalidName(name.to_string()));
        }

        // Basic validation: package names can contain alphanumerics, hyphens, underscores, dots
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(PyPiSearchError::InvalidName(name.to_string()));
        }

        let url = format!("{PYPI_JSON_API_BASE}/{name}/json");
        let response = self.client.get(&url).send()?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PyPiSearchError::NotFound(name.to_string()));
        }

        let package: PyPiPackageResponse = response.error_for_status()?.json()?;
        Ok(package)
    }
}

impl PyPiPackageInfo {
    /// Convert PyPI package info to our internal Package representation.
    pub fn into_package(self) -> Package {
        // Extract repository URL from project_urls if available
        let repository = self.project_urls.as_ref().and_then(|urls| {
            urls.get("Source")
                .or_else(|| urls.get("Repository"))
                .or_else(|| urls.get("GitHub"))
                .cloned()
        });

        // Extract documentation URL
        let documentation = self
            .project_urls
            .as_ref()
            .and_then(|urls| urls.get("Documentation").cloned());

        // Parse keywords if available
        let keywords = self
            .keywords
            .map(|k| {
                k.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        // Parse dependencies into just the package names
        let dependencies = self
            .requires_dist
            .map(|deps| {
                deps.iter()
                    .filter_map(|dep| {
                        // Extract just the package name from requirement strings like "requests>=2.0"
                        dep.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                            .next()
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        Package {
            name: self.name,
            latest_version: Some(self.version),
            description: if self.summary.is_empty() {
                None
            } else {
                Some(self.summary)
            },
            homepage: self.home_page,
            documentation,
            repository,
            license: self.license,
            keywords,
            dependencies,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_package_name_empty() {
        let loader = PyPiPackageLoader::new().expect("Failed to create loader");
        let result = loader.lookup("");
        assert!(matches!(result, Err(PyPiSearchError::InvalidName(_))));
    }

    #[test]
    fn test_invalid_package_name_special_chars() {
        let loader = PyPiPackageLoader::new().expect("Failed to create loader");
        let result = loader.lookup("package@evil");
        assert!(matches!(result, Err(PyPiSearchError::InvalidName(_))));
    }

    #[test]
    fn test_pypi_info_into_package() {
        let info = PyPiPackageInfo {
            name: "requests".to_string(),
            version: "2.31.0".to_string(),
            summary: "Python HTTP for Humans".to_string(),
            description: None,
            author: Some("Kenneth Reitz".to_string()),
            author_email: None,
            license: Some("Apache-2.0".to_string()),
            home_page: Some("https://requests.readthedocs.io".to_string()),
            project_urls: Some(HashMap::from([
                (
                    "Source".to_string(),
                    "https://github.com/psf/requests".to_string(),
                ),
                (
                    "Documentation".to_string(),
                    "https://requests.readthedocs.io".to_string(),
                ),
            ])),
            keywords: Some("http,client,web".to_string()),
            requires_python: None,
            requires_dist: Some(vec![
                "charset-normalizer>=2".to_string(),
                "idna>=2.5".to_string(),
            ]),
        };

        let package = info.into_package();

        assert_eq!(package.name, "requests");
        assert_eq!(package.latest_version, Some("2.31.0".to_string()));
        assert_eq!(
            package.description,
            Some("Python HTTP for Humans".to_string())
        );
        assert_eq!(package.license, Some("Apache-2.0".to_string()));
        assert_eq!(
            package.repository,
            Some("https://github.com/psf/requests".to_string())
        );
        assert_eq!(package.keywords, vec!["http", "client", "web"]);
        assert_eq!(package.dependencies.len(), 2);
    }
}
