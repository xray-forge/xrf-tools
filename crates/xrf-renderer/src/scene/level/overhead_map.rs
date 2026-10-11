use glam::{Mat4, Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphDepthAttachment, GraphRuntime, GraphTexture,
  GraphTextureDescriptor, StorageArray, StorageArrayMut, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic_job::PeriodicJob;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic_ticket::PeriodicTicket;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::sun_shadow_maps::SunShadowMaps;
use crate::frame::texture_copy::add_texture_copy;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::overhead_shift_parameters::OverheadShiftParameters;
use crate::pass::overhead_shift_uniform::OverheadShiftUniform;
use crate::pass::shadow_casters::ShadowCasters;
use crate::pass::shadow_cull::ShadowCull;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_shadow_pass::set_region;
use crate::pass::water_depth_parameters::WaterDepthParameters;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::overhead_shape::OverheadShape;
use crate::scene::level::shadow_cut::ShadowCut;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::static_scene::static_layout::StaticLayout;

/// A square of the level around the camera seen straight down, as its shape lays it out: a depth whose every texel says
/// how high the first thing over that column stands. It moves a whole step at a time. Moved at the same height with
/// nothing changed under it, it keeps what it holds, shifted, and draws only the strips it did not cover before;
/// otherwise it is drawn again whole, by the level's shadow casters.
pub struct OverheadMap {
  /// Reversed: one at the height it is seen from, nought where nothing stands.
  pub depth: wgpu::TextureView,
  /// What culls and draws it whole or its first strip, and its second strip.
  cuts: [ShadowCut; 2],
  /// Where it was drawn, centred and seen from, and what the scene held then, or none before it was.
  drawn: Option<(Vec3, usize)>,
  /// Whether this frame draws it, as `commit` took it, and how.
  is_due: bool,
  draw: OverheadDraw,
  /// The draw offered this frame, where it wants one: where it is centred, what the scene holds for it, and how.
  offer: Option<(PeriodicTicket, (Vec3, usize), OverheadDraw)>,
  /// Frames since it was last drawn, which a change of what the scene holds alone waits out.
  since: u32,
  /// Frames it has waited for a draw it wants.
  waited: u32,
  shape: OverheadShape,
}

/// A region a map's pass draws as its passes take it: the cut it is culled and drawn by, the region, its draw
/// arguments, its cull and its draws.
type OverheadRegion<'a> = (
  &'a ShadowCut,
  [u32; 4],
  GraphBuffer,
  StaticCullParameters,
  [StaticDrawParameters; StaticLayout::COUNT],
);

/// How a map due this frame is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
enum OverheadDraw {
  Whole,
  /// Shifted by the texels across and down a texel reads its old self from, its uncovered strips drawn, each a region
  /// of texels across, down, wide and high.
  Shifted {
    offset: (i32, i32),
    strips: [Option<[u32; 4]>; 2],
  },
}

impl OverheadDraw {
  /// The regions of a map `resolution` texels across it culls and draws, each by a cut of its own: the whole map, or
  /// its strips.
  fn get_regions(self, resolution: u32) -> [Option<[u32; 4]>; 2] {
    match self {
      OverheadDraw::Whole => [Some([0, 0, resolution, resolution]), None],
      OverheadDraw::Shifted { strips, .. } => strips,
    }
  }
}

