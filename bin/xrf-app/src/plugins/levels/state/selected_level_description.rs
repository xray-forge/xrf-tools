use serde::Serialize;
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::{SectorOutline, VisualBounds};

use crate::plugins::levels::state::level_source::LevelSource;
use crate::plugins::levels::state::level_texture_reference::LevelTextureReference;

/// What the viewer is showing, paired with where it came from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedLevelDescription {
  pub source: LevelSource,
  pub xrlc_version: u16,
  pub xrlc_quality: u16,
  pub visuals: u32,
  pub drawables: u32,
  pub shader_entries: u32,
  pub portals: u32,
  pub lights: u32,
  pub has_sun: bool,
  pub sectors: Vec<SectorOutline>,
  /// Every texture the level's surfaces bind, resolved once so a sector arriving later is a lookup rather than a
  /// search.
  pub textures: Vec<LevelTextureReference>,
  /// How the renderer draws each entry of the shader table, in its order, so a surface is cut out, blended and
  /// detailed the way its blender says rather than drawn flat. Indexed by the shader id a packed surface carries.
  pub surfaces: Vec<XraySurfaceDescriptor>,
  /// Extent every sector together covers, which is where a camera is framed from.
  pub bounds: Option<VisualBounds>,
}
