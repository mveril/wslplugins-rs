use super::WSLCSessionID;
use std::ffi::OsString;
use std::fmt;
use std::os::windows::ffi::OsStringExt as _;
use std::os::windows::io::BorrowedHandle;
use widestring::U16CStr;
use win_security_identifier::Sid;

/// Information about a WSL container session passed to plugin hooks.
#[repr(transparent)]
pub struct WSLCSessionInformation(wslpluginapi_sys::WSLCSessionInformation);

impl WSLCSessionInformation {
    /// Returns the WSLC session identifier.
    #[must_use]
    #[inline]
    pub const fn id(&self) -> WSLCSessionID {
        WSLCSessionID::new(self.0.SessionId)
    }

    /// Returns the display name as a borrowed UTF-16 string.
    #[must_use]
    #[inline]
    pub fn display_name(&self) -> &U16CStr {
        // SAFETY: WSLC guarantees a live NUL-terminated string during the hook.
        unsafe { U16CStr::from_ptr_str(self.0.DisplayName) }
    }

    /// Returns an owned copy of the display name.
    #[must_use]
    #[inline]
    pub fn display_name_os_string(&self) -> OsString {
        OsString::from_wide(self.display_name().as_slice())
    }

    /// Returns the PID of the application that created the session.
    #[must_use]
    #[inline]
    pub const fn application_pid(&self) -> u32 {
        self.0.ApplicationPid
    }

    /// Returns the user token borrowed for the duration of the hook.
    #[must_use]
    #[inline]
    pub const fn user_token(&self) -> BorrowedHandle<'_> {
        // SAFETY: WSLC guarantees that this handle remains valid during the hook.
        unsafe { BorrowedHandle::borrow_raw(self.0.UserToken) }
    }

    /// Returns the session user's SID.
    #[must_use]
    #[inline]
    pub const fn user_sid(&self) -> &Sid {
        // SAFETY: WSLC guarantees a valid SID pointer during the hook.
        unsafe { Sid::from_raw(self.0.UserSid) }
    }
}

impl AsRef<WSLCSessionInformation> for wslpluginapi_sys::WSLCSessionInformation {
    #[inline]
    fn as_ref(&self) -> &WSLCSessionInformation {
        // SAFETY: `WSLCSessionInformation` is transparent over the native type.
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

impl fmt::Debug for WSLCSessionInformation {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WSLCSessionInformation")
            .field("id", &self.id())
            .field("display_name", &self.display_name())
            .field("application_pid", &self.application_pid())
            .field("user_token", &self.0.UserToken)
            .field("user_sid", &self.user_sid())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::WSLCSessionInformation;
    use crate::utils::test_transparence;
    use std::ffi::OsString;
    use std::ptr;
    use widestring::u16cstr;

    #[test]
    fn layouts_match() {
        test_transparence::<wslpluginapi_sys::WSLCSessionInformation, WSLCSessionInformation>();
    }

    #[test]
    fn display_name_borrows_native_string() {
        let display_name = u16cstr!("container");
        let native = wslpluginapi_sys::WSLCSessionInformation {
            SessionId: 7,
            DisplayName: display_name.as_ptr(),
            ApplicationPid: 0,
            UserToken: ptr::null_mut(),
            UserSid: ptr::null_mut(),
        };
        let session: &WSLCSessionInformation = native.as_ref();

        assert!(ptr::eq(
            session.display_name().as_ptr(),
            display_name.as_ptr()
        ));
        assert_eq!(
            session.display_name_os_string(),
            OsString::from("container")
        );
    }
}
