use crate::section::EnvironmentSection;
use crate::thunderbolt::thunderbolt_key::ThunderboltKey;

/// A `thunderbolts.ltx` section: one bolt a collection strikes with.
pub type Thunderbolt = EnvironmentSection<ThunderboltKey>;
