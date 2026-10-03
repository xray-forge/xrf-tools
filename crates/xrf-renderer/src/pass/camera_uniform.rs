use glam::{Mat4, Vec3, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_surface_color::RenderSurfaceColor;
use crate::contract::render_view_options::RenderViewOptions;

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
  /// Textured, bumped, the baked hemisphere's strength, how far the water distorts what is behind it.
  pub switches: Vec4,
  /// One where the static surfaces draw as their triangles' edges; times a uv checker repeats in place of every
  /// surface's textures; one where surfaces draw solid.
  pub modes: Vec4,
  /// World to clip without the jitter, this frame and the last, which a surface's motion is measured by.
  pub motion_current: Mat4,
  pub motion_previous: Mat4,
  /// What shows where nothing was drawn and neither the sky nor the fog is; `w` one where it is set.
  pub backdrop: Vec4,
  /// What a surface naming no base texture is drawn; `w` one where it is set, white where it is not.
  pub plain: Vec4,
  /// The backdrop's second colour, and in `w` the side of a square in render pixels; zero for a plain backdrop.
  pub backdrop_squares: Vec4,
}

impl CameraUniform {
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
      backdrop: Vec4::ZERO,
      plain: Vec4::ZERO,
      backdrop_squares: Vec4::ZERO,
    }
  }

  /// The same camera drawing as an asset viewer asks: a uv checker in place of every texture, surfaces solid where
  /// their alpha is not shown, its backdrop and the colour of a surface with no texture. `scale` is render pixels a
  /// viewport pixel, which a backdrop square's side is measured in.
  pub fn with_asset_view(mut self, options: &RenderViewOptions, scale: f32) -> Self {
    let to_set = |color: Option<[f32; 3]>| color.map_or(Vec4::ZERO, |it| Vec4::from((Vec3::from(it), 1.0)));

    self.modes.y = options.checker;
    self.modes.z = f32::from(u8::from(!options.is_alpha_visible));
    self.backdrop = to_set(options.backdrop);
    self.plain = to_set(options.plain_color);
    self.backdrop_squares = options
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

  /// The same camera drawing an untextured surface as clay, or as its shader's tint.
  pub fn with_surface_color(mut self, color: RenderSurfaceColor) -> Self {
    self.modes.w = f32::from(u8::from(color == RenderSurfaceColor::Clay));
    self
  }

  /// The same camera drawing every static surface as its triangles' edges, or as itself.
  pub fn with_wireframe(mut self, is_wireframe: bool) -> Self {
    self.modes.x = f32::from(u8::from(is_wireframe));
    self
  }
}
