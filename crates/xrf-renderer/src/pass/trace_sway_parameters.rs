use xrf_renderer_core::{PassParameters, StorageArray, StorageArrayMut, UniformBinding};

use crate::pass::trace_sway_slot::TraceSwaySlot;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_surface::StaticSurface;

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
