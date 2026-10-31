use glam::{Vec3, Vec4};

use crate::contract::render_display_transform::RenderDisplayTransform;
use crate::contract::render_grading_settings::RenderGradingSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::display_curves::{agx, get_graded, get_key_match, get_least_shown, pbr_neutral};
use crate::pass::exposure_metering::ExposureMetering;

/// How the window pass shows a viewport's scene, numbered as `common/display_transform.wgsl` numbers it: as it is (the
/// engine's passes tonemapped it), by the engine's curve (linear light at the engine's display values), or, where the
/// light is physical, by a display transform and encoded for the display.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisplayMode {
  AsIs = 0,
  Tonemapped = 1,
  EngineEncoded = 2,
  Agx = 3,
  PbrNeutral = 4,
}

impl DisplayMode {
  /// The land's colour a transform is anchored on, a warm earth: AgX's response depends on the hue it is shown, so the
  /// anchor matches the land as it is rather than a grey.
  pub const ANCHOR_HUE: Vec3 = Vec3::new(1.0, 0.75, 0.45);
  /// The keys an anchor is solved within: from the least the engine's curve shows above the display's first level,
  /// `(1/255)^2.2`, under which there is nothing shown to match, or the least the transform itself shows there
  /// (`get_least_shown`), under which its toe would be lifted from black and whatever the night lights above the key
  /// with it, into the shoulder (AgX's moonlit grass bleached grey); to the display's white, past which the engine's
  /// curve shows what AgX and PBR Neutral, held under white, reach only at their bounds.
  const ANCHOR_KEYS: (f32, f32) = (5.07e-6, 1.0);
  /// The most a transform's anchor lifts or lowers the light by, in stops.
  const ANCHOR_BOUNDS: (f32, f32) = (-6.0, 8.0);
  /// The stops under the least the transform shows over which a key gives up its anchor for its bright end's.
  const BRIGHT_FADE_STOPS: f32 = 2.0;

  /// The mode a view's options ask for.
  pub fn of(options: &RenderViewOptions) -> Self {
    if !options.is_scene_linear() {
      return Self::AsIs;
    }

    if !options.is_light_physical() {
      return Self::Tonemapped;
    }

    match options.features.display_transform {
      RenderDisplayTransform::Engine => Self::EngineEncoded,
      RenderDisplayTransform::Agx => Self::Agx,
      RenderDisplayTransform::PbrNeutral => Self::PbrNeutral,
    }
  }

  /// The number the shaders tell it by.
  pub fn get_index(self) -> u32 {
    self as u32
  }

  /// How the shaders show a frame by it (`shown_for_display` in `common/display_transform`, and its inverse for the
  /// game's pictures): its number, `grading`'s contrast and saturation, and its lift for the frame's exposed key and
  /// bright end as `metering` read them (`get_lift`). The present and the lighting take it alike, so a picture entering
  /// the scene through the inverse is shown by the very display it was inverted for.
  pub fn get_display(self, grading: &RenderGradingSettings, metering: &ExposureMetering) -> Vec4 {
    Vec4::new(
      self.get_index() as f32,
      grading.contrast,
      grading.saturation,
      self.get_lift(metering.key, metering.bright, grading.contrast),
    )
  }

  /// The stops the physical light is lifted by after its grading, before the transform: each transform's anchor, which
  /// shows the land's colour (`ANCHOR_HUE`) at the frame's exposed `key` (`ExposureHead::key`), graded by `contrast`,
  /// at the display value the engine's curve shows it at, and nothing else of the engine's look, so each keeps its own
  /// contrast about it. None for the engine's curve and the engine's own showing.
  pub fn get_anchor_stops(self, key: f32, contrast: f32) -> f32 {
    self.get_bounded(key, None, contrast)
  }

  /// The stops the frame is lifted by: the anchor at its exposed `key` (`get_anchor_stops`), and, as the key falls under
  /// the least the transform shows, never more than shows its exposed `bright` end (`ExposureHead::bright`) as bright
  /// as the engine's curve does. A night's land under AgX's black is held at it while what the moon lights rides the
  /// lift, through AgX's steeper middle, into its shoulder; matched there, the bright end shows as the engine's and the
  /// land, which AgX cannot show as the engine does, sinks towards its black. Over the transform's range the key rules
  /// alone: a dusk's bright end is its fires and glints, which would sink the land with them.
  pub fn get_lift(self, key: f32, bright: Option<f32>, contrast: f32) -> f32 {
    self.get_bounded(key, bright, contrast)
  }

  /// The transform the physical light is shown through; none for the engine's curve and the engine's own showing.
  fn get_transform(self) -> Option<fn(Vec3) -> Vec3> {
    match self {
      Self::Agx => Some(agx),
      Self::PbrNeutral => Some(pbr_neutral),
      Self::AsIs | Self::Tonemapped | Self::EngineEncoded => None,
    }
  }

  /// The match at `key`, bounded by the match at `bright` as far as the graded key stands under the least the transform
  /// shows, all of it `BRIGHT_FADE_STOPS` under.
  fn get_bounded(self, key: f32, bright: Option<f32>, contrast: f32) -> f32 {
    let Some(transform) = self.get_transform() else {
      return 0.0;
    };
    let least: f32 = get_least_shown(transform, Self::ANCHOR_HUE);
    let lift: f32 = Self::get_match(transform, least, key, contrast);
    let Some(bright) = bright else {
      return lift;
    };
    let under: f32 = (least / get_graded(key, contrast).max(f32::MIN_POSITIVE)).log2();
    let bounded: f32 = lift.min(Self::get_match(transform, least, bright, contrast));

    lift + (bounded - lift) * (under / Self::BRIGHT_FADE_STOPS).clamp(0.0, 1.0)
  }

  /// The stops that show the land's colour (`ANCHOR_HUE`) at the exposed `key`, graded by `contrast`, through
  /// `transform` as the engine's curve shows it, the key held within `ANCHOR_KEYS` and over the `least` the transform
  /// shows.
  fn get_match(transform: fn(Vec3) -> Vec3, least: f32, key: f32, contrast: f32) -> f32 {
    let (lowest, most): (f32, f32) = Self::ANCHOR_KEYS;
    let shown: f32 = get_graded(key, contrast).clamp(lowest.max(least), most);

    get_key_match(transform, (shown, Self::ANCHOR_HUE), Self::ANCHOR_BOUNDS)
  }
}
