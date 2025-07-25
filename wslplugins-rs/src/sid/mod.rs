mod security_identifier;
mod sid;
pub use security_identifier::SecurityIdentifier;
pub use sid::Sid;
use sid::{SID_HEAD_ALIGN, SID_HEAD_SIZE};
pub(super) mod utils;
use security_identifier::SidSizeInfo;
mod sid_identifier_authority;
#[cfg(test)]
pub(crate) use security_identifier::test::arb_security_identifier;
#[cfg(test)]
pub(crate) use sid_identifier_authority::test::arb_identifier_authority;
pub use sid_identifier_authority::SidIdentifierAuthority;
