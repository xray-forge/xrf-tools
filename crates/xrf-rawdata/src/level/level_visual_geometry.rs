use byteorder::ByteOrder;
use xrf_chunk::ChunkDataSource;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelGeomSource, LevelShaderEntry, LevelShadersChunk, LevelVertex, LevelVisual};

use xrf_math::{Vector2d, Vector3d};

use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;

/// One visual of a compiled level, as the decompile takes it: what it is dressed with, and its full-detail triangles
/// over the vertices `level.geom` stores for it, in the space they are stored in, the level's for static geometry and a
/// tree's own until its placement is applied.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelVisualGeometry {
  pub texture: String,
  pub shader: String,
  /// The compiler shader and game material its surface takes: the editor's defaults as read, which the level's
  /// collision form and lighting may dress.
  pub dress: SurfaceDress,
  pub vertices: Vec<LevelVertex>,
  /// Three a triangle, into [`Self::vertices`].
  pub indices: Vec<u16>,
}

impl LevelVisualGeometry {
  /// Geometry of bare triangles, such as collision faces or occluder polygons, which keep nothing but their corners:
  /// each corner takes its triangle's own normal and texture coordinates spanning the texture once, in parts small
  /// enough for a visual's `u16` indices.
  pub(crate) fn of_triangles(
    texture: &str,
    shader: &str,
    dress: &SurfaceDress,
    triangles: &[[Vector3d; 3]],
  ) -> Vec<Self> {
    triangles
      .chunks(usize::from(u16::MAX) / 3)
      .map(|run| {
        let vertices: Vec<LevelVertex> = run.iter().flat_map(Self::get_bare_corners).collect();

        Self {
          texture: String::from(texture),
          shader: String::from(shader),
          dress: dress.clone(),
          indices: (0..vertices.len()).map(|index| index as u16).collect(),
          vertices,
        }
      })
      .collect()
  }

  /// A bare triangle's corners: its own normal at each, straight up where it has none.
  fn get_bare_corners(points: &[Vector3d; 3]) -> [LevelVertex; 3] {
    let [a, b, c] = points;
    let cross: Vector3d = (b - a).cross(&(c - a));
    let length: f32 = cross.length();
    let normal: Vector3d = if length.is_normal() {
      &cross / length
    } else {
      Vector3d::new(0.0, 1.0, 0.0)
    };
    let corner = |position: &Vector3d, u: f32, v: f32| LevelVertex {
      position: *position,
      normal,
      tangent: None,
      binormal: None,
      texture_coordinate: Vector2d::new(u, v),
      lightmap_coordinate: None,
      color: None,
      hemi: 0,
    };

    [corner(a, 0.0, 0.0), corner(b, 1.0, 0.0), corner(c, 0.0, 1.0)]
  }

  /// The geometry of `visual`: its texture and shader from its own texture chunk, else from the row of `shaders` its
  /// header names (its base texture, the first the row lists), and its vertices and full-detail indices from `source`
  /// (`LevelGeomSource::read_full_detail_indices`, which takes a progressive visual's first slide window). `None` for a
  /// visual drawing nothing from the buffers.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader row is not a shader reference, or the geometry or its window cannot be read.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(
    visual: &LevelVisual,
    shaders: &LevelShadersChunk,
    source: &LevelGeomSource<D>,
  ) -> XrfResult<Option<Self>> {
    let Some(container) = &visual.geometry else {
      return Ok(None);
    };
    let (texture, shader): (String, String) = match &visual.texture {
      Some(chunk) => (chunk.texture_name.clone(), chunk.shader_name.clone()),
      None => match shaders.entries.get(usize::from(visual.header.shader_id)) {
        Some(LevelShaderEntry::Reference(reference)) => (
          reference.textures.first().cloned().unwrap_or_default(),
          reference.shader.clone(),
        ),
        _ => {
          return Err(XrfError::new_invalid_error(format!(
            "A visual names shader row {} of {}, which is no shader reference",
            visual.header.shader_id,
            shaders.entries.len()
          )));
        }
      },
    };
    let vertices: Vec<LevelVertex> = source.read_vertices::<T>(
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    )?;
    let indices: Vec<u16> = source
      .read_full_detail_indices::<T>(visual)?
      .expect("a visual with a geometry container has indices");

    Ok(Some(Self {
      texture,
      shader,
      dress: SurfaceDress::default(),
      vertices,
      indices,
    }))
  }
}
