use std::{
    any::Any,
    ffi::{OsStr, OsString},
};
pub use tracing::debug;
use wslpluginapi_sys::windows_sys::Win32::Foundation::E_FAIL;

#[doc(hidden)]
#[inline]
pub fn catch_unwind_or<T>(fallback: T, callback: impl FnOnce() -> T) -> T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)).unwrap_or_else(|_| {
        debug!("panic caught at WSL plugin FFI boundary");
        fallback
    })
}

fn downcast_os_message<T>(payload: &(dyn Any + Send)) -> Option<OsString>
where
    T: Any + AsRef<OsStr>,
{
    payload
        .downcast_ref::<T>()
        .map(|message| message.as_ref().to_os_string())
}

#[doc(hidden)]
#[inline]
pub fn catch_unwind_plugin<T>(
    callback: impl FnOnce() -> crate::plugin::Result<T>,
) -> crate::plugin::Result<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(callback)) {
        Ok(result) => result,
        Err(payload) => {
            let payload = payload.as_ref();
            let message: Option<OsString> = downcast_os_message::<String>(payload)
                .or_else(|| downcast_os_message::<&str>(payload))
                .or_else(|| downcast_os_message::<OsString>(payload))
                .or_else(|| downcast_os_message::<&OsStr>(payload));
            Err(crate::plugin::Error::new(
                windows_core::HRESULT(E_FAIL),
                message,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::{OsStr, OsString};
    use std::panic::panic_any;

    use wslpluginapi_sys::windows_sys::Win32::Foundation::E_FAIL;

    use super::{catch_unwind_or, catch_unwind_plugin};

    #[test]
    fn returns_callback_result() {
        assert_eq!(catch_unwind_or(1, || 2), 2);
    }

    #[test]
    #[allow(clippy::panic, reason = "Verifies panic handling at the FFI boundary")]
    fn returns_fallback_when_callback_panics() {
        assert_eq!(catch_unwind_or(1, || panic!("test panic")), 1);
    }

    #[test]
    #[allow(clippy::panic, reason = "Verifies panic handling at the FFI boundary")]
    fn converts_string_panic_payload_to_plugin_error() {
        let Err(error) = catch_unwind_plugin::<()>(|| panic!("test panic")) else {
            panic!("the panic should be converted to a plugin error");
        };
        assert_eq!(error.code(), windows_core::HRESULT(E_FAIL));
        assert_eq!(error.message(), Some(OsStr::new("test panic")));
    }

    #[test]
    #[allow(clippy::panic, reason = "Verifies panic handling at the FFI boundary")]
    fn converts_string_panic_str_payload_to_plugin_error() {
        let Err(error) = catch_unwind_plugin::<()>(|| panic!("test panic")) else {
            panic!("the panic should be converted to a plugin error");
        };
        assert_eq!(error.code(), windows_core::HRESULT(E_FAIL));
        assert_eq!(error.message(), Some(OsStr::new("test panic")));
    }

    #[test]
    #[allow(clippy::panic, reason = "Verifies panic handling at the FFI boundary")]
    fn converts_string_panic_string_payload_to_plugin_error() {
        let Err(error) = catch_unwind_plugin::<()>(|| panic!("{}", String::from("test panic")))
        else {
            panic!("the panic should be converted to a plugin error");
        };

        assert_eq!(error.code(), windows_core::HRESULT(E_FAIL));
        assert_eq!(error.message(), Some(OsStr::new("test panic")));
    }

    #[test]
    #[allow(clippy::panic, reason = "Verifies panic handling at the FFI boundary")]
    fn converts_string_panic_os_str_payload_to_plugin_error() {
        let Err(error) = catch_unwind_plugin::<()>(|| panic_any(OsStr::new("test panic"))) else {
            panic!("the panic should be converted to a plugin error");
        };

        assert_eq!(error.code(), windows_core::HRESULT(E_FAIL));
        assert_eq!(error.message(), Some(OsStr::new("test panic")));
    }

    #[test]
    #[allow(clippy::panic, reason = "Verifies panic handling at the FFI boundary")]
    fn converts_string_panic_os_string_payload_to_plugin_error() {
        let Err(error) = catch_unwind_plugin::<()>(|| panic_any(OsString::from("test panic")))
        else {
            panic!("the panic should be converted to a plugin error");
        };
        assert_eq!(error.code(), windows_core::HRESULT(E_FAIL));
        assert_eq!(error.message(), Some(OsStr::new("test panic")));
    }

    #[test]
    #[allow(clippy::panic, reason = "Verifies panic handling at the FFI boundary")]
    fn converts_string_panic_no_message_payload_to_plugin_error() {
        let Err(error) = catch_unwind_plugin::<()>(|| panic_any(42)) else {
            panic!("the panic should be converted to a plugin error");
        };

        assert_eq!(error.code(), windows_core::HRESULT(E_FAIL));
        assert_eq!(error.message(), None);
    }
}
