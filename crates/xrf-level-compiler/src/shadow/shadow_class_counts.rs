use serde::{Deserialize, Serialize};

/// How many ray model faces are of each [`ShadowClass`](crate::shadow::shadow_class::ShadowClass).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShadowClassCounts {
  pub passes: usize,
  pub alpha: usize,
  pub opaque: usize,
}
