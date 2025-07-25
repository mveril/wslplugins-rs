use crate::sid::SidIdentifierAuthority;

use super::SidSizeInfo;
use wslpluginapi_sys::windows_sys::{
    core::PWSTR,
    Win32::{
        Foundation::LocalFree,
        Security::{
            Authorization::ConvertSidToStringSidW, CopySid, GetLengthSid,
            GetSidIdentifierAuthority, GetSidSubAuthority, GetSidSubAuthorityCount, PSID, SID,
        },
    },
};

use std::{
    alloc::Layout,
    fmt::{self, Debug, Display},
    hash::Hash,
    mem::MaybeUninit,
    os::raw::c_void,
    ptr, slice,
};

#[repr(C)]
#[derive(Debug)]
pub struct Sid {
    pub revision: u8,
    pub sub_authority_count: u8,
    pub identifier_authority: SidIdentifierAuthority,
    pub sub_authority: [u32],
}

#[repr(C)]
pub(super) struct SidHead {
    pub revision: u8,
    pub sub_authority_count: u8,
    pub identifier_authority: SidIdentifierAuthority,
}

pub(super) const SID_HEAD_SIZE: usize = std::mem::size_of::<SidHead>();
pub(super) const SID_HEAD_ALIGN: usize = std::mem::align_of::<SidHead>();

impl Sid {
    pub unsafe fn from_raw<'a>(raw: PSID) -> &'a Self {
        let ptr =
            unsafe { ptr::slice_from_raw_parts(raw as *const u8, GetLengthSid(raw) as usize) };
        &*(ptr as *const Self)
    }

    pub unsafe fn as_raw(&self) -> PSID {
        let slice = &*{ self as *const Self as *const [u8] };
        slice.as_ptr() as PSID
    }

    pub unsafe fn as_binary(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.as_raw() as *const u8,
                self.get_current_min_layoot().size(),
            )
        }
    }

    pub unsafe fn as_binary_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                self.as_raw() as *mut u8,
                self.get_current_min_layoot().size(),
            )
        }
    }

    pub fn get_sub_authorities(&self) -> &[u32] {
        unsafe {
            slice::from_raw_parts(
                self.sub_authority.as_ptr(),
                self.sub_authority_count as usize,
            )
        }
    }

    pub(super) fn get_current_min_layoot(&self) -> Layout {
        let size = unsafe { GetLengthSid(self.as_raw()) as usize };
        Layout::from_size_align(size, SID_HEAD_ALIGN).unwrap()
    }
}

impl Display for Sid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write the revision (should always be 1 in modern SIDs)
        write!(f, "S-{}", self.revision)?;

        // Identifier Authority: print as decimal if fits in u32, else as hex
        let mut be_bytes = [0u8; 8];
        be_bytes[2..].copy_from_slice(&self.identifier_authority.value.as_slice());
        let id_auth_value = u64::from_be_bytes(be_bytes);
        if id_auth_value <= 0xFFFFFFFF {
            write!(f, "-{}", id_auth_value)?;
        } else {
            write!(f, "-0x{:X}", id_auth_value)?;
        }

        // SubAuthorities
        for &sub_auth in self.get_sub_authorities() {
            write!(f, "-{}", sub_auth)?;
        }
        Ok(())
    }
}

impl PartialEq for Sid {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.as_binary() == other.as_binary() }
    }
}

impl Eq for Sid {}
impl Hash for Sid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { self.as_binary().hash(state) }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;
    use std::ops::Deref;

    use crate::sid::arb_security_identifier;

    use super::super::arb_identifier_authority;
    use super::*;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn sid_display_round_trip(sid in arb_security_identifier()) {
            let display = sid.deref().to_string();
            // On vérifie le prefixe standard
            prop_assert!(display.starts_with("S-1-"), "Display does not start with S-1-: {}", display);

            // Vérifie le nombre de sub_authorities dans le string
            let dash_count = display.matches('-').count();
            let expected = (sid.sub_authority_count as usize) + 2;
            prop_assert_eq!(dash_count, expected, "Dash count {} vs sub_authority_count {}", dash_count, expected);

            // On pourrait imaginer ici une parse pour un round-trip parfait
        }

        #[test]
        fn sid_hash_and_eq(sid1 in arb_security_identifier(), sid2 in arb_security_identifier()) {
            // Reflexivity
            prop_assert_eq!(sid1.deref(), sid1.deref());

            // Si binaire identique, Eq doit l'être aussi (même instance)
            let sid2_clone = sid1.clone();
            prop_assert_eq!(&sid1, &sid2_clone);
            use std::collections::hash_map::DefaultHasher;
            let mut hasher1 = DefaultHasher::new();
            sid1.hash(&mut hasher1);
            let mut hasher2 = DefaultHasher::new();
            sid2_clone.hash(&mut hasher2);
            prop_assert_eq!(hasher1.finish(), hasher2.finish());

            // S'ils sont différents, il est peu probable que le hash soit identique (collision rare)
            if sid1 != sid2 {
                let mut hasher2 = DefaultHasher::new();
                sid2.hash(&mut hasher2);
                prop_assert!(hasher1.finish() != hasher2.finish() || sid1 == sid2, "Hash collision with different sids");
            }
        }

        #[test]
        fn sid_sub_authorities_len(sid in arb_security_identifier()) {
            let subs = sid.get_sub_authorities();
            prop_assert_eq!(subs.len(), sid.sub_authority_count as usize);
        }
    }
    #[cfg(windows)]
    mod windows {
        use std::ops::Deref;

        use crate::sid::arb_security_identifier;

        use super::super::*;
        use proptest::prelude::*;
        use widestring::WideCStr;
        use wslpluginapi_sys::windows_sys::Win32::Foundation::GetLastError;
        use wslpluginapi_sys::windows_sys::Win32::Security::Authorization::*;
        use wslpluginapi_sys::windows_sys::Win32::Security::*;
        proptest! {
        #[test]
        fn test_display_same_as_windows(sid in arb_security_identifier()) {
            unsafe {
                // Get raw SID pointer
                let sid_ptr = sid.deref().as_raw();

                // Prepare output pointer for Windows API
                let mut pwstr = std::ptr::null_mut();

                // Convert SID to string using Windows API
                let ok = ConvertSidToStringSidW(sid_ptr, pwstr);
                let error_result = if ok == 0 {
                    Some(GetLastError())
                } else {
                    None
                };
                prop_assert_eq!(error_result, None, "There is an error on the ConvertSidToStringSidW call");

                // Construct a Rust string from the wide string
                let wide = WideCStr::from_ptr_str(pwstr as *const u16);
                let expected = wide.to_string_lossy();

                // Compare with your Display impl
                let display_str = sid.to_string();
                prop_assert_eq!(display_str, expected);

                // Free the allocated string from Windows
                LocalFree(pwstr as *mut c_void);
            }
        }

                }
    }
}
