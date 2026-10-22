use crate::capture::map_tile::MapTile;
use crate::contract::render_bloom_mode::RenderBloomMode;
use crate::contract::render_scale::RenderScale;
use crate::contract::render_show_flags::RenderShowFlags;
use crate::contract::render_view_features::RenderViewFeatures;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::render_lighting::RenderLighting;

/// How fast the metering view's exposure adapts, settling within its frames.
const METERING_ADAPTATION: f32 = 100.0;

/// Texels the sun's one cascade spans over a whole tile.
const SUN_CASCADE_RESOLUTION: u32 = 4096;

/// What a level map's tile is drawn with: the viewport's look, as `CDemoRecord`'s map draws the level, less what would
/// differ from one tile to the next.
///
/// What its parallel rays cannot draw is left out by the lens itself (`ViewLens`), as for any view drawn through them.
pub struct MapCaptureLook;

impl MapCaptureLook {
  /// The options a tile is drawn with, from the viewport's `options`.
  pub fn apply(options: &RenderViewOptions, tile: &MapTile) -> RenderViewOptions {
    let mut look: RenderViewOptions = options.clone();
    let show: &mut RenderShowFlags = &mut look.show;

    // Distance fog greys the low ground of a view from above, and particles stand about an eye a map has none of.
    show.is_fogged = false;
    show.is_particled = false;
    show.is_calibration_visible = false;

    let features: &mut RenderViewFeatures = &mut look.features;

    // An experiment: the grass planted over the whole tile, its tufts alike from edge to edge.
    features.grass.is_enabled = tile.is_grassed;
    // The bloom's blur reaches past a tile's margin, so neighbouring tiles would not meet.
    features.bloom.is_enabled = false;
    features.enhanced_bloom.mode = RenderBloomMode::Engine;
    features.shadows.resolution = SUN_CASCADE_RESOLUTION;

    look.output.upscaling.scale = RenderScale::Native;
    look.output.output_height = None;
    look.output.output_scale = 1.0;

    look
  }

  /// The options the metering view is drawn with: a tile's, its exposure adapting fast.
  pub fn apply_metering(options: &RenderViewOptions, tile: &MapTile) -> RenderViewOptions {
    let mut look: RenderViewOptions = Self::apply(options, tile);

    look.features.exposure.adaptation = METERING_ADAPTATION;

    look
  }

  /// The lighting a tile is drawn under: nothing falling, striking or swaying, so neighbouring tiles meet.
  pub fn apply_lighting(lighting: &RenderLighting) -> RenderLighting {
    RenderLighting {
      rain: None,
      thunderbolt: None,
      trees: None,
      ..lighting.clone()
    }
  }
}
