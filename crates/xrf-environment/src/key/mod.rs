//! The vocabulary every section kind is read with: what a key holds, how the engine parses it, and what it defaults to.

pub(crate) mod environment_default;
pub(crate) mod environment_key;
pub(crate) mod environment_key_use;
pub(crate) mod environment_value;
pub(crate) mod environment_value_kind;

pub use crate::key::environment_default::EnvironmentDefault;
pub use crate::key::environment_key::EnvironmentKey;
pub(crate) use crate::key::environment_key::declare_environment_keys;
pub use crate::key::environment_key_use::EnvironmentKeyUse;
pub use crate::key::environment_value::EnvironmentValue;
pub use crate::key::environment_value_kind::EnvironmentValueKind;
