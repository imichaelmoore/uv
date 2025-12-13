//! Data loaders for fetching project information.
//!
//! This module provides abstractions for loading project data from
//! the filesystem, including pyproject.toml and uv.lock files,
//! as well as fetching package information from PyPI.

mod dependency_loader;
mod lockfile_loader;
mod project_loader;
mod pypi_loader;

pub use dependency_loader::{
    DependencyLoadError, DependencyLoader, DependencySource, LoadedDependency,
};
pub use lockfile_loader::{LockedPackage, LockfileLoadError, LockfileLoader};
pub use project_loader::{LoadedProject, ProjectLoadError, ProjectLoader};
pub use pypi_loader::{PyPiPackageInfo, PyPiPackageLoader, PyPiPackageResponse, PyPiSearchError};
