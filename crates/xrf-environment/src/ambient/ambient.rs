use crate::ambient::ambient_key::AmbientKey;
use crate::section::EnvironmentSection;

/// An `ambients.ltx` section: the sounds and particles a keyframe names by `ambient`.
pub type Ambient = EnvironmentSection<AmbientKey>;
