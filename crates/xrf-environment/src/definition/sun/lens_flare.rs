use crate::definition::sun::lens_flare_key::LensFlareKey;
use crate::section::EnvironmentSection;

/// A `suns.ltx` section: the sun's sprite, flares and gradient a keyframe names by `sun`.
pub type LensFlare = EnvironmentSection<LensFlareKey>;
