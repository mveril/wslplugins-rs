use super::{
    WSLCApi, WSLCCommandError, WSLCExitStatus, WSLCOutput, WSLCProcess, WSLCSessionID,
    WSLCSpawnError,
};
use crate::api::wsl_command::IntoCowUtf8UnixPath;
use std::borrow::Cow;
use typed_path::Utf8UnixPath;
pub struct WSLCCommand<'api, 'command> {
    api: WSLCApi<'api>,
    session_id: WSLCSessionID,
    program: Cow<'command, Utf8UnixPath>,
    arg0: Option<Cow<'command, str>>,
    args: Vec<Cow<'command, str>>,
    environment: Vec<Cow<'command, str>>,
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
            args: Vec::new(),
            environment: Vec::new(),
        }
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

    /// Adds an environment entry in `NAME=VALUE` form.
    #[inline]
    pub fn environment<T>(&mut self, entry: T) -> &mut Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.environment.push(entry.into());
        self
    }

    /// Adds an environment entry in `NAME=VALUE` form.
    #[must_use]
    #[inline]
    pub fn with_environment<T>(mut self, entry: T) -> Self
    where
        T: Into<Cow<'command, str>>,
    {
        self.environment.push(entry.into());
        self
    }

    /// Sets an environment variable for the child process.
    #[inline]
    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.environment
            .push(Cow::Owned(format!("{}={}", key.as_ref(), value.as_ref())));
        self
    }

    /// Sets environment variables for the child process.
    #[inline]
    pub fn envs<I, K, V>(&mut self, variables: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.environment.extend(
            variables
                .into_iter()
                .map(|(key, value)| Cow::Owned(format!("{}={}", key.as_ref(), value.as_ref()))),
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

    /// Returns an iterator over `NAME=VALUE` environment entries.
    #[inline]
    pub fn get_environment(&self) -> impl ExactSizeIterator<Item = &str> {
        self.environment.iter().map(AsRef::as_ref)
    }

    /// Creates the process.
    ///
    /// # Errors
    ///
    /// Returns the Windows failure and the Linux `errno` reported by WSL when
    /// process creation fails.
    #[doc(alias = "WSLCCreateProcess")]
    #[inline]
    pub fn spawn(&self) -> std::result::Result<WSLCProcess<'api>, WSLCSpawnError> {
        let arg0 = self.arg0.as_deref().unwrap_or(self.program.as_str());
        let arguments = std::iter::once(arg0).chain(self.args.iter().map(AsRef::as_ref));
        self.api.create_process(
            self.session_id,
            &self.program,
            arguments,
            self.environment.iter().map(AsRef::as_ref),
        )
    }

    /// Executes the command and waits for it to finish.
    ///
    /// Standard input is closed immediately. Standard output and standard
    /// error are drained concurrently.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot be created, a stream cannot be
    /// read, or the process status cannot be obtained.
    #[inline]
    pub fn output(&self) -> std::result::Result<WSLCOutput, WSLCCommandError> {
        self.spawn()?.wait_with_output()
    }

    /// Executes the command and waits for its exit status.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot be created or its status cannot
    /// be obtained.
    #[inline]
    pub fn status(&self) -> std::result::Result<WSLCExitStatus, WSLCCommandError> {
        self.spawn()?.wait_with_status()
    }
}
