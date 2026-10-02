use glam::{Mat4, Vec3};

use crate::camera::camera_view::CameraView;
use crate::lighting::sun_cascade_basis::SunCascadeBasis;
use crate::lighting::sun_cascade_placement::place_sun_cascade;
use crate::lighting::sun_view_rays::SunViewRays;

/// How far a cascade's map reaches past its reach below its centre, in widths: the engine's `1.41421 * map_size`.
const DEPTH: f32 = std::f32::consts::SQRT_2;

/// The step, in widths, a cascade's place along the light is rounded to, so a move along it alone keeps the map.
const DEPTH_STEP: f32 = 1.0 / 16.0;

/// One cascade of the sun's shadow: a square of the level seen from the sun, placed every frame as the engine places it
/// (`compute_caster_model_fixed`), then snapped to the map's own texels, so moving the camera slides the map a whole
/// texel at a time and its edges do not shimmer.
#[derive(Clone, Copy, Debug)]
pub struct SunCascade {
  /// What the cascade is drawn from: looking along the light, the square's width across, depth reversed.
  pub view: CameraView,
  /// Metres one texel of its map is across.
  pub texel: f32,
  /// Bumped whenever the map moved, so its casters are culled and drawn again.
  pub version: u64,
  /// What it was last fitted by: its square's place, width, resolution, reach and the light.
  key: [f32; 9],
}

impl Default for SunCascade {
  fn default() -> Self {
    Self {
      view: CameraView {
        position: Vec3::ZERO,
        view: Mat4::IDENTITY,
        projection: Mat4::IDENTITY,
      },
      texel: 0.0,
      version: 0,
      key: [f32::NAN; 9],
    }
  }
}

impl SunCascade {
  /// Fits the cascade to the camera, starting where the view's edges left the cascade before it, and carries them on.
  ///
  /// `direction` is where the sun's light travels; `width` and `resolution` are the map's metres and texels across;
  /// `reach` is how far towards the sun past the map its casters may stand, and away from it its receivers.
  pub fn fit(
    &mut self,
    camera: &CameraView,
    rays: &mut SunViewRays,
    direction: Vec3,
    width: f32,
    resolution: u32,
    reach: f32,
  ) {
    let basis: SunCascadeBasis = SunCascadeBasis::new(direction);
    let look: Vec3 = -camera.view.inverse().z_axis.truncate().normalize_or_zero();
    // The engine's light stands over the camera: the square starts centred on it, and is then moved across the light.
    let near: [_; 4] = rays.near;
    let center: Vec3 = place_sun_cascade(camera.position, look, &mut rays.rays, &near, &basis, width);
    let texel: f32 = width / resolution.max(1) as f32;
    let step: f32 = width * DEPTH_STEP;
    let x: f32 = (center.dot(basis.right) / texel).round() * texel;
    let y: f32 = (center.dot(basis.up) / texel).round() * texel;
    let z: f32 = (center.dot(basis.light) / step).round() * step;
    let half: f32 = width / 2.0;
    // A step further each way than the rounding strays, towards the sun as far as the reach and below the centre as
    // far again and the engine's margin: a camera flying high still has its ground in the map.
    let back: f32 = reach + step;
    let far: f32 = 2.0 * back + DEPTH * width;
    let snapped: Vec3 = basis.right * x + basis.up * y + basis.light * z;
    let eye: Vec3 = snapped - basis.light * back;
    let key: [f32; 9] = [
      x,
      y,
      z,
      width,
      resolution as f32,
      reach,
      basis.light.x,
      basis.light.y,
      basis.light.z,
    ];

    if key != self.key {
      self.key = key;
      self.version += 1;
    }

    self.texel = texel;
    self.view = CameraView {
      position: eye,
      view: glam::camera::rh::view::look_at_mat4(eye, snapped, basis.up),
      // Near and far swapped reverse the depth, as the camera's own is.
      projection: glam::camera::rh::proj::directx::orthographic(-half, half, -half, half, far, 0.0),
    };
  }
}
