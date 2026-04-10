use std::ffi::OsString;
#[cfg(test)]
use std::mem::{align_of, size_of};
use std::os::windows::ffi::OsStringExt as _;
use windows_core::PCWSTR;
#[cfg(test)]
pub fn test_transparence<T, U>() {
    assert_eq!(align_of::<T>(), align_of::<U>());
    assert_eq!(size_of::<T>(), size_of::<U>());
}

pub fn opt_wide_str(ptr: *const u16) -> Option<OsString> {
    if ptr.is_null() {
        None
    } else {
        let wide_str = PCWSTR::from_raw(ptr);
        // SAFETY: The caller guarantees that `ptr` is valid and points to a null-terminated wide string.
        unsafe {
            let wide = wide_str.as_wide();
            if wide.is_empty() {
                None
            } else {
                Some(OsString::from_wide(wide))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    #[test]
    fn test_empty_as_wide_null() {
        let ptr = std::ptr::null();
        assert_eq!(opt_wide_str(ptr), None);
    }

    #[test]
    fn test_empty_as_wide_empty() {
        let wide_str = [0u16; 1];
        assert_eq!(opt_wide_str(wide_str.as_ptr()), None);
    }

    fn null_terminated_wide() -> impl Strategy<Value = Vec<u16>> {
        let unit = prop_oneof![(1u16..=0xD7FF), (0xE000u16..=0xFFFF),];

        proptest::collection::vec(unit, 0..100).prop_map(|wide| {
            let mut wide: Vec<u16> = wide;
            wide.push(0);
            wide
        })
    }

    proptest! {
        #[test]
        fn test_opt_wide_str_non_empty(wide in null_terminated_wide()) {
            prop_assume!(wide.len() > 1);

            let ptr = wide.as_ptr();

            let result = opt_wide_str(ptr);
            #[allow(clippy::indexing_slicing, reason="Safe because we know the last element is the null terminator")]
            let expected = OsString::from_wide(&wide[..wide.len() - 1]);

            prop_assert_eq!(result, Some(expected));
        }
    }
}
