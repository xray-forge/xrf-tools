use std::collections::HashMap;

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
  /// Every texture the shader table names, resolved once so a sector arriving later is a lookup rather than a search.
  pub textures: Vec<LevelTextureReference>,
  /// How the renderer draws each shader the table names, by shader name, so a surface is cut out or blended the way
  /// its blender says rather than drawn solid.
  pub surfaces: HashMap<String, XraySurfaceDescriptor>,
  /// Extent every sector together covers, which is where a camera is framed from.
  pub bounds: Option<VisualBounds>,
}
