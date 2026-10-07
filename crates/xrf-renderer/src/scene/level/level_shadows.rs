use glam::{Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, StorageArray,
  StorageArrayMut,
};

use crate::contract::render_rect::RenderRect;
use crate::contract::render_shadow_settings::RENDER_MAX_SHADOW_CASCADES;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::lighting::sun_view_rays::SunViewRays;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::shadow_uniform::ShadowUniform;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::scene::level::shadow_cascade_view::ShadowCascadeView;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::static_scene::static_layout::StaticLayout;

/// Each cascade's draw, as the frame's timings name it.
const CASCADE_PASSES: [&str; RENDER_MAX_SHADOW_CASCADES] =
  ["sun shadow 0", "sun shadow 1", "sun shadow 2", "sun shadow 3"];

/// A level's sun shadow: its cascades fitted along the camera's view every frame, each culled and drawn into its map
/// only when its box moved or the scene grew, and then at most as often as its stagger allows, unless its box strayed
/// too far from its map to wait.
pub struct LevelShadows {
  maps: SunShadowMaps,
  uniform: wgpu::Buffer,
  values: ShadowUniform,
  cascades: Vec<ShadowCascadeView>,
  /// Frames the shadow has been in, which a cascade's stagger is counted by.
  frames: u64,
  /// The cascades this frame draws, as `prepare_cascades` decided.
  due: Vec<usize>,
  /// Bumped whenever the maps are made again, which the sun's bind group follows.
  epoch: u64,
}

