use glam::{Mat4, Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::camera::camera_view::CameraView;
use crate::contract::render_camera_settings::RenderCameraSettings;

/// What the camera effects' stages read, as WGSL's `CameraEffects`: the shown and the drawn frame's sizes, how the
/// stored depth turns into metres, the lens, the focus and the shutter.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "CameraEffects")]
pub struct CameraEffectsUniform {
  /// The shown frame's size in pixels, then its reciprocal.
  pub shown: Vec4,
  /// The drawn frame's size in texels, then its reciprocal.
  pub drawn: Vec4,
  /// The projection's depth terms, `z` to `z`, `w` to `z` and `z` to `w`, then one for parallel rays: what a stored
  /// depth is turned back into metres from the camera by.
  pub depth: Vec4,
  /// The lens: its aperture's width and its focal length in metres, the metres it is focused at, and the shown pixels
  /// a metre of the sensor spans.
  pub lens: Vec4,
  /// The depth of field: the widest circle of confusion's radius in shown pixels, one where it is drawn, and the tiles
  /// a tile's circle reaches over either way.
  pub focus: Vec4,
  /// The motion blur: the share of a frame the shutter is open, the longest blur's radius in shown pixels, the tile's
  /// size in shown pixels, and one where it is drawn.
  pub motion: Vec4,
  /// The tangents of the lens's half view across and up, then its vignette's and its aberration's strengths.
  pub optics: Vec4,
}

impl CameraEffectsUniform {
  /// The shown pixels a tile of the depth of field spans: eight of its half-size texels.
  pub const FOCUS_TILE: u32 = 16;
  /// The shown pixels a block of the motion's first reduction spans; a motion tile is a whole number of them.
  pub const MOTION_BLOCK: u32 = 8;
  /// The most shown pixels a motion tile spans, so its reduction stays short however long the blur.
  const MOTION_TILE_MOST: u32 = 128;
  /// The widest circle of confusion's radius, as a share of the frame's height.
  const FOCUS_RADIUS_SHARE: f32 = 0.02;

  /// The camera `settings` set over a frame drawn `drawn` texels across and shown `shown` pixels across, through
  /// `view`'s lens.
  pub fn new(settings: &RenderCameraSettings, view: &CameraView, (drawn, shown): ((u32, u32), (u32, u32))) -> Self {
    let projection: Mat4 = view.projection;
    let is_orthographic: bool = view.is_orthographic();
    let tangents: Vec2 = view.get_lens_tangents();
    let height: f32 = shown.1 as f32;
    // A lens of parallel rays has no focal length to blur by.
    let is_focused: bool = settings.is_defocused && !is_orthographic;
    let sensor: f32 = settings.sensor_height.max(1.0) * 1e-3;
    let focal: f32 = if is_orthographic {
      0.0
    } else {
      sensor * 0.5 / tangents.y
    };
    let focus_radius: f32 = (height * Self::FOCUS_RADIUS_SHARE).max(1.0);
    let motion_radius: f32 = (height * settings.motion_limit.max(0.0)).max(1.0);
    let motion_tile: u32 = Self::get_motion_tile(motion_radius);
    let switch = |is_on: bool, value: f32| if is_on { value.max(0.0) } else { 0.0 };

    Self {
      shown: Self::sized(shown),
      drawn: Self::sized(drawn),
      depth: Vec4::new(
        projection.z_axis.z,
        projection.w_axis.z,
        projection.z_axis.w,
        f32::from(u8::from(is_orthographic)),
      ),
      lens: Vec4::new(
        focal / settings.f_stop.max(0.5),
        focal,
        settings.focus_distance.max(focal * 2.0 + 0.01),
        height / sensor,
      ),
      focus: Vec4::new(
        focus_radius,
        f32::from(u8::from(is_focused)),
        focus_radius.div_euclid(Self::FOCUS_TILE as f32) + 1.0,
        0.0,
      ),
      motion: Vec4::new(
        settings.shutter.clamp(0.0, 1.0),
        motion_radius.min(motion_tile as f32),
        motion_tile as f32,
        f32::from(u8::from(settings.is_motion_blurred)),
      ),
      optics: Vec4::new(
        tangents.x,
        tangents.y,
        switch(settings.is_vignetted, settings.vignette_strength),
        switch(settings.is_aberrated, settings.aberration_strength),
      ),
    }
  }

  /// Whether the frame is blurred out of focus.
  pub fn is_focused(&self) -> bool {
    self.focus.y > 0.0
  }

  /// Whether the frame is blurred by its motion.
  pub fn is_motion_blurred(&self) -> bool {
    self.motion.w > 0.0
  }

  /// Whether the lens dims or splits the frame.
  pub fn is_lensed(&self) -> bool {
    self.optics.z > 0.0 || self.optics.w > 0.0
  }

  /// The shown pixels a motion tile spans this frame.
  pub fn get_motion_tile_size(&self) -> u32 {
    self.motion.z as u32
  }

  /// The shown pixels a motion tile spans for a blur of `radius` shown pixels at most: whole blocks, so the tiles
  /// reduce from them.
  fn get_motion_tile(radius: f32) -> u32 {
    let tile: u32 = (radius.ceil() as u32).div_ceil(Self::MOTION_BLOCK) * Self::MOTION_BLOCK;

    tile.clamp(Self::MOTION_BLOCK, Self::MOTION_TILE_MOST)
  }

  fn sized((width, height): (u32, u32)) -> Vec4 {
    let (width, height): (f32, f32) = (width.max(1) as f32, height.max(1) as f32);

    Vec4::new(width, height, 1.0 / width, 1.0 / height)
  }
}
