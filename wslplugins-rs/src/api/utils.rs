use crate::WSLContext;

use super::errors::require_update_error::{Error, Result};
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

    #[test]
    fn test_check_required_version_result_ok() {
        let current_version = WSLVersion::new(2, 1, 3);
        let required_version = WSLVersion::new(2, 1, 2);

        let result = check_required_version_result(&current_version, &required_version);

        assert!(result.is_ok());
    }

    #[test]
    fn test_check_required_version_result_error() {
        let current_version = WSLVersion::new(2, 1, 1);
        let required_version = WSLVersion::new(2, 1, 2);

        let result = check_required_version_result(&current_version, &required_version);

        assert!(result.is_err());
    }
}
