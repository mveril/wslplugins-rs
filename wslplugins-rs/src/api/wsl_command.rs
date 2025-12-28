use super::super::api::{ApiV1, Result as ApiResult};
use crate::{DistributionID, SessionID};
use std::{borrow::Cow, iter::once, net::TcpStream};
use typed_path::Utf8UnixPath;

mod into_cow_utf8_unix_path;
pub use into_cow_utf8_unix_path::IntoCowUtf8UnixPath;

#[cfg(feature = "smallvec")]
use smallvec::SmallVec;

#[cfg(doc)]
use super::super::api::Error as ApiError;

#[cfg(not(feature = "smallvec"))]
type ArgVec<'a> = Vec<Cow<'a, str>>;

#[cfg(feature = "smallvec")]
type ArgVec<'a> = SmallVec<[Cow<'a, str>; 8]>;

/// A prepared command to be executed inside WSL.
///
/// `WSLCommand` is a builder-style abstraction around the WSL Plugin API
/// execution functions (`ExecuteBinary` / `ExecuteBinaryInDistribution`).
///
/// Instances of `WSLCommand` are created through [`ApiV1::new_command`],
/// ensuring they are always tied to a valid API handle and session.
///
/// # Key points
///
/// - The program path is a **Linux path** (UTF-8, Unix-style) represented by
///   [`Utf8UnixPath`].
/// - Arguments are stored as `Cow<'a, str>` to minimize allocations.
/// - `argv[0]` can be overridden; otherwise it defaults to the program path.
/// - The execution target can be the system context or a specific distribution.
///
/// # Argument semantics
///
/// The underlying WSL Plugin API expects a NULL-terminated `argv` array:
///
/// ```text
/// argv[0] = program name
/// argv[1..] = user arguments
/// ```
///
/// This type exposes:
/// - [`WSLCommand::argv`] for iterating over the full argument vector,
/// - [`WSLCommand::arg0`] / [`WSLCommand::with_arg0`] to override `argv[0]`.
///
/// # Examples
///
/// ## Basic execution
///
/// ```no_run
/// # use wslplugins_rs::{SessionID};
/// # use wslplugins_rs::api::ApiV1;
/// # fn demo(api: &ApiV1) -> Result<(), Box<dyn std::error::Error>> {
/// let stream = api
///     .new_command(SessionID::from(0), "/bin/cat")
///     .with_arg("/proc/version")
///     .execute()?;
///
/// # drop(stream);
/// # Ok(())
/// # }
/// ```
///
/// ## Overriding `argv[0]`
///
/// Some programs inspect `argv[0]` to determine their behavior
/// (for example `busybox`).
///
/// ```no_run
/// # use wslplugins_rs::{SessionID};
/// # use wslplugins_rs::api::ApiV1;
/// # fn demo(api: &ApiV1, session_id: SessionID) -> Result<(), Box<dyn std::error::Error>> {
/// let stream = api
///     .new_command(session_id, "/bin/busybox")
///     .with_arg0("sh")
///     .with_args(["-c", "echo hello"])
///     .execute()?;
///
/// # drop(stream);
/// # Ok(())
/// # }
/// ```
///
/// ## Executing in a user distribution
///
/// ```no_run
/// # use wslplugins_rs::{DistributionID, SessionID};
/// # use wslplugins_rs::api::ApiV1;
/// # use wslplugins_rs::UserDistributionID;
/// # fn demo(api: &ApiV1) -> Result<(), Box<dyn std::error::Error>> {
/// let distro: UserDistributionID = "3B6F3C1E-9B4A-4F2C-8E7A-2A9C6D4E1F52".parse().unwrap();
/// let stream = api
///     .new_command(SessionID::from(0), "/bin/echo")
///     .with_distribution_id(DistributionID::User(distro))
///     .with_arg("hello")
///     .execute()?;
/// # Ok(())
/// # }
/// ```
///
/// # Notes
///
/// - [`WSLCommand::execute`] consumes the command and returns a connected
///   [`TcpStream`] to the process stdin/stdout.
/// - stderr is forwarded to `dmesg` on the Linux side.
/// - This type performs no validation of the Linux path or arguments beyond UTF-8 handling.
#[derive(Clone, Debug)]
pub struct WSLCommand<'a> {
    /// Reference to the API v1 handle used to perform the execution.
    api: &'a ApiV1,

    /// Session in which the program should be executed.
    session_id: SessionID,

    /// Target distribution selection.
    ///
    /// - `System` executes in the root namespace.
    /// - `User(id)` executes in a user distribution identified by its GUID.
    distribution_id: DistributionID,

    /// Linux program path (UTF-8, Unix path).
    path: Cow<'a, Utf8UnixPath>,

    /// Optional override for `argv[0]`.
    ///
    /// If `None`, `argv[0]` defaults to `path.as_str()`.
    arg0: Option<Cow<'a, str>>,

    /// Additional arguments (`argv[1..]`).
    args: ArgVec<'a>,
}

