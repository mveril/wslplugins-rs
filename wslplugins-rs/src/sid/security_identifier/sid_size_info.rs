use crate::sid::{SID_HEAD_ALIGN, SID_HEAD_SIZE};
use std::alloc::Layout;
use std::mem::size_of;

pub(crate) struct SidSizeInfo {
    pub sub_authority_count: u8,
}

impl SidSizeInfo {
    pub fn from_full_size(size: usize) -> Self {
        assert!(
            size >= SID_HEAD_SIZE,
            "Size must be greater than or equal to SID_HEAD_SIZE"
        );
        assert!(
            (size - SID_HEAD_SIZE) % size_of::<u32>() == 0,
            "Size does not match SID layout"
        );
        Self {
            sub_authority_count: ((size - SID_HEAD_SIZE) / size_of::<u32>()) as u8,
        }
    }

    pub fn get_full_size(&self) -> usize {
        self.get_layout().size()
    }

    pub fn get_layout(&self) -> Layout {
        // Usage de from_size_align (safe) plutôt que from_size_align_unchecked
        let head_layout = Layout::from_size_align(SID_HEAD_SIZE, SID_HEAD_ALIGN).unwrap();
        let dinamic_layout = Layout::array::<u32>(self.sub_authority_count as usize).unwrap();
        head_layout.extend(dinamic_layout).unwrap().0.pad_to_align()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::prelude::*;
    use std::alloc::Layout;
    use std::mem::size_of;
    use wslpluginapi_sys::windows_sys::Win32::Security::{GetSidLengthRequired, SID};

    #[test]
    fn test_layout_matches_windows_sid() {
        // Par convention, un SID Windows "classique" a 1 sub-authority.
        let info = SidSizeInfo {
            sub_authority_count: 1,
        };
        assert_eq!(Layout::new::<SID>(), info.get_layout());
    }

    proptest! {
        #[test]
        fn prop_full_size_and_from_full_size(sub_authority_count in 0u8..16) {
            let info = SidSizeInfo { sub_authority_count };
            let size = info.get_full_size();
            let reconstructed = SidSizeInfo::from_full_size(size);
            prop_assert_eq!(info.sub_authority_count, reconstructed.sub_authority_count);
        }

        #[test]
        fn prop_layout_properties(sub_authority_count in 0u8..16) {
            let info = SidSizeInfo { sub_authority_count };
            let layout = info.get_layout();
            let expected_align = align_of::<u32>();
            prop_assert_eq!(layout.size(), info.get_full_size());
            prop_assert_eq!(layout.align(), expected_align);
        }

        #[test]
        fn prop_full_size_always_multiple_of_u32(sub_authority_count in 0u8..16) {
            let info = SidSizeInfo { sub_authority_count };
            let size = info.get_full_size();
            prop_assert!((size - SID_HEAD_SIZE) % size_of::<u32>() == 0);
        }

        #[test]
        fn prop_full_size_compare_windows(sub_authority_count in 0u8..16) {
            let info = SidSizeInfo { sub_authority_count };
            let size = info.get_full_size();
            let winsize = unsafe {
              GetSidLengthRequired(sub_authority_count)
            } as usize;
            prop_assert!(size == winsize );
        }
    }
}
