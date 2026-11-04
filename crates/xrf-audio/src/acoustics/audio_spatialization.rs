use serde::{Deserialize, Serialize};

/// How placed sounds reach the listener's ears: through a head-related transfer function, which places a sound around
/// and above on headphones, or panned between two speakers. Dry sound always pans.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioSpatialization {
  #[default]
  Headphones,
  Speakers,
}
