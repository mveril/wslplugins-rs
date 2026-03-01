use super::wsl_command_execution::WSLCommandExecution;
use super::WSLCommand;
use std::ffi::CString;
use std::net::TcpStream;

use crate::{
    api::{utils, ApiV1, Result as ApiResult},
    DistributionID, SessionID,
};

#[derive(Debug)]
pub struct PreparedWSLCommand<'a> {
    api: &'a ApiV1,
    session_id: SessionID,
    distribution_id: DistributionID,
    c_path: Box<[u8]>,
    argv: Box<[*const u8]>,
    _c_args: Box<[CString]>,
}

impl WSLCommandExecution for PreparedWSLCommand<'_> {
    #[inline]
    fn execute(&self) -> ApiResult<TcpStream> {
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
