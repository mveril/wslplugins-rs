//! # WSL Version Checking Utilities
//!
//! This module provides utilities to verify that the current WSL version meets the required version for plugin compatibility.
//! It includes functions to perform version checks and unit tests to ensure correctness.

use super::errors::require_update_error::{Error, Result};
use crate::WSLContext;
use wslplugins_sys::WSLVersion;

pub(crate) fn check_required_version_result(
    current_version: &WSLVersion,
    required_version: &WSLVersion,
) -> Result<()> {
    if current_version >= required_version {
        Ok(())
    } else {
        Err(Error {
            current_version: *current_version,
            required_version: *required_version,
        })
    }
}

pub(crate) fn check_required_version_result_from_context(
    wsl_context: &WSLContext,
    required_version: &WSLVersion,
) -> Result<()> {
    let current_version = wsl_context.api.version();
    check_required_version_result(current_version, required_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wslplugins_sys::WSLVersion;

    /// Tests that `check_required_version_result` returns `Ok` when the current version meets the requirement.
    #[test]
    fn test_check_required_version_result_ok() {
        let current_version = WSLVersion::new(2, 1, 3);
        let required_version = WSLVersion::new(2, 1, 2);

        let result = check_required_version_result(&current_version, &required_version);

        assert!(result.is_ok());
    }

    /// Tests that `check_required_version_result` returns `Err` when the current version is insufficient.
    #[test]
    fn test_check_required_version_result_error() {
        let current_version = WSLVersion::new(2, 1, 1);
        let required_version = WSLVersion::new(2, 1, 2);

        let result = check_required_version_result(&current_version, &required_version);

        assert!(result.is_err());
    }
}
