use thiserror::Error;

/// Errors returned when parsing a [`super::WSLVersion`] from text.
#[allow(
    clippy::enum_variant_names,
    reason = "These names are more descriptive and easier to understand than the alternatives."
)]
#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum WSLVersionParseError {
    /// The version string does not follow the supported `major.minor` or
    /// `major.minor.revision` shape.
    #[error(
        "invalid WSL version format: expected 'major.minor' or 'major.minor.revision', got '{input}'"
    )]
    InvalidFormat { input: String },

    /// The major component is not a valid `u32`.
    #[error("invalid WSL version major component in '{input}'")]
    InvalidMajor { input: String },

    /// The minor component is not a valid `u32`.
    #[error("invalid WSL version minor component in '{input}'")]
    InvalidMinor { input: String },

    /// The revision component is not a valid `u32`.
    #[error("invalid WSL version revision component in '{input}'")]
    InvalidRevision { input: String },
}
