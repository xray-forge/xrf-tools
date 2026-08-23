//! Synthetic visuals built in code, so the bytes a test reasons about are visible in the test.
//!
//! Shapes here mirror what walking the reference trees measured: a loose visual is a version 4
//! skeleton whose children hold the geometry, and a progressive child's finest detail level sits at a
//! non zero offset rather than covering the whole index buffer.

use xrf_db::{
  OgfBonesChunk, OgfBox, OgfChildrenChunk, OgfDescriptionChunk, OgfFile, OgfGeometry, OgfHeaderChunk,
  OgfKinematicsChunk, OgfMotion, OgfSlideWindow, OgfSphere, OgfSwiDataChunk, OgfTextureChunk, OgfVertex, OgfVertexLink,
  OmfMotionsChunk, Vector3d,
};

pub(crate) const MODEL_TYPE_SKELETON_ANIM: u8 = 3;
pub(crate) const MODEL_TYPE_GEOMDEF_PM: u8 = 4;
pub(crate) const MODEL_TYPE_GEOMDEF_ST: u8 = 5;

/// One bone link, the vertex layout every loose visual in the reference trees was measured to use.
pub(crate) const VERTEX_FORMAT_1_LINK: u32 = 0x1207_1980;

pub(crate) fn vector(x: f32, y: f32, z: f32) -> Vector3d {
  Vector3d { x, y, z }
}

/// A vertex whose position, normal and uv are distinguishable, so a conversion that dropped or
/// duplicated one of them fails rather than passing on symmetry.
pub(crate) fn vertex(position: Vector3d, normal: Vector3d, u: f32, v: f32) -> OgfVertex {
  OgfVertex {
    position,
    normal,
    tangent: vector(1.0, 0.0, 0.0),
    binormal: vector(0.0, 0.0, 1.0),
    texture_u: u,
    texture_v: v,
    links: vec![OgfVertexLink { bone: 0, weight: 1.0 }],
  }
}

pub(crate) fn geometry(vertices: Vec<OgfVertex>, indices: Vec<u16>) -> OgfGeometry {
  OgfGeometry {
    vertex_count: Some(vertices.len() as u32),
    indices: Some(indices),
    skin_bone_indices: vec![0],
    vertex_format: Some(VERTEX_FORMAT_1_LINK),
    vertices: Some(vertices),
  }
}

/// Geometry whose vertex chunk was read but whose layout the decoder does not know.
pub(crate) fn geometry_of_unknown_format(format: u32, indices: Vec<u16>) -> OgfGeometry {
  OgfGeometry {
    vertex_count: Some(3),
    indices: Some(indices),
    skin_bone_indices: vec![],
    vertex_format: Some(format),
    vertices: None,
  }
}

pub(crate) fn window(offset: u32, num_tris: u16, num_verts: u16) -> OgfSlideWindow {
  OgfSlideWindow {
    offset,
    num_tris,
    num_verts,
  }
}

pub(crate) fn swi(windows: Vec<OgfSlideWindow>) -> OgfSwiDataChunk {
  OgfSwiDataChunk {
    reserved: [0; 4],
    windows,
  }
}

/// A visual carrying nothing but a header, to be filled in by the caller.
pub(crate) fn visual(model_type: u8) -> OgfFile {
  OgfFile {
    header: OgfHeaderChunk {
      version: 4,
      model_type,
      shader_id: 0,
      bounding_box: OgfBox {
        min: vector(-1.0, -2.0, -3.0),
        max: vector(4.0, 5.0, 6.0),
      },
      bounding_sphere: OgfSphere {
        position: vector(1.0, 2.0, 3.0),
        radius: 7.0,
      },
    },
    texture: None,
    geometry: None,
    bones: None,
    swi_data: None,
    children: None,
    description: None,
    kinematics: None,
    ik_data: None,
    user_data: None,
    lods: None,
    motions: None,
    motion_parameters: None,
  }
}

/// A skeleton whose children hold the geometry, which is how every loose visual is laid out.
pub(crate) fn skeleton(children: Vec<OgfFile>) -> OgfFile {
  OgfFile {
    children: Some(OgfChildrenChunk { nested: children }),
    ..visual(MODEL_TYPE_SKELETON_ANIM)
  }
}

pub(crate) fn textured(texture_name: &str, shader_name: &str, child: OgfFile) -> OgfFile {
  OgfFile {
    texture: Some(OgfTextureChunk {
      texture_name: String::from(texture_name),
      shader_name: String::from(shader_name),
    }),
    ..child
  }
}

