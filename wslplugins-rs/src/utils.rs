//! # String Encoding Utilities
//!
//! This module provides utility functions to handle string encoding conversions, specifically for:
//! - Encoding `OsStr` as wide, null-terminated UTF-16 strings.
//! - Creating `CString` instances from Rust strings, filtering out null bytes.

use std::ffi::{CString, OsStr};
use std::os::windows::ffi::OsStrExt;

pub fn encode_wide_null_terminated(input: &OsStr) -> Vec<u16> {
    input
        .encode_wide()
        .filter(|&c| c != 0)
        .chain(Some(0))
        .collect()
}

pub fn cstring_from_str(input: &str) -> CString {
    let filtered_input: Vec<u8> = input.bytes().filter(|&c| c != 0).collect();
    unsafe { CString::from_vec_unchecked(filtered_input) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    /// Tests `encode_wide_null_terminated` with a string containing no null characters.
    #[test]
    fn test_encode_wide_null_terminated_no_nulls() {
        let input = OsString::from("Hello");
        let expected: Vec<u16> = "Hello\0".encode_utf16().collect();
        assert_eq!(encode_wide_null_terminated(&input), expected);
    }

    /// Tests `encode_wide_null_terminated` with a string containing null characters.
    #[test]
    fn test_encode_wide_null_terminated_with_nulls() {
        let input = OsString::from("Hel\0lo");
        let expected: Vec<u16> = "Hello\0".encode_utf16().collect();
        assert_eq!(encode_wide_null_terminated(&input), expected);
    }

    /// Tests `cstring_from_str` with a string containing no null characters.
    #[test]
    fn test_cstring_from_str_no_nulls() {
        let input = "Hello";
        let cstring = cstring_from_str(input);
        assert_eq!(cstring.to_str().unwrap(), input);
    }

    /// Tests `cstring_from_str` with a string containing null characters.
    #[test]
    fn test_cstring_from_str_with_nulls() {
        let input = "Hel\0lo";
        let cstring = cstring_from_str(input);
        let expected = "Hello".as_bytes();
        assert_eq!(cstring.into_bytes(), expected);
    }
}

#[cfg(test)]
pub(crate) fn test_transparence<T, U>() {
    assert_eq!(align_of::<T>(), align_of::<U>());
    assert_eq!(size_of::<T>(), size_of::<U>());
}
