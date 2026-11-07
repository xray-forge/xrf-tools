use crate::contract::feature::scene::render_foliage_settings::RenderFoliageSettings;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::scene::wind::foliage_wind_values::FoliageWindValues;
use crate::feature::scene::wind::wind_uniform::WindUniform;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::view::view_restart::ViewRestart;
use crate::lighting::weather::render_lighting::RenderLighting;
use crate::lighting::wind::foliage_motion::FoliageMotion;
use crate::lighting::wind::foliage_wind::FoliageWind;
use crate::scene::level::view::view_time::ViewTime;

/// The sway amplitude the shadows are redrawn at while the enhanced foliage motion blows: its largest move a metre of
/// reach, a tall crown's trunk swing over its height.
const FOLIAGE_SHADOW_AMPLITUDE: f32 = 0.02;

/// How the wind moves the view's trees and grass: the enhanced foliage motion stepped from frame to frame, the trees'
/// sway following the last frame's, which every static draw reads, and the amplitude the shadows sway by.
#[derive(Default)]
pub struct WindFeature {
  foliage: FoliageWind,
  values: FoliageWindValues,
  uniform: WindUniform,
  /// The last frame's sway, which this frame's follows.
  last: Option<WindUniform>,
  /// The wind's amplitude: what has still shadows drawn again.
  sway_amplitude: f32,
}

impl WindFeature {
  /// Steps the foliage motion to `time` in the weather's wind and rain, and takes this frame's sway.
  pub fn prepare(&mut self, options: &RenderViewOptions, lighting: &RenderLighting, time: ViewTime) {
    let settings: RenderFoliageSettings = options.features.grass.foliage;
    let foliage: FoliageWindValues = self.foliage.advance(
      time,
      &settings,
      lighting.wind,
      FoliageMotion::of(lighting.trees.is_some()),
      lighting.rain.map_or(0.0, |rain| rain.density.clamp(0.0, 1.0)),
    );
    let wind: WindUniform = WindUniform::new(lighting.trees.as_ref(), time.now)
      .following(self.last.as_ref())
      .with_foliage(&foliage);

    self.last = Some(wind);
    self.values = foliage;
    self.uniform = wind;
    // The enhanced motion moves a tree by more than the engine's lean for a given reach, so its shadows redraw at the
    // enhanced reach whenever it blows.
    self.sway_amplitude = if settings.is_enhanced() && foliage.wind.z > 0.0 {
      FOLIAGE_SHADOW_AMPLITUDE
    } else {
      wind.get_amplitude()
    };
  }

  /// How the trees sway this frame, which every static draw reads.
  pub fn get_uniform(&self) -> &WindUniform {
    &self.uniform
  }

  /// What the foliage motion reads this frame, which the grass takes as the trees do.
  pub fn get_values(&self) -> &FoliageWindValues {
    &self.values
  }

  /// The wind's amplitude, which a tree's reach is multiplied by to give how far it leans.
  pub fn get_sway_amplitude(&self) -> f32 {
    self.sway_amplitude
  }
}

impl ViewFeature for WindFeature {
  /// Nothing: the wind is values a frame.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}

  /// Its motion restarted, the foliage's drift and the trees' sway start again from nothing, as at a view's first
  /// frame; a cut leaves them, which follow the time and not the camera.
  fn restart(&mut self, restart: ViewRestart) {
    if restart == ViewRestart::Motion {
      *self = Self::default();
    }
  }
}
