use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_parallax_settings::RenderParallaxSettings;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_surface_color::RenderSurfaceColor;
use crate::contract::render_view_options::RenderViewOptions;
use crate::scene::static_scene::static_selection::StaticSelection;

/// The camera as `shaders/common/camera.wgsl` declares it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
  pub view_projection: Mat4,
  pub inverse_view_projection: Mat4,
  pub view: Mat4,
  pub projection: Mat4,
  pub inverse_projection: Mat4,
  pub position: Vec4,
  /// The viewport's size, then its top left corner in the window.
  pub viewport: Vec4,
  /// The frustum's planes in renderer space, pointing inward.
  pub planes: [Vec4; 6],
  /// Textured, bumped, the baked hemisphere's strength, and the surfaces' bits: one where the engines' steep parallax is
  /// drawn, two, four and eight where the G-buffer draws the parallax, the bumps or the heights in place of the albedo
  /// for their debug views, sixteen where the enhanced parallax is marched.
  pub switches: Vec4,
  /// One where the static surfaces draw as their triangles' edges; times a uv checker repeats in place of every
  /// surface's textures; one where surfaces draw solid.
  pub modes: Vec4,
  /// World to clip without the jitter, this frame and the last, which a surface's motion is measured by.
  pub motion_current: Mat4,
  pub motion_previous: Mat4,
  /// World to clip with the jitter the last frame drew by, which a read of that frame's targets reprojects through.
  pub view_projection_previous: Mat4,
  /// What shows where nothing was drawn and neither the sky nor the fog is; `w` one where it is set.
  pub backdrop: Vec4,
  /// What a surface naming no base texture is drawn; `w` one where it is set, white where it is not.
  pub plain: Vec4,
  /// The backdrop's second colour, and in `w` the side of a square in render pixels; zero for a plain backdrop.
  pub backdrop_squares: Vec4,
  /// One while something is selected; the place its clusters are drawn in, or `u32::MAX` for any; how many runs follow.
  pub selection: [u32; 4],
  /// The selection's runs of clusters, two a row: each its first and the one past its last.
  pub selection_runs: [[u32; 4]; SELECTION_ROWS],
  /// The enhanced parallax's steps at most, its range in metres, its depth in detail coordinates, and one where its
  /// march is refined; the sun's direction in view space and how much of it its relief keeps from its hollows; how far a
  /// terrain's layers blend by height, how much its low ground darkens, and a steep surface's depth and range.
  pub parallax: Vec4,
  pub parallax_light: Vec4,
  pub parallax_mix: Vec4,
  /// One where the scene holds linear light at its exposure, which the window pass tonemaps; zero where every pass
  /// writes tonemapped colour, as the engine's do.
  pub output: Vec4,
}

/// Rows of the camera's selection runs, two runs each.
const SELECTION_ROWS: usize = 32;

impl CameraUniform {
  /// What a viewport's own camera draws the static scene with by its `options`: textured, bumped, the hemisphere's
  /// strength, then the parallax's bits (steep, the parallax, bumps and heights debug views, enhanced).
  pub fn get_switches(options: &RenderViewOptions) -> Vec4 {
    Vec4::new(
      f32::from(u8::from(options.mode.surface_color == RenderSurfaceColor::Textured)),
      options.mode.is_bumped as u32 as f32,
      options.features.hemi_strength,
      f32::from(
        u8::from(options.features.parallax.is_steep)
          | u8::from(options.mode.debug_view == RenderDebugView::Parallax) << 1
          | u8::from(options.mode.debug_view == RenderDebugView::Bumps) << 2
          | u8::from(options.mode.debug_view == RenderDebugView::Heights) << 3
          | u8::from(options.features.parallax.is_enhanced()) << 4,
      ),
    )
  }

  pub fn new(view: &CameraView, rect: RenderRect, switches: Vec4) -> Self {
    let view_projection: Mat4 = view.get_view_projection();

    Self {
      view_projection,
      inverse_view_projection: view_projection.inverse(),
      view: view.view,
      projection: view.projection,
      inverse_projection: view.projection.inverse(),
      position: view.position.extend(1.0),
      viewport: Vec4::new(rect.width as f32, rect.height as f32, rect.x as f32, rect.y as f32),
      planes: view.get_planes(),
      switches,
      modes: Vec4::ZERO,
      motion_current: view_projection,
      motion_previous: view_projection,
      view_projection_previous: view_projection,
      backdrop: Vec4::ZERO,
      plain: Vec4::ZERO,
      backdrop_squares: Vec4::ZERO,
      selection: [0; 4],
      selection_runs: [[0; 4]; SELECTION_ROWS],
      parallax: Vec4::ZERO,
      parallax_light: Vec4::ZERO,
      parallax_mix: Vec4::ZERO,
      output: Vec4::ZERO,
    }
  }

