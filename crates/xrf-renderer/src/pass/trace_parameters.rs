use xrf_renderer_core::{AccelerationStructureBinding, PassParameters, StorageArray};

use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_trace_record::StaticTraceRecord;

/// What a pass tracing rays against the level's static geometry binds (`StaticTrace`): the top-level structure, the
/// records naming each geometry's indices, vertices and surface, the baked vertices and the indices a hit's corners are
/// read from, and the surfaces. The bindless textures a hit is shaded from are the texture cache's own group.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub struct TraceParameters<'a> {
  #[acceleration_structure]
  pub trace_scene: AccelerationStructureBinding<'a>,
  #[storage]
  pub trace_records: StorageArray<StaticTraceRecord>,
  #[storage]
  pub trace_words: StorageArray<u32>,
  #[storage]
  pub trace_indices: StorageArray<u32>,
  #[storage]
  pub trace_surfaces: StorageArray<StaticSurface>,
}
