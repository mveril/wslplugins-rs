#[cfg(test)]
use std::mem::{align_of, size_of};
use widestring::U16CStr;
#[cfg(test)]
pub fn test_transparence<T, U>() {
    assert_eq!(align_of::<T>(), align_of::<U>());
    assert_eq!(size_of::<T>(), size_of::<U>());
}

/// Borrows a null-terminated UTF-16 string from a raw pointer.
///
/// # Safety
///
/// `ptr` must point to a valid null-terminated UTF-16 string that remains alive for `'a`.
pub unsafe fn wide_str<'a>(ptr: *const u16) -> &'a U16CStr {
    // SAFETY: The caller guarantees that `ptr` points to a valid null-terminated UTF-16 string
    // for the returned lifetime.
    unsafe { U16CStr::from_ptr_str(ptr) }
}

/// Borrows a non-empty null-terminated UTF-16 string from a nullable raw pointer.
///
/// # Safety
///
/// A non-null `ptr` must point to a valid null-terminated UTF-16 string that remains alive for
/// `'a`.
pub unsafe fn opt_wide_str<'a>(ptr: *const u16) -> Option<&'a U16CStr> {
    if ptr.is_null() {
        None
    } else {
        // SAFETY: The caller guarantees that `ptr` points to a valid null-terminated UTF-16
        // string for the returned lifetime.
        let wide = unsafe { wide_str(ptr) };
        if wide.is_empty() {
            None
        } else {
            Some(wide)
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
        // SAFETY: Null pointers are explicitly supported by `opt_wide_str`.
        assert_eq!(unsafe { opt_wide_str(ptr) }, None);
    }

    #[test]
    fn test_empty_as_wide_empty() {
        let wide_str = [0u16; 1];
        // SAFETY: `wide_str` is a valid null-terminated UTF-16 string for this scope.
        assert_eq!(unsafe { opt_wide_str(wide_str.as_ptr()) }, None);
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

            // SAFETY: The strategy produces a null-terminated UTF-16 buffer that remains alive
            // for the duration of the returned borrow.
            let result = unsafe { opt_wide_str(ptr) };
            #[allow(clippy::indexing_slicing, reason="Safe because we know the last element is the null terminator")]
            let expected = &wide[..wide.len() - 1];

            prop_assert_eq!(result.map(U16CStr::as_slice), Some(expected));
        }
    }
}