impl<'a> WSLCommand<'a> {
    /// Creates a new command builder for the given program.
    ///
    /// - `program` is converted to a [`Utf8UnixPath`] using [`IntoCowUtf8UnixPath`].
    /// - The default target is [`DistributionID::System`].
    /// - `argv[0]` is the program path string unless overridden via [`WSLCommand::arg0`]
    ///   or [`WSLCommand::with_arg0`].
    pub(crate) fn new<P: IntoCowUtf8UnixPath<'a>>(
        api: &'a ApiV1,
        session_id: SessionID,
        program: P,
    ) -> Self {
        Self {
            api,
            arg0: None,
            args: ArgVec::new(),
            path: program.into_cow_utf8_unix_path(),
            distribution_id: DistributionID::System,
            session_id,
        }
    }

    /// Returns the program path as a [`Utf8UnixPath`].
    #[inline]
    #[must_use]
    pub fn get_path(&self) -> &Utf8UnixPath {
        self.path.as_ref()
    }

    /// Returns `argv[0]`.
    ///
    /// If `arg0` has not been overridden, this returns the program path string.
    #[inline]
    #[must_use]
    pub fn get_arg0(&self) -> &str {
        self.arg0.as_deref().unwrap_or(self.path.as_str())
    }

    /// Returns `true` if `argv[0]` is the default value (the program path).
    #[inline]
    #[must_use]
    pub const fn is_standard_arg_0(&self) -> bool {
        self.arg0.is_none()
    }

    /// Resets `argv[0]` to its default value (the program path).
    #[inline]
    pub fn reset_arg0(&mut self) -> &mut Self {
        self.arg0 = None;
        self
    }

    /// Sets `argv[0]` (builder-style, by mutable reference).
    #[inline]
    pub fn arg0<T: Into<Cow<'a, str>>>(&mut self, arg0: T) -> &mut Self {
        self.arg0 = Some(arg0.into());
        self
    }

    /// Sets `argv[0]` (builder-style, by value).
    #[inline]
    #[must_use]
    pub fn with_arg0<T: Into<Cow<'a, str>>>(mut self, arg0: T) -> Self {
        self.arg0 = Some(arg0.into());
        self
    }

    /// Pushes one argument (`argv[n]`, `n >= 1`), builder-style by mutable reference.
    #[inline]
    pub fn arg<T: Into<Cow<'a, str>>>(&mut self, arg: T) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    /// Pushes one argument (`argv[n]`, `n >= 1`), builder-style by value.
    #[inline]
    #[must_use]
    pub fn with_arg<T: Into<Cow<'a, str>>>(mut self, arg: T) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Extends arguments from an iterator, builder-style by mutable reference.
    #[inline]
    pub fn args<I>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'a, str>>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Extends arguments from an iterator, builder-style by value.
    #[inline]
    #[must_use]
    pub fn with_args<I>(mut self, args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'a, str>>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Returns an iterator over user-provided arguments (`argv[1..]`).
    #[inline]
    #[must_use]
    pub fn get_args(&self) -> impl ExactSizeIterator<Item = &str> {
        self.args.iter().map(AsRef::as_ref)
    }

    /// Returns an iterator over the full argv (`argv[0]` + `argv[1..]`).
    #[inline]
    pub fn argv(&self) -> impl Iterator<Item = &str> {
        once(self.get_arg0()).chain(self.args.iter().map(AsRef::as_ref))
    }

    /// Clears user-provided arguments (`argv[1..]`).
    #[inline]
    pub fn clear_args(&mut self) -> &mut Self {
        self.args.clear();
        self
    }

    /// Truncates user-provided arguments (`argv[1..]`) to length `i`.
    #[inline]
    pub fn truncate_args(&mut self, i: usize) -> &mut Self {
        self.args.truncate(i);
        self
    }

    /// Sets the distribution target (builder-style by mutable reference).
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "Useless const")]
    pub fn distribution_id(&mut self, distribution_id: DistributionID) -> &mut Self {
        self.distribution_id = distribution_id;
        self
    }

    /// Sets the distribution target (builder-style by value).
    #[inline]
    #[must_use]
    #[allow(clippy::missing_const_for_fn, reason = "Useless const")]
    pub fn with_distribution_id(mut self, distribution_id: DistributionID) -> Self {
        self.distribution_id = distribution_id;
        self
    }

    /// Resets the distribution target to [`DistributionID::System`].
    #[inline]
    #[allow(clippy::missing_const_for_fn, reason = "Useless const")]
    pub fn reset_distribution_id(&mut self) -> &mut Self {
        self.distribution_id = DistributionID::System;
        self
    }

    /// Returns the current distribution target.
    #[inline]
    #[must_use]
    pub const fn get_distribution_id(&self) -> DistributionID {
        self.distribution_id
    }

    /// Executes the command via the underlying WSL Plugin API.
    ///
    /// On success, returns a [`TcpStream`] connected to the process' stdin/stdout.
    ///
    /// # Behavior
    ///
    /// - `argv[0]` is computed as:
    ///   - overridden `arg0` if set,
    ///   - otherwise the program path string.
    /// - The full argv passed to the API is:
    ///   `argv[0]` + all user-provided args.
    /// - The selected execution method depends on [`DistributionID`].
    ///
    /// # Errors
    ///
    /// Returns an [`ApiError`] if the underlying API call fails.
    #[inline]
    pub fn execute(self) -> ApiResult<TcpStream> {
        let WSLCommand {
            api,
            session_id,
            distribution_id,
            path,
            arg0,
            args,
        } = self;

        let argv0 = arg0.as_deref().unwrap_or(path.as_str());
        let all_args = once(argv0).chain(args.iter().map(AsRef::as_ref));

        match distribution_id {
            DistributionID::System => api
                .execute_binary(session_id, path.as_ref(), all_args)
                .map_err(Into::into),
            DistributionID::User(id) => {
                api.execute_binary_in_distribution(session_id, id, path.as_ref(), all_args)
            }
        }
    }
}
