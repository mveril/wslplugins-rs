//! # WSL Version Checking Utilities
//!
//! This module provides utilities to verify that the current WSL version meets the required version for plugin compatibility.
//! It includes functions to perform version checks and unit tests to ensure correctness.

use super::errors::require_update_error::{Error, Result};
use crate::cstring_ext::CstringExt;
use crate::WSLContext;
use crate::WSLVersion;
use std::ffi::CString;
use std::ptr;
use typed_path::Utf8UnixPath;

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
    wsl_context: Option<&WSLContext>,
    required_version: &WSLVersion,
) -> Result<()> {
    wsl_context.map_or(Ok(()), |context| {
        let current_version = context.api.version();
        check_required_version_result(current_version, required_version)
    })
}
#[inline]
pub(super) fn encode_c_path(path: &Utf8UnixPath) -> Vec<u8> {
    CString::from_str_truncate(path.as_str()).into_bytes_with_nul()
}

#[allow(clippy::similar_names, reason = "naming is clear")]
pub(super) fn encode_c_argv<I>(args: I) -> (Vec<CString>, Vec<*const u8>)
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let iter = args.into_iter();
    let (lower, upper) = iter.size_hint();
    let count = upper.unwrap_or(lower);

    let mut c_args = Vec::<CString>::with_capacity(count);
    let mut argv = Vec::<*const u8>::with_capacity(count + 1);

    for arg in iter {
        let c = CString::from_str_truncate(arg.as_ref());
        // Pointer is stable: moving CString does not move its internal buffer.
        argv.push(c.as_ptr().cast::<u8>());
        c_args.push(c);
    }

    // NULL-terminated list as required by the API contract.
    argv.push(ptr::null());

    (c_args, argv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WSLVersion;

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

    #[test]
    fn encode_c_path_appends_nul_terminator() {
        let encoded = encode_c_path("/bin/sh".as_ref());

        assert_eq!(encoded, b"/bin/sh\0");
    }

    #[test]
    fn encode_c_path_truncates_at_interior_nul() {
        let encoded = encode_c_path("/bin\0/sh".as_ref());

        assert_eq!(encoded, b"/bin\0");
    }

    #[test]
    fn encode_c_argv_truncates_args_and_null_terminates_argv() {
        let (c_args, argv) = encode_c_argv(["sh", "arg\0ignored"]);
        let encoded_args: Vec<_> = c_args
            .iter()
            .map(|arg| arg.as_c_str().to_bytes_with_nul())
            .collect();

        assert_eq!(encoded_args, [b"sh\0".as_slice(), b"arg\0".as_slice()]);
        assert_eq!(argv.len(), 3);
        assert_eq!(argv.iter().take_while(|ptr| !ptr.is_null()).count(), 2);
        assert!(argv.last().is_some_and(|ptr| ptr.is_null()));
    }
}
