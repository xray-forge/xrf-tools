use crate::contract::render_foliage_settings::RenderFoliageSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::foliage_wind::FoliageWind;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::wind_uniform::WindUniform;

/// The sway amplitude the shadows are redrawn at while the enhanced foliage motion blows: its largest move a metre of
/// reach, a tall crown's trunk swing over its height.
const FOLIAGE_SHADOW_AMPLITUDE: f32 = 0.02;

/// How the wind moves the view's trees and grass: the enhanced foliage motion stepped from frame to frame, the trees'
/// sway following the last frame's, which every static draw reads, and the amplitude and time the shadows sway by.
#[derive(Default)]
pub struct WindFeature {
  foliage: FoliageWind,
  values: FoliageWindValues,
  uniform: WindUniform,
  /// The last frame's sway, which this frame's follows.
  last: Option<WindUniform>,
  /// The wind's amplitude and the time the trees sway by: what has still shadows drawn again.
  sway: (f32, f32),
}

impl WindFeature {
  /// Steps the foliage motion to `time` in the weather's wind and rain, and takes this frame's sway.
  pub fn prepare(&mut self, options: &RenderViewOptions, lighting: &RenderLighting, time: f32) {
    let settings: RenderFoliageSettings = options.features.grass.foliage;
    let foliage: FoliageWindValues = self.foliage.advance(
      time,
      &settings,
      (lighting.wind, lighting.trees.is_some()),
      lighting.rain.map_or(0.0, |rain| rain.density.clamp(0.0, 1.0)),
    );
    let wind: WindUniform = WindUniform::new(lighting.trees.as_ref(), time)
      .following(self.last.as_ref())
      .with_foliage(&foliage);

    self.last = Some(wind);
    self.values = foliage;
    self.uniform = wind;
    // The enhanced motion moves a tree by more than the engine's lean for a given reach, so its shadows redraw at the
    // enhanced reach whenever it blows.
    self.sway = (
      if settings.is_enhanced() && foliage.wind.z > 0.0 {
        FOLIAGE_SHADOW_AMPLITUDE
      } else {
        wind.get_amplitude()
      },
      time,
    );
  }

  /// How the trees sway this frame, which every static draw reads.
  pub fn get_uniform(&self) -> &WindUniform {
    &self.uniform
  }

  /// What the foliage motion reads this frame, which the grass takes as the trees do.
  pub fn get_values(&self) -> &FoliageWindValues {
    &self.values
  }

  /// The wind's amplitude and the time the trees sway by.
  pub fn get_sway(&self) -> (f32, f32) {
    self.sway
  }
}
