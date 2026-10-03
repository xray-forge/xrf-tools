use glam::{Vec3, Vec4};
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::{SectorSurface, VisualClusters, VisualDrawRange, VisualGeometry, VisualPackage};

use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_model_part::StaticModelPart;
use crate::scene::static_scene::static_model_skin::StaticModelSkin;
use crate::scene::static_scene::static_vertex_words::pack_model_words;

/// Bones a skinned vertex hangs from.
const LINKS: usize = 4;

/// How far past its declared sphere a skinned model is taken to reach, for its clusters: it moves with its bones, so
/// its clusters are culled by the model's whole sphere, widened, rather than by where they stand bound.
const SKINNED_REACH: f32 = 2.0;

/// A visual packed on a loader thread for the static scene: its submeshes' vertices end to end in the model layout,
/// the indices of each one's chosen level moved past the vertices before it, a part a submesh, its declared sphere,
/// which every object standing as it is culled and discarded by, and its skin where it moves with bones.
#[derive(Clone, Debug)]
pub struct StaticModel {
  pub words: Vec<u32>,
  pub indices: Vec<u32>,
  pub parts: Vec<StaticModelPart>,
  /// In its own space.
  pub sphere: Vec4,
  /// Its box's least and greatest corners in its own space, as the visual declares it.
  pub bounds: [Vec3; 2],
  /// How its vertices hang from its bones, for a visual still carrying its skin; `None` for one posed or rigid.
  pub skin: Option<StaticModelSkin>,
}

impl StaticModel {
  /// A visual packed with each submesh at `detail` down its collapse chain, zero its finest level and one its coarsest,
  /// dressed by its descriptor; `color_id` picks the flat colour drawn without textures. A submesh that did not pack is
  /// left out; one still carrying its skin keeps it, and is cut into clusters its whole model's sphere culls.
  pub fn pack_at(package: &VisualPackage, descriptors: &[XraySurfaceDescriptor], color_id: u16, detail: f32) -> Self {
    let declared_box = &package.description.declared_bounds.bounding_box;
    let sphere = &package.description.declared_bounds.bounding_sphere;
    let sphere: Vec4 = Vec4::new(sphere.center.x, sphere.center.y, sphere.center.z, sphere.radius);
    let is_skinned: bool = package
      .description
      .submeshes
      .iter()
      .any(|submesh| submesh.geometry().is_some_and(|geometry| geometry.skin.is_some()));
    let mut words: Vec<u32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut links: Vec<u32> = Vec::new();
    let mut parts: Vec<StaticModelPart> = Vec::new();

    for (index, submesh) in package.description.submeshes.iter().enumerate() {
      let Some(geometry) = submesh.geometry() else {
        continue;
      };
      let vertex_base: u32 = (words.len() / StaticLayout::STRIDE as usize) as u32;
      let index_base: u32 = indices.len() as u32;
      let level: VisualDrawRange = to_level(geometry, detail);
      let stored: Vec<u16> = read_pods(&package.buffer, &geometry.indices);
      let packed: Vec<u32> = pack_model_words(geometry, &package.buffer);
      let vertices: usize = packed.len() / StaticLayout::STRIDE as usize;

      if is_skinned {
        links.extend(pack_links(geometry, &package.buffer, vertices));
      }

      words.extend(packed);
      indices.extend(
        stored
          .iter()
          .skip(level.start as usize)
          .take(level.count as usize)
          .map(|it| u32::from(*it) + vertex_base),
      );
      parts.push(StaticModelPart {
        clusters: if geometry.clusters.is_some() && !is_skinned {
          to_clusters(geometry, &package.buffer, (level.start, level.count), index_base)
        } else {
          to_reaching_clusters(level.count, index_base, sphere)
        },
        surface: SectorSurface {
          shader_id: color_id,
          shader_name: submesh.shader_name.clone(),
          texture_name: submesh.texture_name.clone(),
          hemi: None,
        },
        descriptor: descriptors.get(index).cloned(),
      });
    }

    Self {
      words,
      indices,
      parts,
      sphere,
      bounds: [
        Vec3::new(declared_box.min.x, declared_box.min.y, declared_box.min.z),
        Vec3::new(declared_box.max.x, declared_box.max.y, declared_box.max.z),
      ],
      skin: is_skinned.then_some(StaticModelSkin {
        links,
        bones: package.description.bones.len() as u32,
      }),
    }
  }
}

/// The level `detail` picks down a geometry's collapse chain, as the viewers pick it: rounded to the nearest.
fn to_level(geometry: &VisualGeometry, detail: f32) -> VisualDrawRange {
  let coarsest: usize = geometry.detail_levels.len().saturating_sub(1);
  let chosen: usize = (detail.clamp(0.0, 1.0) * coarsest as f32).round() as usize;

  geometry
    .detail_levels
    .get(chosen)
    .cloned()
    .unwrap_or_else(|| geometry.get_default_level())
}

/// Each vertex's links as two words: its four bones' indices as bytes, then their weights as bytes. A geometry without
/// a skin hangs from nothing, which its vertices' zero weights say.
fn pack_links(geometry: &VisualGeometry, buffer: &[u8], vertices: usize) -> Vec<u32> {
  let Some(skin) = &geometry.skin else {
    return vec![0; vertices * 2];
  };
  let bones: Vec<u16> = read_pods(buffer, &skin.indices);
  let weights: Vec<f32> = read_pods(buffer, &skin.weights);

  (0..vertices)
    .flat_map(|vertex| {
      let link = |at: usize| -> (u32, u32) {
        let bone: u32 = bones.get(vertex * LINKS + at).copied().map_or(0, u32::from).min(255);
        let weight: f32 = weights.get(vertex * LINKS + at).copied().unwrap_or(0.0);

        (bone, (weight.clamp(0.0, 1.0) * 255.0).round() as u32)
      };
      let [a, b, c, d] = [link(0), link(1), link(2), link(3)];

      [
        a.0 | (b.0 << 8) | (c.0 << 16) | (d.0 << 24),
        a.1 | (b.1 << 8) | (c.1 << 16) | (d.1 << 24),
      ]
    })
    .collect()
}

/// The clusters a geometry drawn as stored was cut into within its level, their first index moved to where the
/// level's indices now start.
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

/// A level's indices cut into clusters in order, each culled by the model's whole sphere widened: what a geometry
/// that moves, or was never cut, is drawn by.
fn to_reaching_clusters(count: u32, base: u32, sphere: Vec4) -> Vec<(u32, u32, Vec4)> {
  let reach: Vec4 = sphere.truncate().extend(sphere.w * SKINNED_REACH);
  let triangles: u32 = count / 3;

  (0..triangles.div_ceil(VisualClusters::MAX_TRIANGLES))
    .map(|cluster| {
      let first: u32 = cluster * VisualClusters::MAX_TRIANGLES;

      (
        base + first * 3,
        (triangles - first).min(VisualClusters::MAX_TRIANGLES),
        reach,
      )
    })
    .collect()
}
