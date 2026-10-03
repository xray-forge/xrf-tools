use glam::Vec4;
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::{SectorSurface, VisualGeometry, VisualPackage};

use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_model_part::StaticModelPart;
use crate::scene::static_scene::static_vertex_words::pack_model_words;

/// A spawned visual packed on a loader thread for the static scene: its posed submeshes' vertices end to end in the
/// model layout, the indices of each one's finest level moved past the vertices before it, a part a submesh, and its
/// declared sphere, which every object standing as it is culled and discarded by.
#[derive(Clone, Debug)]
pub struct StaticModel {
  pub name: String,
  pub words: Vec<u32>,
  pub indices: Vec<u32>,
  pub parts: Vec<StaticModelPart>,
  /// In its own space.
  pub sphere: Vec4,
}

impl StaticModel {
  /// A posed visual packed, each submesh dressed by its descriptor; `color_id` picks the flat colour drawn without
  /// textures. A submesh that did not pack is left out.
  pub fn pack(name: &str, package: &VisualPackage, descriptors: &[XraySurfaceDescriptor], color_id: u16) -> Self {
    let mut words: Vec<u32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut parts: Vec<StaticModelPart> = Vec::new();

    for (index, submesh) in package.description.submeshes.iter().enumerate() {
      let Some(geometry) = submesh.geometry() else {
        continue;
      };
      let vertex_base: u32 = (words.len() / StaticLayout::STRIDE as usize) as u32;
      let index_base: u32 = indices.len() as u32;
      let level = geometry.get_default_level();
      let stored: Vec<u16> = read_pods(&package.buffer, &geometry.indices);

      words.extend(pack_model_words(geometry, &package.buffer));
      indices.extend(
        stored
          .iter()
          .skip(level.start as usize)
          .take(level.count as usize)
          .map(|it| u32::from(*it) + vertex_base),
      );
      parts.push(StaticModelPart {
        clusters: to_clusters(geometry, &package.buffer, (level.start, level.count), index_base),
        surface: SectorSurface {
          shader_id: color_id,
          shader_name: submesh.shader_name.clone(),
          texture_name: submesh.texture_name.clone(),
          hemi: None,
        },
        descriptor: descriptors.get(index).cloned(),
      });
    }

    let sphere = &package.description.declared_bounds.bounding_sphere;

    Self {
      name: name.to_owned(),
      words,
      indices,
      parts,
      sphere: Vec4::new(sphere.center.x, sphere.center.y, sphere.center.z, sphere.radius),
    }
  }
}

/// The clusters a posed geometry was cut into within its finest level, their first index moved to where the level's
/// indices now start.
fn to_clusters(
  geometry: &VisualGeometry,
  buffer: &[u8],
  (start, count): (u32, u32),
  base: u32,
) -> Vec<(u32, u32, Vec4)> {
  let Some(clusters) = &geometry.clusters else {
    return Vec::new();
  };
  let ranges: Vec<[u32; 4]> = read_pods(buffer, &clusters.ranges);
  let spheres: Vec<Vec4> = read_pods(buffer, &clusters.spheres);

  ranges
    .iter()
    .zip(spheres)
    .filter(|(range, _)| range[0] >= start && range[0] < start + count)
    .map(|(range, sphere)| (range[0] - start + base, range[1], sphere))
    .collect()
}
