use std::fmt::{LowerHex, UpperHex};
#[allow(dead_code)]
pub trait Formatter: LowerHex + UpperHex {}
