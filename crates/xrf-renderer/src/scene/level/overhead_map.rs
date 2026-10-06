use glam::{Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, StorageArray,
  StorageArrayMut, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::shadow_cull::ShadowCull;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_depth_parameters::WaterDepthParameters;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::overhead_shape::OverheadShape;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_layout::StaticLayout;

/// A square of the level around the camera seen straight down, as its shape lays it out: a depth whose every texel says
/// how high the first thing over that column stands. It moves a whole step at a time, and is drawn again where it
/// moved or the scene grew, by the level's shadow casters.
pub struct OverheadMap {
  /// Reversed: one at the height it is seen from, nought where nothing stands.
  pub depth: wgpu::TextureView,
  lists: GrowableBuffer,
  args: wgpu::Buffer,
  view: ViewBinding,
  /// Where it was drawn, centred and seen from, and what the scene held then, or none before it was.
  drawn: Option<(Vec3, usize)>,
  /// Whether this frame draws it, as `prepare` decided.
  is_due: bool,
  /// Frames since it was last drawn, which a change of what the scene holds alone waits out.
  since: u32,
  shape: OverheadShape,
}

impl OverheadMap {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64, shape: OverheadShape) -> Self {
    Self {
      depth: device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some(shape.label),
          size: wgpu::Extent3d {
            width: shape.resolution,
            height: shape.resolution,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: SunShadowMaps::FORMAT,
          usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default()),
      lists: GrowableBuffer::new(device, shape.label, wgpu::BufferUsages::STORAGE),
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(shape.label),
        size: args_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      view: ViewBinding::new(device, view_layout),
      drawn: None,
      is_due: false,
      since: 0,
      shape,
    }
  }

  /// How it is laid out.
  pub fn get_shape(&self) -> OverheadShape {
    self.shape
  }

  /// Whether this frame draws it, as `prepare` decided.
  pub fn is_due(&self) -> bool {
    self.is_due
  }

  /// Its centre in `x` and `z`, its half width, and the height it is seen from; seen from under everything before it
  /// was drawn, so nothing is covered.
  pub fn get_window(&self) -> Vec4 {
    match self.drawn {
      Some((center, _)) => Vec4::new(center.x, center.z, self.shape.width / 2.0, center.y),
      None => Vec4::new(0.0, 0.0, self.shape.width / 2.0, -1e9),
    }
  }

  /// Centres it over the camera, a whole step at a time, and readies it to be drawn where it moved or the scene grew;
  /// `encoder` takes the copies a list growing makes.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame: &ShadowFrame<'_>,
  ) {
    let position: Vec3 = frame.camera.position;
    let step: f32 = self.shape.step;
    let center: Vec3 = Vec3::new(
      (position.x / step).round() * step,
      (position.y / self.shape.height_step).round() * self.shape.height_step + self.shape.height,
      (position.z / step).round() * step,
    );
    let scene = frame.scene;
    let state: (Vec3, usize) = (center, scene.get_contents());

    self.is_due = false;
    self.since = self.since.saturating_add(1);

    let is_moved: bool = self.drawn.is_none_or(|(drawn, _)| drawn != center);

    if self.drawn == Some(state) || (!is_moved && self.since < self.shape.content_delay) {
      return;
    }

    let half: f32 = self.shape.width / 2.0;
    let view: CameraView = CameraView {
      position: center,
      view: glam::camera::rh::view::look_at_mat4(center, center - Vec3::Y, -Vec3::Z),
      projection: glam::camera::rh::proj::directx::orthographic(-half, half, -half, half, self.shape.depth, 0.0),
    };

    self
      .lists
      .reserve(device, encoder, (scene.get_list_capacity().max(1) as u64) * 8);
    queue.write_buffer(&self.args, 0, bytemuck::cast_slice(scene.get_initial_args()));
    self.view.write(
      queue,
      &CameraUniform::new(
        &view,
        RenderRect {
          x: 0,
          y: 0,
          width: self.shape.resolution,
          height: self.shape.resolution,
        },
        Vec4::ZERO,
      ),
    );

    self.is_due = true;
    self.drawn = Some(state);
    self.since = 0;
  }

  /// Declares its cull and its draw, where this frame's `prepare` found it due: the level's shadow casters, or its
  /// water, its waves lifted by `water`, where it is a map of the water.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (scene, depth): (&StaticSceneHandles, GraphTexture),
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
    water: Option<UniformBinding<WaterUniform>>,
  ) {
    if !self.is_due {
      return;
    }

    let shape: OverheadShape = self.shape;
    let args: GraphBuffer = bindings.import_buffer(graph, shape.draw_pass, &self.args);
    let lists: GraphBuffer = bindings.import_buffer(graph, shape.cull_pass, self.lists.get_buffer());
    let cull: StaticCullParameters = scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args));
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_layout_draws(StorageArray::new(lists));

    graph
      .add_compute_pass(shape.cull_pass)
      .parameters(&cull)
      .record(move |context| {
        let kind: ShadowCull = if shape.is_water {
          ShadowCull::Water
        } else {
          ShadowCull::Cascade
        };

        passes.cull.record_shadow(context, &self.view, &cull, params, kind);
      });
    StaticDraws::declare_layouts(
      graph
        .add_raster_pass(shape.draw_pass)
        .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0)))
        .buffer(args, GraphBufferAccess::Indirect),
      &layouts,
    )
    .record(move |context| {
      let args: &wgpu::Buffer = context.get_buffer(args);

      match water {
        Some(water) => {
          let parameters: WaterDepthParameters = WaterDepthParameters { water };

          passes
            .water
            .record_depth(context, (&self.view, textures, layouts), args, &parameters);
        }
        None => passes
          .shadow
          .record_layered(context, (&self.view, &layouts, textures), args, shape.is_tree_drawn),
      }
    });
  }
}
