use glam::Vec4;
use xrf_renderer_core::{FrameGraph, GraphBindings, GraphRuntime, GraphTexture};

use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::cover_pyramid_depth_parameters::CoverPyramidDepthParameters;
use crate::pass::cover_pyramid_level_parameters::CoverPyramidLevelParameters;
use crate::pass::level_passes::LevelPasses;
use crate::pass::static_cull_params::StaticCullParams;
use crate::scene::level::cover_pyramid::CoverPyramid;
use crate::scene::level::cover_textures::CoverTextures;
use crate::scene::level::overhead_map::OverheadMap;
use crate::scene::level::overhead_shape::OverheadShape;
use crate::scene::level::shadow_frame::ShadowFrame;

/// What stands over the rain around the camera, drawn straight down as both engines draw their rain shadow map, which
/// stops the streaks and keeps what is under it dry: one overhead map, fitted to the wet reach where the wetting reaches
/// past the engine's, and its pyramid of the lowest and highest under each texel, reduced whenever it is drawn.
pub struct RainCover {
  map: OverheadMap,
  pyramid: CoverPyramid,
}

impl RainCover {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    let shape: OverheadShape = OverheadShape::RAIN_COVER;

    Self {
      map: OverheadMap::new(device, view_layout, args_size, shape),
      pyramid: CoverPyramid::new(device, "rain cover pyramid", shape.resolution),
    }
  }

  /// Centres it over the camera, out to `reach` metres where the wetting reaches that far and over the near square
  /// otherwise, and offers `scheduler` a draw where it moved, changed width or the scene changed under it.
  pub fn offer(&mut self, frame: &ShadowFrame<'_>, reach: Option<f32>, scheduler: &mut PeriodicScheduler) {
    // Twice the reach across and a twentieth more, as the camera stands up to half a step off the centre, in whole
    // 64 m.
    let width: f32 = reach.map_or(OverheadShape::RAIN_COVER.width, |reach| {
      (reach * 2.1 / 64.0).ceil().max(2.0) * 64.0
    });

    self.map.fit_width(width);
    self.map.offer(frame, scheduler);
  }

  /// Readies the draw `scheduler` granted; `encoder` takes the copies a list growing makes.
  pub fn commit(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    self.map.commit(device, encoder, frame, scheduler);
  }

  /// Forgets that this frame drew it, where its frame was not submitted.
  pub fn abandon_frame(&mut self) {
    self.map.abandon_frame();
  }

  /// Its centre in `x` and `z`, its half width, and the height it is seen from.
  pub fn get_window(&self) -> Vec4 {
    self.map.get_window()
  }

  /// The window and the shape of a cover not made yet.
  pub fn get_unmade() -> (Vec4, (f32, f32)) {
    let shape: OverheadShape = OverheadShape::RAIN_COVER;

    (
      OverheadMap::get_unmade_window(shape),
      (shape.resolution as f32, shape.depth),
    )
  }

  /// Bytes its map and its pyramid take.
  pub fn get_memory(&self) -> u64 {
    let shape: OverheadShape = self.map.get_shape();
    let map: u64 = u64::from(shape.resolution).pow(2) * 4;
    let pyramid: u64 = (0..self.pyramid.levels)
      .map(|level| u64::from((self.pyramid.size >> level).max(1)).pow(2) * 8)
      .sum();

    map + pyramid
  }

  /// Its texels across and the metres it reaches down.
  pub fn get_shape(&self) -> (f32, f32) {
    let shape: OverheadShape = self.map.get_shape();

    (shape.resolution as f32, shape.depth)
  }

  /// Declares its draw and its pyramid's reduction where this frame's `commit` took it, and answers its depth
  /// and pyramid as the graph knows them.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
  ) -> CoverTextures {
    let depth: GraphTexture = bindings.import_view(&mut *graph, "rain cover", &self.map.depth);

    if self.map.is_due() {
      self.map.add_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        (scene, depth),
        (params, textures),
        None,
      );

      // Each level as its own resource, the cover reduced into the first and each into the next.
      let level_views: Vec<GraphTexture> = self
        .pyramid
        .level_views
        .iter()
        .map(|view| bindings.import_view(&mut *graph, "rain cover pyramid level", view))
        .collect();
      let first: CoverPyramidDepthParameters = CoverPyramidDepthParameters {
        source_depth: depth,
        cover_level: level_views[0],
      };
      let levels: Vec<CoverPyramidLevelParameters> = level_views
        .windows(2)
        .map(|pair| CoverPyramidLevelParameters {
          cover_source: pair[0],
          cover_level: pair[1],
        })
        .collect();
      let pyramid: &CoverPyramid = &self.pyramid;

      levels
        .iter()
        .fold(
          graph.add_compute_pass("rain cover pyramid").parameters(&first),
          |builder, level| builder.parameters(level),
        )
        .record(move |context| passes.pyramid.record_cover(context, pyramid, (&first, &levels)));
    }

    CoverTextures {
      depth,
      pyramid: bindings.import_view(&mut *graph, "rain cover pyramid", &self.pyramid.view),
    }
  }
}
