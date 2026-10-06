use glam::Vec4;
use xrf_renderer_core::{FrameGraph, GraphBindings, GraphRuntime, GraphTexture, UniformBinding};

use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lowest_heights_parameters::LowestHeightsParameters;
use crate::pass::puddle_keep_parameters::PuddleKeepParameters;
use crate::pass::puddle_sites_parameters::PuddleSitesParameters;
use crate::pass::puddle_sites_uniform::PuddleSitesUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::water_uniform::WaterUniform;
use crate::pass::wet_uniform::WetUniform;
use crate::scene::level::overhead_map::OverheadMap;
use crate::scene::level::overhead_shape::OverheadShape;
use crate::scene::level::shadow_frame::ShadowFrame;

/// The level's surface around the camera, which the enhanced rain places its puddles on: its overhead map without its
/// trees, its water over the same square, and what is placed from them each time they are drawn, kept until they are
/// drawn again: the surface's lowest heights, and a puddle site a cell where a puddle may stand whole; and each frame,
/// the puddle each site holds, which the wetting draws.
pub struct LevelSurface {
  pub map: OverheadMap,
  pub water: OverheadMap,
  pub lowest: wgpu::TextureView,
  sites: wgpu::TextureView,
  pub puddles: wgpu::TextureView,
}

/// What deciding the puddles reads beside the sites: the wet surfaces' settings and state, and the puddles' regional
/// noise and its sampler.
pub struct PuddleKeepInputs<'a> {
  pub wet: UniformBinding<WetUniform>,
  pub noise: &'a wgpu::TextureView,
  pub sampler: &'a wgpu::Sampler,
}

impl LevelSurface {
  pub const HEIGHT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
  pub const SITES_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
  /// Metres a cell of sites is across: a whole number of them to the map's step, so cells stay put in the world.
  pub const SITE_CELL: f32 = 8.0;

  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    let shape: OverheadShape = OverheadShape::LEVEL_SURFACE;
    let cells: u32 = Self::get_cells(shape);
    let create = |label: &str, size: u32, format: wgpu::TextureFormat| -> wgpu::TextureView {
      device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some(label),
          size: wgpu::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format,
          usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default())
    };

    Self {
      map: OverheadMap::new(device, view_layout, args_size, shape),
      water: OverheadMap::new(device, view_layout, args_size, OverheadShape::LEVEL_WATER),
      lowest: create("level surface lowest", shape.resolution, Self::HEIGHT_FORMAT),
      sites: create("puddle sites", cells, Self::SITES_FORMAT),
      puddles: create("puddles", cells, Self::SITES_FORMAT),
    }
  }

  /// Cells of sites a side of a map of a shape.
  fn get_cells(shape: OverheadShape) -> u32 {
    (shape.width / Self::SITE_CELL) as u32
  }

  /// Centres it and its water over the camera, readying them to be drawn and derived where they moved or the scene
  /// grew; both alike, as the sites read the water through the surface's window.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ShadowFrame<'_>,
  ) {
    self.map.prepare(device, queue, encoder, frame);
    self.water.prepare(device, queue, encoder, frame);
  }

  /// Its centre in `x` and `z`, its half width and the height it is seen from; then its texels across, the metres it
  /// reaches down, and the metres a cell of sites is across.
  pub fn get_window(&self) -> (Vec4, Vec4) {
    let shape: OverheadShape = self.map.get_shape();

    (
      self.map.get_window(),
      Vec4::new(shape.resolution as f32, shape.depth, Self::SITE_CELL, 0.0),
    )
  }

  /// Declares its draws and, where they are drawn, what is placed from them, then this frame's puddles; answers its
  /// lowest heights, its puddles and its water as the graph knows them. `water` lifts the water's waves.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
    (water, keep): (UniformBinding<WaterUniform>, PuddleKeepInputs<'a>),
  ) -> [GraphTexture; 3] {
    let lowest: GraphTexture = bindings.import_view(&mut *graph, "level surface lowest", &self.lowest);
    let sites: GraphTexture = bindings.import_view(&mut *graph, "puddle sites", &self.sites);
    let puddles: GraphTexture = bindings.import_view(&mut *graph, "puddles", &self.puddles);
    let water_depth: GraphTexture = bindings.import_view(&mut *graph, "level water", &self.water.depth);
    let cells: u32 = Self::get_cells(self.map.get_shape());

    self.water.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (scene, water_depth),
      (params, textures),
      Some(water),
    );

    if self.map.is_due() {
      self.add_placing_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        scene,
        (params, textures),
        water_depth,
      );
    }

    let keeping: PuddleKeepParameters<'a> = PuddleKeepParameters {
      kept_sites: sites,
      region_noise: bindings.import_view(&mut *graph, "puddle region noise", keep.noise),
      noise_sampler: keep.sampler,
      wet: keep.wet,
      puddles,
    };

    graph
      .add_compute_pass("puddles")
      .parameters(&keeping)
      .record(move |context| passes.puddle_sites.record_keep(context, cells, &keeping));

    [lowest, puddles, water_depth]
  }

  /// Draws the surface and places its lowest heights and sites from it.
  fn add_placing_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
    water_depth: GraphTexture,
  ) {
    let surface: GraphTexture = bindings.import_view(&mut *graph, "level surface", &self.map.depth);
    let lowest: GraphTexture = bindings.import_view(&mut *graph, "level surface lowest", &self.lowest);
    let sites: GraphTexture = bindings.import_view(&mut *graph, "puddle sites", &self.sites);

    self.map.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (scene, surface),
      (params, textures),
      None,
    );

    let (window, shape) = self.get_window();
    let uniform: UniformBinding<PuddleSitesUniform> = runtime.push_uniform(&PuddleSitesUniform { window, shape });
    let heights: LowestHeightsParameters = LowestHeightsParameters {
      surface,
      lowest,
      shape: uniform,
    };
    let placing: PuddleSitesParameters = PuddleSitesParameters {
      shape: uniform,
      heights: lowest,
      water: water_depth,
      sites,
    };
    let shape: OverheadShape = self.map.get_shape();
    let size: (u32, u32) = (shape.resolution, Self::get_cells(shape));

    graph
      .add_compute_pass("puddle sites")
      .parameters(&heights)
      .parameters(&placing)
      .record(move |context| passes.puddle_sites.record(context, size, (&heights, &placing)));
  }
}
