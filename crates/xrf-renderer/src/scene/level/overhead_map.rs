use glam::{Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphTexture, StorageArray,
  StorageArrayMut, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic_job::PeriodicJob;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic_ticket::PeriodicTicket;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::shadow_casters::ShadowCasters;
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
  /// Whether this frame draws it, as `commit` took it.
  is_due: bool,
  /// The draw offered this frame, where it wants one, and where it is centred and what the scene holds for it.
  offer: Option<(PeriodicTicket, (Vec3, usize))>,
  /// Frames since it was last drawn, which a change of what the scene holds alone waits out.
  since: u32,
  /// Frames it has waited for a draw it wants.
  waited: u32,
  shape: OverheadShape,
}

impl OverheadMap {
  /// Frames a map drawn before waits at most for a draw it wants.
  pub const WAIT_MOST: u32 = 8;

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
      offer: None,
      since: 0,
      waited: 0,
      shape,
    }
  }

  /// How it is laid out.
  pub fn get_shape(&self) -> OverheadShape {
    self.shape
  }

  /// Makes it `width` metres across, its step a whole 64 texels of it, drawn again where that changes it.
  pub fn fit_width(&mut self, width: f32) {
    if width == self.shape.width {
      return;
    }

    self.shape.width = width;
    self.shape.step = width / self.shape.resolution as f32 * 64.0;
    self.drawn = None;
  }

  /// Draws nothing this frame.
  pub fn rest(&mut self) {
    self.is_due = false;
    self.offer = None;
  }

  /// Whether this frame draws it, as `commit` took it.
  pub fn is_due(&self) -> bool {
    self.is_due
  }

  /// Its centre in `x` and `z`, its half width, and the height it is seen from; seen from under everything before it
  /// was drawn, so nothing is covered.
  pub fn get_window(&self) -> Vec4 {
    match self.drawn {
      Some((center, _)) => Vec4::new(center.x, center.z, self.shape.width / 2.0, center.y),
      None => Self::get_unmade_window(self.shape),
    }
  }

  /// The window of a map of `shape` not drawn yet: centred nowhere, seen from far below.
  pub fn get_unmade_window(shape: OverheadShape) -> Vec4 {
    Vec4::new(0.0, 0.0, shape.width / 2.0, -1e9)
  }

  /// Centres it over the camera, a whole step at a time, and offers a draw to `scheduler` where it moved or the scene
  /// changed under it: at once where it was never drawn, else within `WAIT_MOST` frames, as it holds what it covers by
  /// a margin past a step.
  pub fn offer(&mut self, frame: &ShadowFrame<'_>, scheduler: &mut PeriodicScheduler) {
    let position: Vec3 = frame.camera.position;
    let step: f32 = self.shape.step;
    let center: Vec3 = Vec3::new(
      (position.x / step).round() * step,
      (position.y / self.shape.height_step).round() * self.shape.height_step + self.shape.height,
      (position.z / step).round() * step,
    );
    let scene = frame.scene;
    let state: (Vec3, usize) = (center, scene.get_contents());
    let half: f32 = self.shape.width / 2.0;

    self.is_due = false;
    self.offer = None;
    self.since = self.since.saturating_add(1);

    let is_moved: bool = self.drawn.is_none_or(|(drawn, _)| drawn != center);
    // Only an item coming or going over the square it covers changes it.
    let is_changed: bool = self.drawn.is_some_and(|(_, contents)| {
      scene.is_changed_since(contents, |sphere| {
        (sphere.x - center.x).abs() <= half + sphere.w && (sphere.z - center.z).abs() <= half + sphere.w
      })
    });

    if !is_moved && (!is_changed || self.since < self.shape.content_delay) {
      self.waited = 0;

      return;
    }

    let texels: f32 = (self.shape.resolution as f32).powi(2);
    let job: PeriodicJob = if self.drawn.is_none() {
      PeriodicJob::new_now(self.shape.passes, self.shape.seed, texels)
    } else {
      PeriodicJob::new_waiting(
        self.shape.passes,
        self.shape.seed,
        texels,
        (self.waited, Self::WAIT_MOST),
      )
    };

    self.offer = Some((scheduler.offer(job), state));
  }

  /// Readies the draw `offer` offered, where `scheduler` granted it; one not granted waits a frame more. `encoder` takes
  /// the copies a list growing makes.
  pub fn commit(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    let Some((ticket, (center, contents))) = self.offer.take() else {
      return;
    };

    if !scheduler.is_granted(ticket) {
      self.waited += 1;

      return;
    }

    let half: f32 = self.shape.width / 2.0;
    let view: CameraView = CameraView {
      position: center,
      view: glam::camera::rh::view::look_at_mat4(center, center - Vec3::Y, -Vec3::Z),
      projection: glam::camera::rh::proj::directx::orthographic(-half, half, -half, half, self.shape.depth, 0.0),
    };
    let scene = frame.scene;

    self
      .lists
      .reserve(device, encoder, (scene.cull.get_list_capacity().max(1) as u64) * 8);
    scene.cull.reset_args(encoder, &self.args);
    self.view.write(
      encoder,
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
    self.drawn = Some((center, contents));
    self.since = 0;
    self.waited = 0;
  }

  /// Forgets that this frame drew it, where its frame was not submitted: it draws again.
  pub fn abandon_frame(&mut self) {
    if self.is_due {
      self.is_due = false;
      self.drawn = None;
    }
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
        None => passes.shadow.record_some(
          context,
          (&self.view, &layouts, textures),
          args,
          if shape.is_tree_drawn {
            ShadowCasters::All
          } else {
            ShadowCasters::Fixed
          },
        ),
      }
    });
  }
}
