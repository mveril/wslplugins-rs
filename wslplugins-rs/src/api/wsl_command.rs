//! # WSL Distribution and WSLCommand Management Module
//!
//! This module provides abstractions for identifying and manipulating WSL
//! distributions and for running commands in those distributions. It builds on the WSL APIs and
//! uses [GUID]s to uniquely identify distributions.

use std::{convert::TryFrom, fmt::Display, net::TcpStream};
use thiserror::Error;
use windows::core::GUID;

#[cfg(doc)]
use super::super::api::Error as ApiError;
use super::super::api::Result as ApiResult;
use crate::{CoreDistributionInformation, WSLContext, WSLSessionInformation};

/// Represents a distribution identifier in WSL.
///
/// This can either be the system-level distribution or a user-specific distribution
/// identified by a GUID.
#[derive(Debug, Clone, Copy)]
pub enum DistributionID {
    /// Represents the system-level distribution.
    System,
    /// Represents a user-specific distribution identified by a GUID.
    User(GUID),
}

/// Error type for conversion failures between `DistributionID` and GUID.
#[derive(Debug, Error)]
#[error("Cannot convert System distribution to GUID.")]
pub struct ConversionError;

impl TryFrom<DistributionID> for GUID {
    type Error = ConversionError;

    /// Attempts to convert a `DistributionID` into a GUID.
    ///
    /// # Errors
    /// Returns `ConversionError` if the `DistributionID` is `System`.
    fn try_from(value: DistributionID) -> Result<Self, Self::Error> {
        match value {
            DistributionID::User(id) => Ok(id),
            DistributionID::System => Err(ConversionError),
        }
    }
}

impl From<GUID> for DistributionID {
    /// Converts a GUID into a `DistributionID`.
    fn from(value: GUID) -> Self {
        Self::User(value)
    }
}

impl<T: CoreDistributionInformation> From<T> for DistributionID {
    /// Converts a type implementing `CoreDistributionInformation` into a `DistributionID`.
    fn from(value: T) -> Self {
        (*value.id()).into()
    }
}

impl From<Option<GUID>> for DistributionID {
    /// Converts an `Option<GUID>` into a `DistributionID`, defaulting to `System` if `None`.
    fn from(value: Option<GUID>) -> Self {
        match value {
            Some(id) => Self::User(id),
            None => Self::System,
        }
    }
}

impl From<DistributionID> for Option<GUID> {
    /// Converts a `DistributionID` into an `Option<GUID>`.
    fn from(value: DistributionID) -> Self {
        match value {
            DistributionID::System => None,
            DistributionID::User(id) => Some(id),
        }
    }
}

impl Display for DistributionID {
    /// Formats the `DistributionID` for display.
    ///
    /// Displays "System" for the system-level distribution, or the GUID for a user-specific distribution.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributionID::System => f.write_str("System"),
            DistributionID::User(id) => std::fmt::Debug::fmt(id, f),
        }
    }
}

/// Represents a command to be executed in WSL.
///
/// The `WSLCommand` struct encapsulates details such as the program path, arguments,
/// and the associated distribution ID for execution.
#[derive(Clone)]
pub struct WSLCommand<'a> {
    /// The WSL context associated with the command.
    context: &'static WSLContext,
    /// Arguments for the command.
    args: Vec<&'a str>,
    /// Path to the program being executed.
    path: &'a str,
    /// The distribution ID under which the command is executed.
    distribution_id: DistributionID,
    /// Session information for the current WSL session.
    session: &'a WSLSessionInformation<'a>,
}

impl<'a> WSLCommand<'a> {
    /// Creates a new `WSLCommand` instance.
    ///
    /// # Parameters
    /// - `context`: The WSL context.
    /// - `session`: The session information for the WSL instance.
    /// - `program`: The path to the program to be executed.
    pub fn new(
        context: &'static WSLContext,
        session: &'a WSLSessionInformation<'a>,
        program: &'a str,
    ) -> Self {
        Self {
            context,
            args: vec![program],
            path: program,
            distribution_id: DistributionID::System,
            session,
        }
    }

    /// Returns the path of the command.
    pub fn get_path(&self) -> &'a str {
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
        self.args[0] = self.path;
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
    ///   with the associated [`GUID`].
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
                self.context
                    .api
                    .execute_binary(self.session, self.path, self.args.as_slice())?
            }
            DistributionID::User(id) => self.context.api.execute_binary_in_distribution(
                self.session,
                &id,
                self.path,
                self.args.as_slice(),
            )?,
        };
        Ok(stream)
    }
}
