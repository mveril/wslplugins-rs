use typed_path::Utf8UnixPath;

#[cfg(doc)]
use super::super::api::Error as ApiError;
use super::super::api::{ApiV1, Result as ApiResult};
use crate::{DistributionID, WSLSessionInformation};
use std::net::TcpStream;
#[cfg(doc)]
use windows::core::GUID;

/// Represents a command to be executed in WSL.
///
/// The `WSLCommand` struct encapsulates details such as the program path, arguments,
/// and the associated distribution ID for execution.
#[derive(Clone)]
pub struct WSLCommand<'a> {
    /// The WSL context associated with the command.
    api: &'a ApiV1<'a>,
    /// Arguments for the command.
    args: Vec<&'a str>,
    /// Path to the program being executed.
    path: &'a Utf8UnixPath,
    /// The distribution ID under which the command is executed.
    distribution_id: DistributionID,
    /// Session information for the current WSL session.
    session: &'a WSLSessionInformation<'a>,
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
    pub(crate) fn new<T: AsRef<Utf8UnixPath> + ?Sized>(
        api: &'a ApiV1<'a>,
        session: &'a WSLSessionInformation<'a>,
        program: &'a T,
    ) -> Self {
        let my_program = program.as_ref();
        let program_str = my_program.as_str();
        Self {
            api,
            args: vec![program_str],
            path: my_program,
            distribution_id: DistributionID::System,
            session,
        }
    }

    /// Returns the path of the command.
    pub fn get_path(&self) -> &'a Utf8UnixPath {
        self.path
    }

    /// Sets the first argument (arg0) of the command.
    ///
    /// # Parameters
    /// - `arg0`: The new value for the first argument.
    pub fn arg0(&mut self, arg0: &'a str) -> &mut Self {
        self.args[0] = arg0;
        self
    }

    /// Gets the first argument (arg0) of the command.
    pub fn get_arg0(&self) -> &str {
        self.args[0]
    }

    /// Checks if the first argument is the standard argument (the path).
    pub fn is_standard_arg_0(&self) -> bool {
        self.path == self.get_arg0()
    }

    /// Resets the first argument to the path of the command.
    pub fn reset_arg0(&mut self) -> &mut Self {
        self.args[0] = self.path.as_str();
        self
    }

    /// Adds an argument to the command.
    ///
    /// # Parameters
    /// - `arg`: The argument to add.
    pub fn arg(&mut self, arg: &'a str) -> &mut Self {
        self.args.push(arg);
        self
    }

    /// Adds multiple arguments to the command.
    ///
    /// # Parameters
    /// - `args`: An iterator of arguments to add.
    pub fn args<I: IntoIterator<Item = &'a str>>(&mut self, args: I) -> &mut Self {
        self.args.extend(args);
        self
    }

    /// Returns an iterator over the arguments of the command, excluding arg0.
    pub fn get_args(&self) -> impl ExactSizeIterator<Item = &str> {
        self.args[1..].iter().copied()
    }

    /// Sets the distribution ID for the command.
    ///
    /// # Parameters
    /// - `distribution_id`: The new distribution ID to set.
    pub fn distribution_id(&mut self, distribution_id: DistributionID) -> &mut Self {
        self.distribution_id = distribution_id;
        self
    }

    /// Resets the distribution ID to the system default.
    pub fn reset_distribution_id(&mut self) -> &mut Self {
        self.distribution_id = DistributionID::System;
        self
    }

    /// Gets the current distribution ID for the command.
    pub fn get_distribution_id(&self) -> DistributionID {
        self.distribution_id
    }

    /// Executes the command and returns a [`TcpStream`].
    ///
    /// This method determines the API call to be used based on the [`DistributionID`]:
    /// - If [`DistributionID::System`], the method invokes [`execute_binary`](super::ApiV1::execute_binary).
    /// - If [`DistributionID::User`], it invokes [`execute_binary_in_distribution`](super::ApiV1::execute_binary_in_distribution)
    ///   with the associated [GUID].
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
    pub fn execute(&mut self) -> ApiResult<TcpStream> {
        let stream = match self.distribution_id {
            DistributionID::System => {
                self.api
                    .execute_binary(self.session, self.path, self.args.as_slice())?
            }
            DistributionID::User(id) => self.api.execute_binary_in_distribution(
                self.session,
                id,
                self.path,
                self.args.as_slice(),
            )?,
        };
        Ok(stream)
    }
}