impl LevelShadows {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      maps: SunShadowMaps::new(device, 1),
      uniform: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sun shadows"),
        size: size_of::<ShadowUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      values: ShadowUniform::default(),
      cascades: Vec::new(),
      frames: 0,
      due: Vec::new(),
      epoch: 0,
    }
  }

  pub fn get_maps(&self) -> &SunShadowMaps {
    &self.maps
  }

  pub fn get_uniform(&self) -> &wgpu::Buffer {
    &self.uniform
  }

  pub fn get_epoch(&self) -> u64 {
    self.epoch
  }

  /// Makes the maps again at the resolution the settings ask for, when it changed, so the sun binds them before the
  /// frame draws.
  pub fn prepare(&mut self, device: &wgpu::Device, resolution: u32) {
    let resolution: u32 = resolution.clamp(1, device.limits().max_texture_dimension_2d);

    if resolution != self.maps.resolution {
      self.maps = SunShadowMaps::new(device, resolution);
      self.epoch += 1;
      self.cascades.iter_mut().for_each(|cascade| cascade.drawn = None);
    }
  }

  /// Fits the cascades, decides which are due and readies what culls and draws them, and writes what the sun samples
  /// them by; `encoder` takes the copies a list growing makes.
  pub fn prepare_cascades(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    view_layout: &wgpu::BindGroupLayout,
    frame: &ShadowFrame<'_>,
  ) {
    let settings = frame.settings;
    let count: usize = settings.get_cascade_count();

    self.values.count = count as u32;
    self.due.clear();

    if count > 0 {
      self.frames += 1;
      self.ready_cascades(device, queue, encoder, view_layout, frame, count);
    }

    let look: Vec3 = -frame.camera.view.inverse().z_axis.truncate();

    self.values.forward = look.extend(0.0);
    self.values.filter = settings.filter;
    self.values.resolution = self.maps.resolution as f32;
    self.values.bias = settings.bias;
    self.values.blend = settings.blend;
    queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.values));
  }

  /// Declares the cascades due this frame: every one culled into its own lists in one compute pass, then each drawn
  /// into its layer of the maps.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    scene: &StaticSceneHandles,
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
  ) {
    if self.due.is_empty() {
      return;
    }

    let maps: GraphTexture = bindings.import_view(graph, "sun shadow maps", &self.maps.view);
    let due: Vec<(usize, &'a ShadowCascadeView, [GraphBuffer; 2], StaticCullParameters)> = self
      .due
      .iter()
      .map(|&index| {
        let cascade: &'a ShadowCascadeView = &self.cascades[index];
        let args: GraphBuffer = bindings.import_buffer(graph, "sun shadow draw arguments", &cascade.args);
        let lists: GraphBuffer = bindings.import_buffer(graph, "sun shadow lists", cascade.lists.get_buffer());

        (
          index,
          cascade,
          [args, lists],
          scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args)),
        )
      })
      .collect();

    due
      .iter()
      .fold(graph.add_compute_pass("sun shadow cull"), |builder, (.., cull)| {
        builder.parameters(cull)
      })
      .record({
        let culls: Vec<(&'a ShadowCascadeView, StaticCullParameters)> =
          due.iter().map(|(_, cascade, _, cull)| (*cascade, *cull)).collect();

        move |context| {
          for (cascade, cull) in &culls {
            passes.cull.record_shadow(context, &cascade.view, cull, params, false);
          }
        }
      });

    for (index, cascade, [args, lists], _) in due {
      let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_layout_draws(StorageArray::new(lists));

      StaticDraws::declare_layouts(
        graph
          .add_raster_pass(CASCADE_PASSES[index])
          .depth(GraphDepthAttachment::new(maps, wgpu::LoadOp::Clear(0.0)).with_array_layer(index as u32))
          .buffer(args, GraphBufferAccess::Indirect),
        &layouts,
      )
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(args);

        passes.shadow.record(context, (&cascade.view, &layouts, textures), args);
      });
    }
  }

  fn ready_cascades(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    view_layout: &wgpu::BindGroupLayout,
    frame: &ShadowFrame<'_>,
    count: usize,
  ) {
    let scene = frame.scene;
    let resolution: u32 = self.maps.resolution;
    let list_bytes: u64 = (scene.get_list_capacity().max(1) as u64) * 8;
    let mut rays: SunViewRays = SunViewRays::new(frame.camera);

    while self.cascades.len() < count.min(RENDER_MAX_SHADOW_CASCADES) {
      self
        .cascades
        .push(ShadowCascadeView::new(device, view_layout, scene.args.size()));
    }

    for (index, cascade) in self.cascades.iter_mut().take(count).enumerate() {
      let width: f32 = frame.settings.cascades.get(index).copied().unwrap_or(1.0);

      cascade.cascade.fit(
        frame.camera,
        &mut rays,
        frame.sun_direction,
        width,
        resolution,
        frame.settings.reach,
      );

      let state: (u64, usize) = (cascade.cascade.version, scene.get_contents());
      let is_due: bool = cascade.drawn.is_none()
        || !frame.settings.is_staggered
        || is_cascade_due(index, self.frames)
        || !cascade.cascade.is_held_by(&cascade.drawn_fit);

      // A map holds depth in the world, sampled with the matrix it was drawn with, so a still one is still exact.
      let is_changed: bool = cascade.drawn != Some(state);

      if !is_due
        || !(is_changed
          || frame
            .sway
            .is_redrawn(frame.sway.reach, cascade.cascade.texel, cascade.drawn_at))
      {
        continue;
      }

      cascade.lists.reserve(device, encoder, list_bytes);
      queue.write_buffer(&cascade.args, 0, bytemuck::cast_slice(scene.get_initial_args()));
      cascade.view.write(
        queue,
        &CameraUniform::new(
          &cascade.cascade.view,
          RenderRect {
            x: 0,
            y: 0,
            width: resolution,
            height: resolution,
          },
          Vec4::ZERO,
        ),
      );

      self.due.push(index);
      cascade.drawn = Some(state);
      cascade.drawn_at = frame.sway.time;
      cascade.drawn_fit = cascade.cascade;
      self.values.matrices[index] = cascade.cascade.view.get_view_projection();
      self.values.texels[index] = cascade.cascade.texel;
    }
  }
}

/// Whether a cascade may draw on a frame counted from one: the nearest every frame, the second every other, the third
/// every fourth, each on frames the ones before it leave, so at most two draw in one frame of a steady view.
fn is_cascade_due(cascade: usize, frame: u64) -> bool {
  let rate: u64 = 1 << cascade;

  cascade == 0 || frame % rate == (1 << (cascade - 1)) % rate
}
