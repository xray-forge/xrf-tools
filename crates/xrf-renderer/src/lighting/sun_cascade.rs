use glam::{Mat4, Vec3};

use crate::camera::camera_view::CameraView;
use crate::lighting::sun_cascade_basis::SunCascadeBasis;
use crate::lighting::sun_cascade_placement::place_sun_cascade;
use crate::lighting::sun_view_rays::SunViewRays;

/// How far a cascade's map reaches past its reach below its centre, in widths: the engine's `1.41421 * map_size`.
const DEPTH: f32 = std::f32::consts::SQRT_2;

/// The step, in widths, a cascade's place along the light is rounded to, so a move along it alone keeps the map.
const DEPTH_STEP: f32 = 1.0 / 16.0;

/// How far, in widths, a cascade may stray from where its map was drawn and still wait for its stagger to draw it.
const DRIFT: f32 = 1.0 / 16.0;

/// Texels a caster as far towards the sun as the reach may see its shadow move as the sun turns before a map stops
/// holding: a clock running turns the light a little every frame, which alone would draw every cascade every frame.
const TURN_TEXELS: f32 = 0.5;

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
  /// How far down the light its map reaches, from where it is seen.
  far: f32,
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
      far: 0.0,
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
    self.far = far;
    self.view = CameraView {
      position: eye,
      view: glam::camera::rh::view::look_at_mat4(eye, snapped, basis.up),
      // Near and far swapped reverse the depth, as the camera's own is.
      projection: glam::camera::rh::proj::directx::orthographic(-half, half, -half, half, far, 0.0),
    };
  }

  /// Whether a map drawn at `drawn` still holds this fit near enough to wait for its turn: drawn at the same width,
  /// resolution and reach, strayed from it by a sixteenth of the width at most along each axis, and under a light turned
  /// by no more than moves a caster as far as the reach `TURN_TEXELS` across. A sharp turn moves the square further,
  /// and its view would fall out of the map until the map's turn came.
  pub fn is_held_by(&self, drawn: &SunCascade) -> bool {
    self.is_held_within(drawn, 1.0)
  }

  /// Whether a map drawn at `drawn` holds this fit within `share` of what `is_held_by` allows: a map about to stop
  /// holding has its successor drawn ahead.
  pub fn is_held_within(&self, drawn: &SunCascade, share: f32) -> bool {
    let [x, y, z, width, resolution, reach, light @ ..] = self.key;
    let [
      drawn_x,
      drawn_y,
      drawn_z,
      drawn_width,
      drawn_resolution,
      drawn_reach,
      drawn_light @ ..,
    ] = drawn.key;
    let slack: f32 = width * DRIFT * share;
    // The turn's sine, as a cosine this near one rounds to it.
    let turn: f32 = Vec3::from(light).cross(Vec3::from(drawn_light)).length();

    width == drawn_width
      && resolution == drawn_resolution
      && reach == drawn_reach
      && Vec3::from(light).dot(Vec3::from(drawn_light)) > 0.0
      && turn <= TURN_TEXELS * share * self.texel / reach.max(1.0)
      && [x - drawn_x, y - drawn_y, z - drawn_z]
        .iter()
        .all(|offset| offset.abs() <= slack)
  }

  /// The camera rows `first` to `last` (past the end) of its map `resolution` texels across are drawn by: its own, its
  /// projection narrowed to those rows, so their texels land where the whole map's would.
  pub fn get_rows_view(&self, (first, last): (u32, u32), resolution: u32) -> CameraView {
    let width: f32 = self.key[3];
    let half: f32 = width / 2.0;
    let texel: f32 = width / resolution.max(1) as f32;

    CameraView {
      projection: glam::camera::rh::proj::directx::orthographic(
        -half,
        half,
        half - last as f32 * texel,
        half - first as f32 * texel,
        self.far,
        0.0,
      ),
      ..self.view
    }
  }
}
