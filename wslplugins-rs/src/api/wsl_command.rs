use typed_path::Utf8UnixPath;

#[cfg(doc)]
use super::super::api::Error as ApiError;
use super::super::api::{ApiV1, Result as ApiResult};
#[cfg(doc)]
use crate::UserDistributionID;
use crate::{DistributionID, SessionID};
use std::{borrow::Cow, net::TcpStream};
mod into_cow_utf8_unix_path;
pub use into_cow_utf8_unix_path::IntoCowUtf8UnixPath;

/// Represents a command to be executed in WSL.
///
/// The `WSLCommand` struct encapsulates details such as the program path, arguments,
/// and the associated distribution ID for execution.
#[derive(Clone, Debug)]
pub struct WSLCommand<'a> {
    /// The WSL context associated with the command.
    api: &'a ApiV1,
    /// Session information for the current WSL session.
    session_id: SessionID,
    /// The distribution ID under which the command is executed.
    distribution_id: DistributionID,
    /// Path to the program being executed.
    path: Cow<'a, Utf8UnixPath>,
    /// Optional argv[0] override.
    arg0: Option<Cow<'a, str>>,
    /// Arguments for the command.
    args: Vec<Cow<'a, str>>,
}

impl<'a> WSLCommand<'a> {
    /// Creates a new `WSLCommand` instance.
    ///
    /// This function initializes a `WSLCommand` with the necessary details to
    /// execute a program in a WSL instance.
    ///
    /// # Parameters
    /// - `api`: A reference to the WSL API version 1.
    /// - `session`: The session information associated with the WSL instance.
    /// - `program`: A reference to the path of the program to be executed,
    ///   represented as an object implementing `AsRef<Utf8UnixPath>`.
    ///
    /// # Returns
    /// A new `WSLCommand` instance.
    ///
    /// # Type Parameters
    /// - `T`: A type that implements `AsRef<Utf8UnixPath>`.
    pub(crate) fn new<P: IntoCowUtf8UnixPath<'a>>(
        api: &'a ApiV1,
        session_id: SessionID,
        program: P,
    ) -> Self {
        Self {
            api,
            arg0: None,
            args: Vec::new(),
            path: program.into_cow_utf8_unix_path(),
            distribution_id: DistributionID::System,
            session_id,
        }
    }

    /// Returns the path of the command.
    #[inline]
    #[must_use]
    pub fn get_path(&self) -> &Utf8UnixPath {
        self.path.as_ref()
    }

    /// Sets the first argument (arg0) of the command.
    ///
    /// # Parameters
    /// - `arg0`: The new value for the first argument.
    #[inline]
    #[expect(
        clippy::indexing_slicing,
        reason = "The vec is known to have at least one value (the arg0)"
    )]
    pub fn arg0<T: Into<Cow<'a, str>> + ?Sized>(&mut self, arg0: T) -> &mut Self {
        self.arg0 = Some(arg0.into());
        self
    }

    /// Gets the first argument (arg0) of the command.
    #[inline]
    #[must_use]
    pub fn get_arg0(&self) -> &str {
        self.arg0.as_deref().unwrap_or_else(|| self.path.as_str())
    }

    /// Checks if the first argument is the standard argument (the path).
    #[inline]
    #[must_use]
    pub fn is_standard_arg_0(&self) -> bool {
        self.arg0.is_none()
    }

    /// Resets the first argument to the path of the command.
    #[inline]
    pub fn reset_arg0(&mut self) -> &mut Self {
        self.arg0 = None;
        self
    }

    /// Adds an argument to the command.
    ///
    /// # Parameters
    /// - `arg`: The argument to add.
    #[inline]
    pub fn arg<T: Into<Cow<'a, str>> + ?Sized>(&mut self, arg: T) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to the command.
    ///
    /// # Parameters
    /// - `args`: An iterator of arguments to add.
    #[inline]
    pub fn args<I>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'a, str>>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Returns an iterator over the arguments of the command, excluding arg0.
    #[inline]
    #[must_use]
    pub fn get_args(&self) -> impl ExactSizeIterator<Item = &str> {
        self.args.iter().map(AsRef::as_ref)
    }

    /// Clears all arguments except arg0.
    ///
    /// This method removes all additional arguments from the command,
    /// effectively resetting the arguments to only include the program path.
    ///
    /// # Returns
    /// A mutable reference to the current `WSLCommand` instance.
    ///
    /// # Example
    /// ```rust ignore
    /// command.arg("Hello").arg("World");
    /// command.clear_args(); // Only arg0 ("/bin/echo") remains.
    /// assert_eq!(command.get_args().count(), 0)
    /// ```
    #[inline]
    pub fn clear_args(&mut self) -> &mut Self {
        self.args.clear();
        self
    }

    /// Truncates the arguments of the command after a specified index.
    ///
    /// This method keeps `arg0` and the first `i` additional arguments, discarding the rest.
    ///
    /// # Parameters
    /// - `i`: The index after which arguments will be removed. Note that `i = 0` keeps only `arg0`.
    ///
    /// # Returns
    /// A mutable reference to the current `WSLCommand` instance.
    ///
    /// # Example
    /// ```rust ignore
    /// let mut command = WSLCommand::new(context, session, "/bin/echo");
    /// command.arg("Hello").arg("World");
    /// command.truncate_args(1); // Keeps only "/bin/echo" and "Hello".
    /// ```
    #[inline]
    pub fn truncate_args(&mut self, i: usize) -> &mut Self {
        self.args.truncate(i);
        self
    }

    /// Sets the distribution ID for the command.
    ///
    /// # Parameters
    /// - `distribution_id`: The new distribution ID to set.
    #[inline]
    pub const fn distribution_id(&mut self, distribution_id: DistributionID) -> &mut Self {
        self.distribution_id = distribution_id;
        self
    }

    /// Resets the distribution ID to the system default.
    #[inline]
    pub const fn reset_distribution_id(&mut self) -> &mut Self {
        self.distribution_id = DistributionID::System;
        self
    }

    /// Gets the current distribution ID for the command.
    #[inline]
    #[must_use]
    pub const fn get_distribution_id(&self) -> DistributionID {
        self.distribution_id
    }

    /// Executes the command and returns a [`TcpStream`].
    ///
    /// This method determines the API call to be used based on the [`DistributionID`]:
    /// - If [`DistributionID::System`], the method invokes [`execute_binary`](super::ApiV1::execute_binary).
    /// - If [`DistributionID::User`], it invokes [`execute_binary_in_distribution`](super::ApiV1::execute_binary_in_distribution)
    ///   with the associated [`UserDistributionID`].
    ///
    /// # Returns
    /// - On success, it returns a [`TcpStream`] connected to the executed process, enabling interaction
    ///   with the process's stdin and stdout.
    /// - On failure, an error indicating the cause of the failure.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - [`ApiError::RequiresUpdate`]: The WSL runtime version does not support targeting a user distribution.
    /// - [`ApiError::WinError`]: The API call fails to execute the binary.
    ///
    /// # Example
    /// ```rust ignore
    /// let mut command = ;
    /// match WSLCommand::new(context, session, "/bin/echo").arg("Hello, World!").execute() {
    ///     Ok(stream) => {
    ///         // Interact with the process via the TcpStream.
    ///     },
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    #[inline]
    pub fn execute(&self) -> ApiResult<TcpStream> {
        let all_args_iter = std::iter::once(self.get_arg0())
            .into_iter()
            .chain(self.args.iter().map(|item| item.as_ref()));
        let stream = match self.distribution_id {
            DistributionID::System => {
                self.api
                    .execute_binary(self.session_id, &self.path, all_args_iter)?
            }
            DistributionID::User(id) => self.api.execute_binary_in_distribution(
                self.session_id,
                id,
                &self.path,
                all_args_iter,
            )?,
        };
        Ok(stream)
    }
}
