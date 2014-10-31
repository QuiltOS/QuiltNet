#[macro_export]
macro_rules! debug(
  ( $($arg:tt) *) => (())
)

#[macro_export]
macro_rules! log_enabled(
  ($arg:expr) => (false)
)
