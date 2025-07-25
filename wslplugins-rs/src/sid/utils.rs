use crate::sid::Sid;
use std::{os::raw::c_void, slice};

pub(super) unsafe fn build_sid_ptr(src: *mut c_void, dynamic_size_count: usize) -> *mut Sid {
    unsafe { slice::from_raw_parts_mut(src, dynamic_size_count) as *mut [_] as *mut Sid }
}
