use super::{
    PreparedWSLCCommand, WSLCApi, WSLCCommandExecution, WSLCProcessStreams, WSLCSessionID,
};
use crate::api::wsl_command::IntoCowUtf8UnixPath;
use std::borrow::Cow;
use typed_path::Utf8UnixPath;

#[cfg(feature = "smallvec")]
type ArgumentVec<'a> = smallvec::SmallVec<[Cow<'a, str>; 8]>;
#[cfg(not(feature = "smallvec"))]
type ArgumentVec<'a> = Vec<Cow<'a, str>>;

#[cfg(feature = "smallvec")]
type EnvironmentVec<'a> = smallvec::SmallVec<[(Cow<'a, str>, Cow<'a, str>); 8]>;
#[cfg(not(feature = "smallvec"))]
type EnvironmentVec<'a> = Vec<(Cow<'a, str>, Cow<'a, str>)>;

pub struct WSLCCommand<'api, 'command> {
    api: WSLCApi<'api>,
    session_id: WSLCSessionID,
    program: Cow<'command, Utf8UnixPath>,
    arg0: Option<Cow<'command, str>>,
    args: ArgumentVec<'command>,
    environment: EnvironmentVec<'command>,
    streams: WSLCProcessStreams,
}

impl<'api, 'command> WSLCCommand<'api, 'command> {
    pub(super) fn new<P>(api: WSLCApi<'api>, session_id: WSLCSessionID, program: P) -> Self
    where
        P: IntoCowUtf8UnixPath<'command>,
    {
        Self {
            api,
            session_id,
            program: program.into_cow_utf8_unix_path(),
            arg0: None,
            args: ArgumentVec::new(),
            environment: EnvironmentVec::new(),
            streams: WSLCProcessStreams::default(),
        }
    }

    /// Acquires a writable handle for the child process's standard input.
    #[inline]
    pub fn pipe_stdin(&mut self) -> &mut Self {
        self.streams.insert(WSLCProcessStreams::STDIN);
        self
    }

    /// Acquires a writable handle for the child process's standard input.
    #[inline]
    #[mustè_use]
    pub fn with_pipe_stdin(&mut self) -> Self {
        self.streams.insert(WSLCProcessStreams::STDIN);
        self
    }

    /// Acquires a readable handle for the child process's standard output.
    #[inline]
    pub fn pipe_stdout(&mut self) -> &mut Self {
        self.streams.insert(WSLCProcessStreams::STDOUT);
        self
    }

    /// Acquires a readable handle for the child process's standard output.
    #[inline]
    #[must_use]
    pub fn with_pipe_stdout(&mut self) -> Self {
        self.streams.insert(WSLCProcessStreams::STDOUT);
        self
    }

    /// Acquires a readable handle for the child process's standard error.
    #[inline]
    pub fn pipe_stderr(&mut self) -> &mut Self {
        self.streams.insert(WSLCProcessStreams::STDERR);
        self
    }

        /// Acquires a readable handle for the child process's standard error.
    #[inline]
    #[must_use]
    pub fn with_pipe_stderr(&mut self) -> Self {
        self.streams.insert(WSLCProcessStreams::STDERR);
        self
    }

    /// Acquires handles for all three standard streams.
    #[inline]
    pub fn pipe_standard_streams(&mut self) -> &mut Self {
        self.streams = WSLCProcessStreams::ALL;
        self
    }

    /// Acquires handles for all three standard streams.
    #[inline]
    #[must_use]
    pub fn with_pipe_standard_streams(&mut self) -> Self {
        self.streams = WSLCProcessStreams::ALL;
        self
    }

    /// Sets `argv[0]`.
    #[inline]
    pub fn arg0<T>(&mut self, arg0: T) -> &mut Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.arg0 = Some(arg0.into());
        self
    }

    /// Sets `argv[0]`.
    #[must_use]
    #[inline]
    pub fn with_arg0<T>(mut self, arg0: T) -> Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.arg0 = Some(arg0.into());
        self
    }

    /// Adds one command-line argument.
    #[inline]
    pub fn arg<T>(&mut self, argument: T) -> &mut Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.args.push(argument.into());
        self
    }

    /// Adds one command-line argument.
    #[must_use]
    #[inline]
    pub fn with_arg<T>(mut self, argument: T) -> Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.args.push(argument.into());
        self
    }

    /// Adds command-line arguments.
    #[inline]
    pub fn args<I, T>(&mut self, arguments: I) -> &mut Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Cow<'command, str>>,
    {
        self.args.extend(arguments.into_iter().map(Into::into));
        self
    }

    /// Adds command-line arguments.
    #[must_use]
    #[inline]
    pub fn with_args<I, T>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Cow<'command, str>>,
    {
        self.args.extend(arguments.into_iter().map(Into::into));
        self
    }

    /// Sets an environment variable for the child process.
    #[inline]
    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<Cow<'command, str>>,
        V: Into<Cow<'command, str>>,
    {
        self.environment.push((key.into(), value.into()));
        self
    }

    /// Sets an environment variable for the child process.
    #[must_use]
    #[inline]
    pub fn with_env<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Cow<'command, str>>,
        V: Into<Cow<'command, str>>,
    {
        self.environment.push((key.into(), value.into()));
        self
    }

    /// Sets environment variables for the child process.
    #[inline]
    pub fn envs<I, K, V>(&mut self, variables: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Cow<'command, str>>,
        V: Into<Cow<'command, str>>,
    {
        self.environment.extend(
            variables
                .into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }

    /// Sets environment variables for the child process.
    #[must_use]
    #[inline]
    pub fn with_envs<I, K, V>(mut self, variables: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Cow<'command, str>>,
        V: Into<Cow<'command, str>>,
    {
        self.environment.extend(
            variables
                .into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }

    /// Returns the command's program.
    #[must_use]
    #[inline]
    pub fn get_program(&self) -> &Utf8UnixPath {
        &self.program
    }

    /// Returns an iterator over the command's arguments, excluding `argv[0]`.
    #[inline]
    pub fn get_args(&self) -> impl ExactSizeIterator<Item = &str> {
        self.args.iter().map(AsRef::as_ref)
    }

    /// Returns an iterator over environment key-value pairs.
    #[must_use]
    #[inline]
    pub fn get_environment(&self) -> impl ExactSizeIterator<Item = (&str, &str)> {
        self.environment
            .iter()
            .map(|(key, value)| (key.as_ref(), value.as_ref()))
    }

    /// Encodes this command for repeated process creation.
    #[must_use]
    #[inline]
    pub fn prepare(&self) -> PreparedWSLCCommand<'api> {
        let arg0 = self.arg0.as_deref().unwrap_or(self.program.as_str());
        PreparedWSLCCommand::new(
            self.api,
            self.session_id,
            &self.program,
            std::iter::once(arg0).chain(self.args.iter().map(AsRef::as_ref)),
            self.get_environment(),
            self.streams,
        )
    }
}

impl<'api> WSLCCommandExecution<'api> for WSLCCommand<'api, '_> {
    #[inline]
    fn spawn(&self) -> std::result::Result<super::WSLCProcess<'api>, super::WSLCSpawnError> {
        self.prepare().spawn()
    }

    #[inline]
    fn output(&self) -> std::result::Result<super::WSLCOutput, super::WSLCCommandError> {
        self.prepare().spawn_for_output()?.wait_with_output()
    }
}
