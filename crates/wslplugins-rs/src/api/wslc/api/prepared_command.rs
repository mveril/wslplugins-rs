use super::{
    WSLCApi, WSLCCommandExecution, WSLCProcess, WSLCProcessStreams, WSLCSessionID, WSLCSpawnError,
};
use crate::api::utils;
use crate::cstring_ext::CstringExt as _;
use std::ffi::CString;
use std::ptr;
use typed_path::Utf8UnixPath;

#[cfg(feature = "smallvec")]
type CStringVec = smallvec::SmallVec<[CString; 10]>;
#[cfg(not(feature = "smallvec"))]
type CStringVec = Vec<CString>;

#[cfg(feature = "smallvec")]
type PointerVec = smallvec::SmallVec<[*const u8; 10]>;
#[cfg(not(feature = "smallvec"))]
type PointerVec = Vec<*const u8>;

#[derive(Debug)]
struct EncodedStringArray {
    #[allow(dead_code, reason = "owns the C strings backing the pointer array")]
    strings: CStringVec,
    pointers: PointerVec,
}

impl EncodedStringArray {
    fn new<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::from_c_strings(
            values
                .into_iter()
                .map(|value| CString::from_str_truncate(value.as_ref())),
        )
    }

    fn environment<'a>(values: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        Self::from_c_strings(values.into_iter().map(|(key, value)| {
            let mut entry = String::with_capacity(key.len() + value.len() + 1);
            entry.push_str(key);
            entry.push('=');
            entry.push_str(value);
            CString::from_str_truncate(&entry)
        }))
    }

    fn from_c_strings(values: impl IntoIterator<Item = CString>) -> Self {
        let values = values.into_iter();
        let (lower, upper) = values.size_hint();
        let mut strings = CStringVec::with_capacity(upper.unwrap_or(lower));
        strings.extend(values);

        let mut pointers = PointerVec::with_capacity(strings.len() + 1);
        pointers.extend(strings.iter().map(|value| value.as_ptr().cast()));
        pointers.push(ptr::null());

        Self { strings, pointers }
    }

    #[inline]
    fn as_slice(&self) -> &[*const u8] {
        &self.pointers
    }
}

/// A WSLC command whose native strings and pointer arrays are encoded once.
#[derive(Debug)]
pub struct PreparedWSLCCommand<'api> {
    api: WSLCApi<'api>,
    session_id: WSLCSessionID,
    executable: Box<[u8]>,
    arguments: EncodedStringArray,
    environment: EncodedStringArray,
    streams: WSLCProcessStreams,
}

impl<'api> PreparedWSLCCommand<'api> {
    pub(super) fn new<'a>(
        api: WSLCApi<'api>,
        session_id: WSLCSessionID,
        executable: &Utf8UnixPath,
        arguments: impl IntoIterator<Item = &'a str>,
        environment: impl IntoIterator<Item = (&'a str, &'a str)>,
        streams: WSLCProcessStreams,
    ) -> Self {
        Self {
            api,
            session_id,
            executable: utils::encode_c_path(executable).into_boxed_slice(),
            arguments: EncodedStringArray::new(arguments),
            environment: EncodedStringArray::environment(environment),
            streams,
        }
    }

    pub(super) fn spawn_for_output(
        &self,
    ) -> std::result::Result<WSLCProcess<'api>, WSLCSpawnError> {
        self.spawn_with_streams(
            self.streams
                .union(WSLCProcessStreams::STDOUT)
                .union(WSLCProcessStreams::STDERR),
        )
    }

    fn spawn_with_streams(
        &self,
        streams: WSLCProcessStreams,
    ) -> std::result::Result<WSLCProcess<'api>, WSLCSpawnError> {
        self.api.create_process_encoded(
            self.session_id,
            &self.executable,
            self.arguments.as_slice(),
            self.environment.as_slice(),
            streams,
        )
    }
}

impl<'api> WSLCCommandExecution<'api> for PreparedWSLCCommand<'api> {
    #[inline]
    fn spawn(&self) -> std::result::Result<WSLCProcess<'api>, WSLCSpawnError> {
        self.spawn_with_streams(self.streams)
    }

    #[inline]
    fn output(&self) -> std::result::Result<super::WSLCOutput, super::WSLCCommandError> {
        self.spawn_for_output()?.wait_with_output()
    }
}

#[cfg(all(test, feature = "smallvec"))]
mod tests {
    use super::EncodedStringArray;

    #[test]
    fn common_string_arrays_remain_inline() {
        let encoded = EncodedStringArray::new(["command", "one", "two"]);

        assert!(!encoded.strings.spilled());
        assert!(!encoded.pointers.spilled());
    }

    #[test]
    fn large_string_arrays_spill_without_a_size_limit() {
        let values = (0..32).map(|index| index.to_string()).collect::<Vec<_>>();
        let encoded = EncodedStringArray::new(&values);

        assert!(encoded.strings.spilled());
        assert!(encoded.pointers.spilled());
        assert_eq!(encoded.as_slice().len(), values.len() + 1);
    }
}
