//! Lockfile loading for installed version information.
//!
//! This module handles parsing uv.lock files to extract
//! installed package versions.

use std::path::Path;

use uv_normalize::PackageName;
use uv_pep440::Version;
use uv_resolver::Lock;

use thiserror::Error;

/// A package from the lockfile with version info.
#[derive(Clone, Debug)]
pub struct LockedPackage {
    /// The package name.
    pub name: PackageName,
    /// The locked version.
    pub version: Version,
}

/// Error type for lockfile loading.
#[derive(Debug, Error)]
pub enum LockfileLoadError {
    /// Failed to read the file.
    #[error("Failed to read lockfile: {0}")]
    ReadError(#[from] std::io::Error),
    /// Failed to parse the TOML.
    #[error("Failed to parse lockfile: {0}")]
    ParseError(#[from] toml::de::Error),
    /// Unsupported lockfile version.
    #[error("Unsupported lockfile version: {0} (expected {1})")]
    UnsupportedVersion(u32, u32),
}

/// The expected lockfile version.
const EXPECTED_VERSION: u32 = 1;

/// Loads package versions from uv.lock.
pub struct LockfileLoader;

impl LockfileLoader {
    /// Load the lockfile and extract package versions.
    pub fn load(lock_path: &Path) -> Result<Vec<LockedPackage>, LockfileLoadError> {
        let content = fs_err::read_to_string(lock_path)?;
        let lock: Lock = toml::from_str(&content)?;

        // Check version compatibility
        if lock.version() != EXPECTED_VERSION {
            return Err(LockfileLoadError::UnsupportedVersion(
                lock.version(),
                EXPECTED_VERSION,
            ));
        }

        Ok(lock
            .packages()
            .iter()
            .filter_map(|pkg| {
                // Only include packages with versions (skip dynamic source trees)
                pkg.version().map(|version| LockedPackage {
                    name: pkg.name().clone(),
                    version: version.clone(),
                })
            })
            .collect())
    }

    /// Try to load, returning None if file doesn't exist.
    pub fn try_load(lock_path: &Path) -> Result<Option<Vec<LockedPackage>>, LockfileLoadError> {
        if !lock_path.exists() {
            return Ok(None);
        }
        Self::load(lock_path).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_try_load_nonexistent() {
        let result = LockfileLoader::try_load(Path::new("/nonexistent/uv.lock"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_load_minimal_lockfile() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
version = 1
requires-python = ">=3.8"

[[package]]
name = "requests"
version = "2.31.0"
source = {{ registry = "https://pypi.org/simple" }}
"#
        )
        .unwrap();

        let result = LockfileLoader::load(file.path());
        assert!(result.is_ok());
        let packages = result.unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name.as_str(), "requests");
        assert_eq!(packages[0].version.to_string(), "2.31.0");
    }
}
