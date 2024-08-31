use std::ffi::{CString, OsStr};
use std::os::windows::ffi::OsStrExt;
use wslplugins_sys::WSLPluginAPIV1;
use crate::{ApiV1, WSLPluginV1};

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

pub fn create_plugin_with_required_version<'a, T: WSLPluginV1<'a>>(
    api: &'a WSLPluginAPIV1,
    required_major: u32,
    required_minor: u32,
    required_revision: u32,
) -> windows::core::Result<T> {
    unsafe {
        wslplugins_sys::require_version(required_major, required_minor, required_revision, api)
            .ok()?;
    }
    let plugin = T::try_new(ApiV1::from(api))?;
    Ok(plugin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    #[test]
    fn test_encode_wide_null_terminated_no_nulls() {
        let input = OsString::from("Hello");
        let expected: Vec<u16> = "Hello\0".encode_utf16().collect();
        assert_eq!(encode_wide_null_terminated(&input), expected);
    }

    #[test]
    fn test_encode_wide_null_terminated_with_nulls() {
        let input = OsString::from("Hel\0lo");
        let expected: Vec<u16> = "Hello\0".encode_utf16().collect();
        assert_eq!(encode_wide_null_terminated(&input), expected);
    }

    #[test]
    fn test_cstring_from_str_no_nulls() {
        let input = "Hello";
        let cstring = cstring_from_str(input);
        assert_eq!(cstring.to_str().unwrap(), input);
    }

    #[test]
    fn test_cstring_from_str_with_nulls() {
        let input = "Hel\0lo";
        let cstring = cstring_from_str(input);
        let expected = "Hello".as_bytes();
        assert_eq!(cstring.into_bytes(), expected);
    }
}
