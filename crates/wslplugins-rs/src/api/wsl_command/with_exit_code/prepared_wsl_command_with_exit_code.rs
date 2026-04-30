use crate::{
    api::{
        utils,
        wsl_command::{
            prepared_wsl_command::PreparedWSLCommand, wsl_command_execution::WSLCommandExecution,
            WSLChild, WSLCommand, WSLCommandExecutionError, WSLCommandExecutionResult,
        },
        Result as ApiResult,
    },
    DistributionID,
};
use std::iter::once;
use std::net::TcpStream;
use typed_path::Utf8UnixPath;

/// A command prepared to emit its process exit code on stdout before exiting.
///
/// The original command is executed through `/bin/sh -c`, and the shell appends
/// `ESC ] 9 ; 4 ; <exit_code> BEL` after the command completes.
#[derive(Debug)]
pub struct PreparedWSLCommandWithExitCode<'a> {
    command: PreparedWSLCommand<'a>,
}

impl WSLCommandExecution for PreparedWSLCommandWithExitCode<'_> {
    #[inline]
    fn execute_stream(&self) -> ApiResult<TcpStream> {
        self.command.execute_stream()
    }

    #[inline]
    fn execute(&self) -> WSLCommandExecutionResult<WSLChild> {
        let stream = self.execute_stream()?;
        match stream.try_clone() {
            Ok(stdin_stream) => Ok(WSLChild::new_with_exit_code(stdin_stream, stream)),
            Err(source) => Err(WSLCommandExecutionError::Split { source, stream }),
        }
    }
}

impl<'a> From<WSLCommand<'a>> for PreparedWSLCommandWithExitCode<'a> {
    #[inline]
    fn from(value: WSLCommand<'a>) -> Self {
        Self::from(&value)
    }
}

impl<'a> From<&WSLCommand<'a>> for PreparedWSLCommandWithExitCode<'a> {
    fn from(value: &WSLCommand<'a>) -> Self {
        const EXIT_CODE_SCRIPT: &str = concat!(
            "\"$@\"\n",
            "code=$?\n",
            "printf '\\033]9;4;%s\\007' \"$code\"\n",
            "exit \"$code\"",
        );

        let argv = once("/bin/sh")
            .chain(["-c", EXIT_CODE_SCRIPT, "wslplugins-rs-exit-code"])
            .chain(once(value.get_path().as_str()))
            .chain(value.get_args());
        let (c_args, argv) = utils::encode_c_argv(argv);

        Self {
            command: PreparedWSLCommand {
                api: value.api,
                session_id: value.session_id,
                distribution_id: DistributionID::from(value.distribution_id),
                c_path: utils::encode_c_path(Utf8UnixPath::new("/bin/sh")).into_boxed_slice(),
                argv: argv.into_boxed_slice(),
                _c_args: c_args.into_boxed_slice(),
            },
        }
    }
}