/// A static child covering one triangle, the simplest thing that draws.
pub(crate) fn static_triangle_child() -> OgfFile {
  OgfFile {
    geometry: Some(geometry(
      vec![
        vertex(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 1.0), 0.0, 0.0),
        vertex(vector(1.0, 0.0, 0.0), vector(0.0, 1.0, 0.0), 1.0, 0.0),
        vertex(vector(0.0, 1.0, 0.0), vector(1.0, 0.0, 0.0), 0.0, 1.0),
      ],
      vec![0, 1, 2],
    )),
    ..visual(MODEL_TYPE_GEOMDEF_ST)
  }
}

/// Index buffer of the progressive child below: two coarse triangles, then the two fine ones.
pub(crate) const PROGRESSIVE_INDICES: [u16; 12] = [0, 1, 2, 0, 2, 3, 0, 1, 4, 1, 2, 5];

/// Where detail level zero starts in that buffer, and how many triangles it draws.
pub(crate) const PROGRESSIVE_FINE_OFFSET: u32 = 6;
pub(crate) const PROGRESSIVE_FINE_TRIANGLES: u16 = 2;

/// The one vertex only the coarse level reaches, placed far outside the rest so that measuring the
/// whole vertex buffer instead of the drawn range is a visible difference rather than a silent one.
const PROGRESSIVE_COARSE_ONLY_VERTEX: u16 = 3;

/// A progressive child whose finest detail level is the tail of its index buffer.
///
/// The measured case in `gamedata` has 17,850 indices with level zero at offset 14,982, so drawing the
/// whole buffer draws six times the triangles as stacked shells. This is that shape in miniature: the
/// leading six indices are the coarse level and must not be drawn.
pub(crate) fn progressive_child() -> OgfFile {
  progressive_child_with_windows(vec![
    window(PROGRESSIVE_FINE_OFFSET, PROGRESSIVE_FINE_TRIANGLES, 6),
    window(0, 4, 4),
  ])
}

/// The same progressive child with a detail table of the caller's choosing, for the levels that do not validate.
pub(crate) fn progressive_child_with_windows(windows: Vec<OgfSlideWindow>) -> OgfFile {
  OgfFile {
    geometry: Some(geometry(
      (0..6)
        .map(|index| {
          let offset: f32 = index as f32;
          let position: Vector3d = match index == PROGRESSIVE_COARSE_ONLY_VERTEX {
            true => vector(100.0, 100.0, 100.0),
            false => vector(offset, offset * 2.0, offset * 3.0),
          };

          vertex(position, vector(0.0, 0.0, 1.0), offset / 8.0, offset / 4.0)
        })
        .collect(),
      PROGRESSIVE_INDICES.to_vec(),
    )),
    swi_data: Some(swi(windows)),
    ..visual(MODEL_TYPE_GEOMDEF_PM)
  }
}

pub(crate) fn bones(names: &[(&str, &str)]) -> OgfBonesChunk {
  OgfBonesChunk {
    bones: names
      .iter()
      .map(|(name, parent)| xrf_db::OgfBone {
        name: String::from(*name),
        parent: String::from(*parent),
        rotation: (vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 0.0)),
        translate: vector(0.0, 0.0, 0.0),
        half_size: vector(0.0, 0.0, 0.0),
      })
      .collect(),
  }
}

pub(crate) fn kinematics(motion_refs: &[&str]) -> OgfKinematicsChunk {
  OgfKinematicsChunk {
    source_chunk_id: OgfKinematicsChunk::CHUNK_ID,
    motion_refs: motion_refs.iter().map(|it| String::from(*it)).collect(),
  }
}

pub(crate) fn embedded_motions(names: &[&str]) -> OmfMotionsChunk {
  OmfMotionsChunk {
    motions: names
      .iter()
      .map(|name| OgfMotion {
        name: String::from(*name),
        count: 0,
        flags: 0,
        remaining: vec![],
      })
      .collect(),
  }
}

pub(crate) fn description(source_file: &str) -> OgfDescriptionChunk {
  OgfDescriptionChunk {
    source_file: String::from(source_file),
    convertor: String::new(),
    built_at: 0,
    creator: String::new(),
    created_at: 0,
    editor: String::new(),
    edited_at: 0,
  }
}
