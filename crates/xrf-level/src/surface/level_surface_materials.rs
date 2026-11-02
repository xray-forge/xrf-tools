use std::collections::BTreeMap;

use byteorder::ByteOrder;
use rayon::prelude::*;
use xrf_chunk::ChunkDataSource;
use xrf_error::XrfResult;
use xrf_math::{Matrix4x4, Vector3d};

use crate::cform::level_cform_tracer::LevelCformTracer;
use crate::geom::level_geom_source::LevelGeomSource;
use crate::geom::vertex::level_vertex::LevelVertex;
use crate::level::visuals::level_visual::LevelVisual;
use crate::level::visuals::level_visuals_chunk::LevelVisualsChunk;
use crate::surface::level_surface_material::LevelSurfaceMaterial;

/// What each of a level's render surfaces is made of, read off the collision form beneath it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelSurfaceMaterials {
  /// Each entry's tally, by its index in the shader table.
  pub surfaces: BTreeMap<u16, LevelSurfaceMaterial>,
  /// Visuals whose geometry could not be read, and so are not in any tally.
  pub unread_visuals: usize,
}

impl LevelSurfaceMaterials {
  /// How far above a triangle its ray starts, and half how far it reaches: a render face and the collision face made of
  /// it coincide, so a few centimetres either side finds the one under it and nothing past it.
  pub const RAY_LIFT: f32 = 0.05;

  /// Tallies every drawable visual of `level.geom` against the collision form `tracer` holds. Impostors are left out:
  /// a billboard standing for trees lies on nothing.
  pub fn survey<T: ByteOrder, D: ChunkDataSource + Sync>(
    visuals: &LevelVisualsChunk,
    source: &LevelGeomSource<D>,
    tracer: &LevelCformTracer,
  ) -> Self {
    visuals
      .visuals
      .par_iter()
      .filter(|visual| visual.lod.is_none())
      .filter_map(|visual| {
        visual
          .geometry
          .as_ref()
          .map(|_| (visual.header.shader_id, Self::tally::<T, D>(visual, source, tracer)))
      })
      .fold(Self::default, |mut survey, (shader, tally)| {
        survey.add(shader, tally);
        survey
      })
      .reduce(Self::default, |mut survey, other| {
        survey.unread_visuals += other.unread_visuals;

        for (shader, tally) in other.surfaces {
          survey.surfaces.entry(shader).or_default().merge(tally);
        }

        survey
      })
  }

  fn add(&mut self, shader: u16, tally: XrfResult<LevelSurfaceMaterial>) {
    match tally {
      Ok(tally) => self.surfaces.entry(shader).or_default().merge(tally),
      Err(_) => self.unread_visuals += 1,
    }
  }

  /// One visual's triangles, each looked up where it lies in the level.
  fn tally<T: ByteOrder, D: ChunkDataSource>(
    visual: &LevelVisual,
    source: &LevelGeomSource<D>,
    tracer: &LevelCformTracer,
  ) -> XrfResult<LevelSurfaceMaterial> {
    let mut tally: LevelSurfaceMaterial = LevelSurfaceMaterial::default();
    let Some(container) = &visual.geometry else {
      return Ok(tally);
    };
    let vertices: Vec<LevelVertex> = source.read_vertices::<T>(
      container.vertex_buffer_id,
      container.vertex_base,
      container.vertex_count,
    )?;
    let indices: Vec<u16> =
      source.read_indices::<T>(container.index_buffer_id, container.index_base, container.index_count)?;
    let placement: Option<&Matrix4x4> = visual.get_placement();
    let place = |index: u16| -> Option<Vector3d<f32>> {
      let position: &Vector3d<f32> = &vertices.get(usize::from(index))?.position;

      Some(placement.map_or_else(|| position.clone(), |transform| transform.transform_point(position)))
    };

    for [first, second, third] in indices.as_chunks::<3>().0 {
      let (Some(a), Some(b), Some(c)) = (place(*first), place(*second), place(*third)) else {
        continue;
      };
      let cross: Vector3d<f32> = (&b - &a).cross(&(&c - &a));
      let length: f32 = cross.length();

      // A degenerate triangle covers nothing and faces nowhere.
      if !length.is_normal() {
        continue;
      }

      let normal: Vector3d<f32> = &cross / length;
      let centre: Vector3d<f32> = &(&(&a + &b) + &c) / 3.0;
      let origin: Vector3d<f32> = &centre + &(&normal * Self::RAY_LIFT);
      let material: Option<u16> = tracer.get_nearest_material(&origin, &-&normal, Self::RAY_LIFT * 2.0);

      tally.add(f64::from(length) * 0.5, material);
    }

    Ok(tally)
  }
}
