use xrf_renderer_core::{PassParameters, StorageArray, StorageArrayMut, UniformBinding};

use crate::feature::scene::wind::wind_uniform::WindUniform;
use crate::feature::tracing::view_trace::sway::trace_sway_slot::TraceSwaySlot;
use crate::scene::static_scene::item::static_place::StaticPlace;
use crate::scene::static_scene::surface::static_surface::StaticSurface;

/// What the trace's sway pass reads and writes: the tree arena's words, the places and surfaces, the view's wind, the
/// swaying instances, and their swayed vertices, three floats each, in renderer space.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct TraceSwayParameters {
  #[storage]
  pub words: StorageArray<u32>,
  #[storage]
  pub places: StorageArray<StaticPlace>,
  #[storage]
  pub surfaces: StorageArray<StaticSurface>,
  #[uniform]
  pub wind: UniformBinding<WindUniform>,
  #[storage]
  pub sway_slots: StorageArray<TraceSwaySlot>,
  #[storage]
  pub swayed_vertices: StorageArrayMut<f32>,
}
