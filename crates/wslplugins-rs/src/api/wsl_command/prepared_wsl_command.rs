use super::wsl_command_execution::WSLCommandExecution;
use super::WSLCommand;
use std::ffi::CString;
use std::net::TcpStream;

use crate::{
    api::{utils, ApiV1, Result as ApiResult},
    DistributionID, SessionID,
};

/// A command pre-encoded for the WSL Plugin API.
///
/// `PreparedWSLCommand` stores:
/// - a NUL-terminated program path (`c_path`),
/// - a NUL-terminated `argv` pointer array (`argv`),
/// - owned C strings backing `argv` (`_c_args`).
///
/// This avoids rebuilding C-compatible buffers when executing the same command
/// multiple times.
///
/// Instances are usually created from [`WSLCommand`] via [`WSLCommand::prepare`]
/// or `From<WSLCommand>`.
#[derive(Debug)]
pub struct PreparedWSLCommand<'a> {
    pub(super) api: &'a ApiV1,
    pub(super) session_id: SessionID,
    pub(super) distribution_id: DistributionID,
    pub(super) c_path: Box<[u8]>,
    pub(super) argv: Box<[*const u8]>,
    pub(super) _c_args: Box<[CString]>,
}

impl WSLCommandExecution for PreparedWSLCommand<'_> {
    /// Executes the prepared command via the underlying WSL Plugin API.
    ///
    /// # Behavior
    ///
    /// - [`DistributionID::System`] uses `ExecuteBinary`.
    /// - [`DistributionID::User`] uses `ExecuteBinaryInDistribution`.
    ///
    /// # Errors
    ///
    /// Returns an API error if the call fails, including version requirements
    /// for distribution-scoped execution.
    #[doc(alias = "ExecuteBinary")]
    #[doc(alias = "ExecuteBinaryInDistribution")]
    #[inline]
    fn execute_stream(&self) -> ApiResult<TcpStream> {
        match self.distribution_id {
            // Safety: the caller ensures that the path and argv are correctly encoded
            DistributionID::System => unsafe {
                self.api
                    .execute_binary_internal(self.session_id, &self.c_path, &self.argv)
                    .map_err(Into::into)
            },
            // Safety: the caller ensures that the path and argv are correctly encoded
            DistributionID::User(id) => unsafe {
                self.api.execute_binary_in_distribution_internal(
                    self.session_id,
                    id,
                    &self.c_path,
                    &self.argv,
                )
            },
        }
    }
}

impl<'a> From<WSLCommand<'a>> for PreparedWSLCommand<'a> {
    #[inline]
    fn from(value: WSLCommand<'a>) -> Self {
        let (c_args, argv) = utils::encode_c_argv(value.argv());
        Self {
            api: value.api,
            session_id: value.session_id,
            distribution_id: value.distribution_id,
            c_path: utils::encode_c_path(&value.path).into_boxed_slice(),
            argv: argv.into_boxed_slice(),
            _c_args: c_args.into_boxed_slice(),
        }
    }
}

impl<'a> From<&WSLCommand<'a>> for PreparedWSLCommand<'a> {
    #[inline]
    fn from(value: &WSLCommand<'a>) -> Self {
        let (c_args, argv) = utils::encode_c_argv(value.argv());
        Self {
            api: value.api,
            session_id: value.session_id,
            distribution_id: value.distribution_id,
            c_path: utils::encode_c_path(&value.path).into_boxed_slice(),
            argv: argv.into_boxed_slice(),
            _c_args: c_args.into_boxed_slice(),
        }
    }
}