impl OverheadMap {
  /// Frames a map drawn before waits at most for a draw it wants.
  pub const WAIT_MOST: u32 = 8;
  /// What shifting the whole map costs, as texels of a draw.
  const SHIFT_SHARE: f32 = 1.0 / 8.0;

  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64, shape: OverheadShape) -> Self {
    let cut = || ShadowCut::new(device, view_layout, shape.label, args_size);

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
          usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
          view_formats: &[],
        })
        .create_view(&Default::default()),
      cuts: [cut(), cut()],
      drawn: None,
      is_due: false,
      draw: OverheadDraw::Whole,
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

  /// The strips a map `resolution` texels across did not cover before it moved so a texel reads its old self
  /// `(across, down)` texels away: the columns it moved into, every row of them, and the rows it moved into, of the
  /// other columns. Each a region of texels across, down, wide and high.
  pub fn get_strips((across, down): (i32, i32), resolution: u32) -> [Option<[u32; 4]>; 2] {
    let size: i32 = resolution as i32;
    let columns: Option<[u32; 4]> = match across {
      0 => None,
      across if across > 0 => Some([(size - across) as u32, 0, across as u32, resolution]),
      across => Some([0, 0, (-across) as u32, resolution]),
    };
    let (first, last): (i32, i32) = (across.min(0).abs(), size - across.max(0));
    let rows: Option<[u32; 4]> = match down {
      0 => None,
      down if down > 0 => Some([first as u32, (size - down) as u32, (last - first) as u32, down as u32]),
      down => Some([first as u32, 0, (last - first) as u32, (-down) as u32]),
    };

    [columns, rows]
  }

  /// Centres it over the camera, a whole step at a time, and offers a draw to `scheduler` where it moved or the scene
  /// changed under it: at once where it was never drawn, else within `WAIT_MOST` frames, as it holds what it covers by
  /// a margin past a step. Moved alone, it is offered shifted, its uncovered strips drawn.
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
    let draw: OverheadDraw = match self.drawn.filter(|_| !is_changed) {
      Some((drawn, _)) => self
        .get_shift(drawn, center)
        .map_or(OverheadDraw::Whole, |offset| OverheadDraw::Shifted {
          offset,
          strips: Self::get_strips(offset, self.shape.resolution),
        }),
      None => OverheadDraw::Whole,
    };
    let (passes, units): (&'static [&'static str], f32) = match draw {
      OverheadDraw::Whole => (self.shape.passes, texels),
      OverheadDraw::Shifted { strips, .. } => (
        self.shape.shift_passes,
        strips
          .iter()
          .flatten()
          .map(|[.., width, height]| (*width * *height) as f32)
          .sum::<f32>()
          + texels * Self::SHIFT_SHARE,
      ),
    };
    let job: PeriodicJob = if self.drawn.is_none() {
      PeriodicJob::new_now(passes, self.shape.seed, units)
    } else {
      PeriodicJob::new_waiting(passes, self.shape.seed, units, (self.waited, Self::WAIT_MOST))
    };

    self.offer = Some((scheduler.offer(job), state, draw));
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
    let Some((ticket, (center, contents), draw)) = self.offer.take() else {
      return;
    };

    if !scheduler.is_granted(ticket) {
      self.waited += 1;

      return;
    }

    let scene = frame.scene;

    for (cut, region) in self.cuts.iter_mut().zip(draw.get_regions(self.shape.resolution)) {
      let Some(region) = region else {
        continue;
      };
      let [x, y, width, height] = region;

      cut
        .lists
        .reserve(device, encoder, (scene.cull.get_list_capacity().max(1) as u64) * 8);
      scene.cull.reset_args(encoder, &cut.args);
      cut.view.write(
        encoder,
        &CameraUniform::new(
          &Self::create_view(self.shape, center, region),
          RenderRect {
            x: x as i32,
            y: y as i32,
            width,
            height,
          },
          Vec4::ZERO,
        ),
      );
    }

    self.is_due = true;
    self.draw = draw;
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

  /// Declares its draw, where this frame's `commit` took it: whole, culled and drawn; or shifted from a copy of
  /// itself, its strips culled and drawn over. Drawn of the level's shadow casters, or of its water, its waves lifted
  /// by `water`, where it is a map of the water.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (scene, depth): (&StaticSceneHandles, GraphTexture),
    (params, textures): (&'a StaticCullParams, &'a wgpu::BindGroup),
    water: Option<UniformBinding<WaterUniform>>,
  ) {
    if !self.is_due {
      return;
    }

    let shape: OverheadShape = self.shape;
    let resolution: u32 = shape.resolution;
    let regions: [Option<[u32; 4]>; 2] = self.draw.get_regions(resolution);
    let shift: Option<(i32, i32)> = match self.draw {
      OverheadDraw::Whole => None,
      OverheadDraw::Shifted { offset, .. } => Some(offset),
    };
    let (cull_pass, draw_pass): (&'static str, &'static str) = match shift {
      Some(_) => (shape.shift_passes[1], shape.shift_passes[2]),
      None => (shape.cull_pass, shape.draw_pass),
    };
    // Each region's cull and its draws, with the cut it is culled and drawn by.
    let cuts: Vec<OverheadRegion<'a>> = self
      .cuts
      .iter()
      .zip(regions)
      .filter_map(|(cut, region)| {
        let region: [u32; 4] = region?;
        let args: GraphBuffer = bindings.import_buffer(graph, draw_pass, &cut.args);
        let lists: GraphBuffer = bindings.import_buffer(graph, cull_pass, cut.lists.get_buffer());

        Some((
          cut,
          region,
          args,
          scene.get_cull_parameters(StorageArrayMut::new(lists), StorageArrayMut::new(args)),
          scene.get_layout_draws(StorageArray::new(lists)),
        ))
      })
      .collect();

    cuts
      .iter()
      .fold(graph.add_compute_pass(cull_pass), |builder, (.., cull, _)| {
        builder.parameters(cull)
      })
      .record({
        let culls: Vec<(&'a ShadowCut, StaticCullParameters)> =
          cuts.iter().map(|(cut, _, _, cull, _)| (*cut, *cull)).collect();

        move |context| {
          let kind: ShadowCull = if shape.is_water {
            ShadowCull::Water
          } else {
            ShadowCull::Cascade
          };

          for (cut, cull) in &culls {
            passes.cull.record_shadow(context, &cut.view, cull, params, kind);
          }
        }
      });

    // Shifted, it is copied first, then the copy laid back in where it now stands under the strips.
    let shifted: Option<OverheadShiftParameters> = shift.map(|offset| {
      let previous: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        shape.shift_passes[0],
        resolution,
        resolution,
        SunShadowMaps::FORMAT,
      ));

      add_texture_copy(&mut *graph, shape.shift_passes[0], depth, previous);

      OverheadShiftParameters {
        previous,
        shift: runtime.push_uniform(&OverheadShiftUniform::new(offset, resolution)),
      }
    });
    let builder = cuts.iter().fold(
      graph
        .add_raster_pass(draw_pass)
        .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0))),
      |builder, (_, _, args, _, layouts)| {
        StaticDraws::declare_layouts(builder.buffer(*args, GraphBufferAccess::Indirect), layouts)
      },
    );
    let builder = match &shifted {
      Some(shifted) => builder.parameters(shifted),
      None => builder,
    };

    builder.record(move |context| {
      if let Some(shifted) = &shifted {
        passes.shadow.record_shift(context, shifted);
      }

      for (cut, region, args, _, layouts) in &cuts {
        let args: &wgpu::Buffer = context.get_buffer(*args);

        match water {
          Some(water) => {
            let parameters: WaterDepthParameters = WaterDepthParameters { water };

            set_region(context, *region);
            passes
              .water
              .record_depth(context, (&cut.view, textures, *layouts), args, &parameters);
          }
          None => passes.shadow.record_region(
            context,
            *region,
            (&cut.view, layouts, textures),
            args,
            if shape.is_tree_drawn {
              ShadowCasters::All
            } else {
              ShadowCasters::Fixed
            },
          ),
        }
      }
    });
  }

  /// How far it moved from `drawn` to `center`, as the texels across and down a texel reads its old self from, where
  /// it stays at its height and still overlaps where it stood.
  fn get_shift(&self, drawn: Vec3, center: Vec3) -> Option<(i32, i32)> {
    let texel: f32 = self.shape.width / self.shape.resolution as f32;
    let across: i32 = ((center.x - drawn.x) / texel).round() as i32;
    let down: i32 = ((center.z - drawn.z) / texel).round() as i32;
    let size: i32 = self.shape.resolution as i32;

    (drawn.y == center.y && across.abs() < size && down.abs() < size).then_some((across, down))
  }

  /// The camera a region of a map centred at `center` is drawn by: the whole map's, looking straight down from where
  /// it is seen from, its projection narrowed to the region, so the region's texels land where the whole map's would.
  fn create_view(shape: OverheadShape, center: Vec3, [x, y, width, height]: [u32; 4]) -> CameraView {
    let texel: f32 = shape.width / shape.resolution as f32;
    let half: f32 = shape.width / 2.0;
    let left: f32 = -half + x as f32 * texel;
    let top: f32 = half - y as f32 * texel;
    let view: Mat4 = glam::camera::rh::view::look_at_mat4(center, center - Vec3::Y, -Vec3::Z);

    CameraView {
      position: center,
      view,
      projection: glam::camera::rh::proj::directx::orthographic(
        left,
        left + width as f32 * texel,
        top - height as f32 * texel,
        top,
        shape.depth,
        0.0,
      ),
    }
  }
}
