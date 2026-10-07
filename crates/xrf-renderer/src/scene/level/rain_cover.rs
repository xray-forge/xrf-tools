use glam::{Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, StorageArray,
  StorageArrayMut,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_layout::StaticLayout;

/// Metres the cover is across: every streak's column falls within it, however far the camera is from its centre, and
/// every surface the rain wets, 25 metres out at most.
pub const RAIN_COVER_WIDTH: f32 = 56.0;

/// Texels it is across, some five centimetres each.
pub const RAIN_COVER_RESOLUTION: u32 = 1024;

/// Metres above the camera it is seen from, past where every streak starts, and how far down it reaches from there.
const HEIGHT: f32 = 60.0;
pub const RAIN_COVER_DEPTH: f32 = 120.0;

/// Metres the cover moves by, so a camera moving less draws it again for nothing.
const STEP: f32 = 4.0;

/// What stands over the rain around the camera, seen straight down: a square of the level whose depth says, for each
/// column, how high the first thing a drop lands on is. It moves a whole step at a time, and is drawn again where it
/// moved or the scene grew, by the level's shadow casters.
pub struct RainCover {
  /// Reversed: one at the height it is seen from, nought where nothing stands.
  pub depth: wgpu::TextureView,
  lists: GrowableBuffer,
  args: wgpu::Buffer,
  view: ViewBinding,
  /// Where it was drawn, centred and seen from, and what the scene held then, or none before it was.
  drawn: Option<(Vec3, usize)>,
  /// Whether this frame draws it, as `prepare` decided.
  is_due: bool,
}

impl RainCover {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    Self {
      depth: device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some("rain cover"),
          size: wgpu::Extent3d {
            width: RAIN_COVER_RESOLUTION,
            height: RAIN_COVER_RESOLUTION,
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
      lists: GrowableBuffer::new(device, "rain cover lists", wgpu::BufferUsages::STORAGE),
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rain cover draw arguments"),
        size: args_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      view: ViewBinding::new(device, view_layout),
      drawn: None,
      is_due: false,
    }
  }

  /// Its centre in `x` and `z`, its half width, and the height it is seen from; seen from under everything before it
  /// was drawn, so nothing is covered.
  pub fn get_window(&self) -> Vec4 {
    match self.drawn {
      Some((center, _)) => Vec4::new(center.x, center.z, RAIN_COVER_WIDTH / 2.0, center.y),
      None => Vec4::new(0.0, 0.0, RAIN_COVER_WIDTH / 2.0, -1e9),
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
    let center: Vec3 = Vec3::new(
      (position.x / STEP).round() * STEP,
      (position.y / STEP).round() * STEP + HEIGHT,
      (position.z / STEP).round() * STEP,
    );
    let scene = frame.scene;
    let state: (Vec3, usize) = (center, scene.get_contents());

    self.is_due = false;

    if self.drawn == Some(state) {
      return;
    }

    let half: f32 = RAIN_COVER_WIDTH / 2.0;
    let view: CameraView = CameraView {
      position: center,
      view: glam::camera::rh::view::look_at_mat4(center, center - Vec3::Y, -Vec3::Z),
      projection: glam::camera::rh::proj::directx::orthographic(-half, half, -half, half, RAIN_COVER_DEPTH, 0.0),
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
          width: RAIN_COVER_RESOLUTION,
          height: RAIN_COVER_RESOLUTION,
        },
        Vec4::ZERO,
      ),
    );

    self.is_due = true;
    self.drawn = Some(state);
  }

  /// Declares its cull and its draw, where this frame's `prepare` found it due.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (scene, depth): (&StaticSceneHandles, GraphTexture),
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
  ) {
    if !self.is_due {
      return;
    }

    let args: GraphBuffer = bindings.import_buffer(graph, "rain cover draw arguments", &self.args);
    let lists: GraphBuffer = bindings.import_buffer(graph, "rain cover lists", self.lists.get_buffer());
    let cull: StaticCullParameters = scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args));
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene.get_layout_draws(StorageArray::new(lists));

    graph
      .add_compute_pass("rain cover cull")
      .parameters(&cull)
      .record(move |context| {
        passes.cull.record_shadow(context, &self.view, &cull, params, false);
      });
    StaticDraws::declare_layouts(
      graph
        .add_raster_pass("rain cover")
        .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0)))
        .buffer(args, GraphBufferAccess::Indirect),
      &layouts,
    )
    .record(move |context| {
      let args: &wgpu::Buffer = context.get_buffer(args);

      passes.shadow.record(context, (&self.view, &layouts, textures), args);
    });
  }
}
