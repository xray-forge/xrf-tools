use crate::section::EnvironmentSection;
use crate::sun::lens_flare_key::LensFlareKey;

/// A `suns.ltx` section: the sun's sprite, flares and gradient a keyframe names by `sun`.
pub type LensFlare = EnvironmentSection<LensFlareKey>;
