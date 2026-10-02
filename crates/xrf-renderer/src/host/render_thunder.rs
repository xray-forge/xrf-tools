use std::collections::HashMap;

use xrf_visual::LightAnimatorDescription;

use crate::host::render_thunder_settings::RenderThunderSettings;
use crate::host::render_thunderbolt::RenderThunderbolt;
use crate::host::render_thunderbolt_model::RenderThunderboltModel;

/// What a level's weather strikes with (`CEffect_Thunderbolt`): its collections, their bolts, and where bolts strike.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderThunder {
  /// Where bolts strike, or none for a game that says nowhere, which strikes nothing.
  pub settings: Option<RenderThunderSettings>,
  /// Every collection by name, its bolts by name in the order written.
  pub collections: HashMap<String, Vec<String>>,
  /// Every bolt a collection names that the game has, by name.
  pub bolts: HashMap<String, RenderThunderbolt>,
  pub models: Vec<RenderThunderboltModel>,
  pub animators: Vec<LightAnimatorDescription>,
}
