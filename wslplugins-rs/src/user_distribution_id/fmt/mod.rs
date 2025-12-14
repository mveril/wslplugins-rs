mod formatter;
mod guid_formatter;
pub use guid_formatter::GuidFormatter;
#[cfg(feature = "uuid")]
mod uuid_formatter;
#[cfg(feature = "uuid")]
pub use uuid_formatter::UuidFormatter;
#[cfg(not(feature = "uuid"))]
pub(crate) type DefaultFormatter = guid_formatter::GuidFormatter;
#[cfg(feature = "uuid")]
pub(crate) type DefaultFormatter = uuid_formatter::UuidFormatter;
