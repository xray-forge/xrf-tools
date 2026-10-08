use glam::Vec4;
use xrf_renderer_core::{AccelerationStructureBinding, GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::grass_model_record::GrassModelRecord;
use crate::pass::light_record::LightRecord;
use crate::pass::lights_uniform::LightsUniform;
use crate::pass::shadow_uniform::ShadowUniform;
use crate::pass::trace_grass_uniform::TraceGrassUniform;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_trace_instance::StaticTraceInstance;
use crate::scene::static_scene::static_trace_record::StaticTraceRecord;

/// What a pass tracing rays against the level's static geometry binds (`StaticTrace`): the top-level structure; each
/// instance's first record and place, and each geometry's record naming its indices, vertices and surface; the places,
/// whose hemisphere lights an instance; each layout's vertices and the indices a hit's corners are read from; and the
/// surfaces; the view's local lights, how many and how their shadows compare, and the shadow atlas, which light a
/// hit; the sun's cascades and what they were drawn with, which shadow a hit they cover; and the level's grass as the
/// view plants it, its slot grid, slots, models and their colours, which covers a terrain hit. The bindless textures a
/// hit is shaded from are the texture cache's own group.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub struct TraceParameters<'a> {
  #[acceleration_structure]
  pub trace_scene: AccelerationStructureBinding<'a>,
  #[storage]
  pub trace_instances: StorageArray<StaticTraceInstance>,
  #[storage]
  pub trace_records: StorageArray<StaticTraceRecord>,
  #[storage]
  pub trace_places: StorageArray<StaticPlace>,
  #[storage]
  pub trace_baked_words: StorageArray<u32>,
  #[storage]
  pub trace_tree_words: StorageArray<u32>,
  #[storage]
  pub trace_model_words: StorageArray<u32>,
  #[storage]
  pub trace_indices: StorageArray<u32>,
  #[storage]
  pub trace_surfaces: StorageArray<StaticSurface>,
  #[storage]
  pub trace_lights: StorageArray<LightRecord>,
  #[uniform]
  pub trace_light_view: UniformBinding<LightsUniform>,
  #[texture(d2, depth)]
  pub trace_shadow_atlas: GraphTexture,
  #[sampler(comparison)]
  pub trace_shadow_sampler: &'a wgpu::Sampler,
  #[texture(d2_array, depth)]
  pub trace_sun_maps: GraphTexture,
  #[uniform]
  pub trace_sun_shadows: UniformBinding<ShadowUniform>,
  #[storage]
  pub trace_grass_grid: StorageArray<u32>,
  #[storage]
  pub trace_grass_slots: StorageArray<u32>,
  #[storage]
  pub trace_grass_models: StorageArray<GrassModelRecord>,
  #[storage]
  pub trace_grass_colours: StorageArray<Vec4>,
  #[uniform]
  pub trace_grass: UniformBinding<TraceGrassUniform>,
}
