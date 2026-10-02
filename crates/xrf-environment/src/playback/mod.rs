//! A weather played as the engine plays it: the pair of keyframes it blends from frame to frame, and an effect laid
//! over the cycle.

pub(crate) mod weather_effect_start;
pub(crate) mod weather_effect_timeline;
pub(crate) mod weather_pair;
pub(crate) mod weather_played_keyframe;

pub use crate::playback::weather_effect_start::WeatherEffectStart;
pub use crate::playback::weather_effect_timeline::WeatherEffectTimeline;
pub use crate::playback::weather_pair::WeatherPair;
pub use crate::playback::weather_played_keyframe::WeatherPlayedKeyframe;
