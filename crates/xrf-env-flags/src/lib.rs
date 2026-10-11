//! Every environment variable the workspace reads, in one table that parses each the same way and generates its
//! document.

mod env_flag;
mod env_flag_kind;
mod env_flag_reading;
mod env_flag_value;
mod env_flags;

#[cfg(test)]
mod tests;

pub use crate::env_flag::EnvFlag;
pub use crate::env_flag_kind::EnvFlagKind;
pub use crate::env_flag_reading::EnvFlagReading;
pub use crate::env_flag_value::EnvFlagValue;
pub use crate::env_flags::EnvFlags;
