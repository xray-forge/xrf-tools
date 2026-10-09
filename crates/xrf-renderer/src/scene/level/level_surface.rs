use glam::Vec4;
use xrf_renderer_core::{FrameGraph, GraphBindings, GraphRuntime, GraphTexture, UniformBinding};

use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::surface_heights_parameters::SurfaceHeightsParameters;
use crate::pass::surface_mask_uniform::SurfaceMaskUniform;
use crate::pass::surface_sites_parameters::SurfaceSitesParameters;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::overhead_map::OverheadMap;
use crate::scene::level::overhead_shape::OverheadShape;
use crate::scene::level::shadow_frame::ShadowFrame;

/// The level's surface around the camera, which the enhanced rain places its puddles on: its overhead map without its
/// trees, its water over the same square, and what is placed from them each time they are drawn, kept until they are
/// drawn again: the surface's lowest heights, and a puddle site a cell where a puddle may stand whole.
pub struct LevelSurface {
  pub map: OverheadMap,
  pub water: OverheadMap,
  pub lowest: wgpu::TextureView,
  pub sites: wgpu::TextureView,
}

impl LevelSurface {
  pub const HEIGHT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
  pub const SITES_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
  /// Metres a cell of sites is across, as `shaders/frame/surface_mask.wgsl` places them.
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

  /// Its centre in `x` and `z`, its half width and the height it is seen from; then its texels across and the metres
  /// it reaches down.
  pub fn get_window(&self) -> (Vec4, Vec4) {
    let shape: OverheadShape = self.map.get_shape();

    (
      self.map.get_window(),
      Vec4::new(shape.resolution as f32, shape.depth, 0.0, 0.0),
    )
  }

  /// Declares its draws and, where they are drawn, what is derived from them; answers its lowest heights, its mask and
  /// its water as the graph knows them. `water` lifts the water's waves.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
    water: UniformBinding<WaterUniform>,
  ) -> [GraphTexture; 3] {
    let surface: GraphTexture = bindings.import_view(&mut *graph, "level surface", &self.map.depth);
    let lowest: GraphTexture = bindings.import_view(&mut *graph, "level surface lowest", &self.lowest);
    let sites: GraphTexture = bindings.import_view(&mut *graph, "puddle sites", &self.sites);
    let water_depth: GraphTexture = bindings.import_view(&mut *graph, "level water", &self.water.depth);

    self.water.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (scene, water_depth),
      (params, textures),
      Some(water),
    );

    if !self.map.is_due() {
      return [lowest, sites, water_depth];
    }

    self.map.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (scene, surface),
      (params, textures),
      None,
    );

    let (window, shape) = self.get_window();
    let uniform: UniformBinding<SurfaceMaskUniform> = runtime.push_uniform(&SurfaceMaskUniform { window, shape });
    let heights: SurfaceHeightsParameters = SurfaceHeightsParameters {
      surface,
      lowest,
      shape: uniform,
    };
    let placing: SurfaceSitesParameters = SurfaceSitesParameters {
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
      .record(move |context| passes.surface_mask.record(context, size, (&heights, &placing)));

    [lowest, sites, water_depth]
  }
}
