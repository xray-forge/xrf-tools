use crate::definition::thunderbolt::thunderbolt_key::ThunderboltKey;
use crate::section::EnvironmentSection;

/// A `thunderbolts.ltx` section: one bolt a collection strikes with.
pub type Thunderbolt = EnvironmentSection<ThunderboltKey>;
