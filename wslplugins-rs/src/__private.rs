#[cfg(feature = "tracing")]
pub use tracing::{debug, error, info, trace, warn};
#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! debug {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! error {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! info {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! trace {
    ($($tt:tt)*) => {};
}
#[cfg(not(feature = "tracing"))]
#[macro_export]
macro_rules! warn {
    ($($tt:tt)*) => {};
}
