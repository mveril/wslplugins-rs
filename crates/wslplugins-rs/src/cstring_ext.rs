use std::ffi::CString;

pub trait CstringExt {
    /// Creates a `CString` from a string slice, truncating at the first null byte if present.
    fn from_str_truncate(value: &str) -> Self;
}

impl CstringExt for CString {
    #[expect(clippy::indexing_slicing, reason = "The slice is controlled")]
    fn from_str_truncate(value: &str) -> Self {
        let bytes = value.as_bytes();
        let truncated_bytes = bytes
            .iter()
            .position(|&b| b == 0)
            .map_or(bytes, |pos| &bytes[..pos]);
        // SAFETY: `truncated_bytes` is guaranteed not to contain null bytes.
        unsafe { Self::from_vec_unchecked(truncated_bytes.to_vec()) }
    }
}
#[allow(clippy::expect_used, clippy::unwrap_used, reason = "Tests")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_from_str_truncate_no_null() {
        let input = "Hello, world!";
        let cstring = CString::from_str_truncate(input);
        assert_eq!(cstring.to_str().unwrap(), input);
    }

    #[test]
    fn test_from_str_truncate_with_null() {
        let input = "Hello\0world!";
        let cstring = CString::from_str_truncate(input);
        assert_eq!(cstring.to_str().unwrap(), "Hello");
    }

    #[test]
    fn test_from_str_truncate_empty() {
        let input = "";
        let cstring = CString::from_str_truncate(input);
        assert_eq!(cstring.to_str().unwrap(), "");
    }

    #[test]
    fn test_from_str_truncate_null_only() {
        let input = "\0";
        let cstring = CString::from_str_truncate(input);
        assert_eq!(cstring.to_str().unwrap(), "");
    }

    #[test]
    fn test_from_str_truncate_null_in_middle() {
        let input = "Rust\0is awesome!";
        let cstring = CString::from_str_truncate(input);
        assert_eq!(cstring.to_str().unwrap(), "Rust");
    }
}