  /// The same camera marching the enhanced parallax by `settings`, shadowing towards the sun along `to_sun`, a view
  /// space direction.
  pub fn with_parallax(mut self, settings: &RenderParallaxSettings, to_sun: Vec3) -> Self {
    self.parallax = Vec4::new(
      settings.steps.max(1) as f32,
      settings.range,
      settings.height,
      f32::from(u8::from(settings.is_refined)),
    );
    self.parallax_light = to_sun.normalize_or_zero().extend(settings.shadow);
    self.parallax_mix = Vec4::new(
      settings.blending,
      settings.cavity,
      settings.surface_height,
      settings.surface_range,
    );
    self
  }

  /// The same camera drawing as an asset viewer asks: a uv checker in place of every texture, surfaces solid where
  /// their alpha is not shown, its backdrop and the colour of a surface with no texture. `scale` is render pixels a
  /// viewport pixel, which a backdrop square's side is measured in.
  pub fn with_asset_view(mut self, options: &RenderViewOptions, scale: f32) -> Self {
    let to_set = |color: Option<[f32; 3]>| color.map_or(Vec4::ZERO, |it| Vec4::from((Vec3::from(it), 1.0)));

    self.modes.y = options.asset.checker;
    self.modes.z = f32::from(u8::from(!options.show.is_alpha_visible));
    self.backdrop = to_set(options.asset.backdrop);
    self.plain = to_set(options.asset.plain_color);
    self.backdrop_squares = options
      .asset
      .backdrop_squares
      .map_or(Vec4::ZERO, |it| Vec3::from(it.color).extend((it.size * scale).max(1.0)));
    self
  }

  /// The same camera measuring motion between two unjittered view projections, the last frame's and this one's.
  pub fn with_motion(mut self, (current, previous): (Mat4, Mat4)) -> Self {
    self.motion_current = current;
    self.motion_previous = previous;
    self
  }

  /// The same camera remembering the jittered view projection the last frame drew by.
  pub fn with_previous(mut self, previous: Mat4) -> Self {
    self.view_projection_previous = previous;
    self
  }

  /// The same camera marking what a selection names where the G-buffer draws it; past the runs it holds, the runs
  /// nearest each other are joined, which marks the fewest clusters it does not name.
  pub fn with_selection(mut self, selection: Option<&StaticSelection>) -> Self {
    let Some(selection) = selection.filter(|it| !it.runs.is_empty()) else {
      return self;
    };
    let runs: Vec<[u32; 2]> = join_nearest(&selection.runs, SELECTION_ROWS * 2);

    self.selection = [1, selection.place.unwrap_or(u32::MAX), runs.len() as u32, 0];

    for (index, run) in runs.iter().enumerate() {
      self.selection_runs[index / 2][(index % 2) * 2] = run[0];
      self.selection_runs[index / 2][(index % 2) * 2 + 1] = run[1];
    }

    self
  }

  /// The same camera drawing an untextured surface as clay, or as its shader's tint.
  pub fn with_surface_color(mut self, color: RenderSurfaceColor) -> Self {
    self.modes.w = f32::from(u8::from(color == RenderSurfaceColor::Clay));
    self
  }

  /// The same camera over a scene holding linear light (`is_linear`), or tonemapped colour.
  pub fn with_linear_scene(mut self, is_linear: bool) -> Self {
    self.output.x = f32::from(u8::from(is_linear));
    self
  }

  /// The same camera drawing every static surface as its triangles' edges, or as itself.
  pub fn with_wireframe(mut self, is_wireframe: bool) -> Self {
    self.modes.x = f32::from(u8::from(is_wireframe));
    self
  }
}

/// Runs in order joined across their narrowest gaps until at most `most` remain.
fn join_nearest(runs: &[[u32; 2]], most: usize) -> Vec<[u32; 2]> {
  let mut joined: Vec<[u32; 2]> = runs.to_vec();

  while joined.len() > most {
    let narrowest: usize = (1..joined.len())
      .min_by_key(|index| joined[*index][0].saturating_sub(joined[index - 1][1]))
      .unwrap_or(1);

    joined[narrowest - 1][1] = joined[narrowest][1];
    joined.remove(narrowest);
  }

  joined
}
