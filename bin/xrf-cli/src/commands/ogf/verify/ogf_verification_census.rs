use std::collections::BTreeMap;

use serde::Serialize;

/// What a sweep counted, beside what it found.
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfVerificationCensus {
  pub files: usize,
  pub unreadable_files: usize,
  pub files_without_geometry: usize,
  pub submeshes: usize,
  pub packed_submeshes: usize,
  pub unsupported_submeshes: usize,
  pub malformed_submeshes: usize,
  pub progressive_submeshes: usize,
  pub progressive_submeshes_drawing_part_of_the_buffer: usize,
  pub skinned_submeshes: usize,
  /// Vertices whose four weights do not sum to one, which a renderer would draw pulled toward the origin.
  pub vertices_with_stray_skin_weights: usize,
  pub bounds_disagreements: usize,
  pub versions: BTreeMap<u8, usize>,
  pub root_model_types: BTreeMap<String, usize>,
  pub submesh_model_types: BTreeMap<String, usize>,
  pub vertex_formats: BTreeMap<String, usize>,
  pub visuals_without_root: usize,
  pub texture_references: usize,
  pub resolved_texture_references: usize,
  /// References that resolved inside an archive, which the sweep locates without reading a header.
  pub located_texture_references: usize,
  /// Which search step answered each resolved reference, so an overlay shows what it still takes from its base.
  pub texture_steps: BTreeMap<String, usize>,
  pub missing_texture_references: usize,
  pub unreadable_textures: usize,
  pub distinct_textures: usize,
  pub textures_without_mipmaps: usize,
  pub texture_formats: BTreeMap<String, usize>,
  pub texture_sizes: BTreeMap<String, usize>,
}

impl OgfVerificationCensus {
  /// Counts one more `key` in `map`.
  pub fn count(map: &mut BTreeMap<String, usize>, key: impl Into<String>) {
    *map.entry(key.into()).or_default() += 1;
  }
}
