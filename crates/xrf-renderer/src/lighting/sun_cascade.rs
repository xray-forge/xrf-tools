use glam::{Mat4, Vec3};

use crate::camera::camera_view::CameraView;
use crate::lighting::sun_cascade_basis::SunCascadeBasis;
use crate::lighting::sun_cascade_placement::SunCascadePlacement;
use crate::lighting::sun_view_rays::SunViewRays;

/// How far a cascade's map reaches below its centre at least, in widths: the engine's `1.41421 * map_size`.
const DEPTH: f32 = std::f32::consts::SQRT_2;

/// The step, in widths, a cascade's place along the light is rounded to, so a move along it alone keeps the map.
const DEPTH_STEP: f32 = 1.0 / 16.0;

/// How far, in widths, a cascade may stray from where its map was drawn and still wait for its stagger to draw it.
const DRIFT: f32 = 1.0 / 16.0;

/// Texels a caster as far towards the sun as the map reaches may see its shadow move as the sun turns before a map
/// stops holding: a clock running turns the light a little every frame, which alone would draw every cascade every
/// frame.
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
  /// How far towards the sun from its square's centre its map reaches, which its farthest caster stands within.
  back: f32,
  /// How far down the light its map reaches, from where it is seen.
  far: f32,
  /// What it was last fitted by: its square's place, width, resolution, the level's span along the light and the light.
  key: [f32; 10],
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
      back: 0.0,
      far: 0.0,
      key: [f32::NAN; 10],
    }
  }
}

impl SunCascade {
  /// Fits the cascade to the camera, starting where the view's edges left the cascade before it, and carries them on;
  /// through parallel rays, a level map's, centred on the box the view holds, which a width of its diagonal covers
  /// whichever way the light comes.
  ///
  /// `direction` is where the sun's light travels; `width` and `resolution` are the map's metres and texels across;
  /// `span` is how far along the light the level stands, nearest and farthest, which its casters and receivers stand
  /// within: measured from the level rather than the camera, a caster's shadow does not come and go as the camera moves
  /// along the light.
  pub fn fit(
    &mut self,
    camera: &CameraView,
    rays: &mut SunViewRays,
    direction: Vec3,
    width: f32,
    resolution: u32,
    span: Option<(f32, f32)>,
  ) {
    let basis: SunCascadeBasis = SunCascadeBasis::new(direction);
    let look: Vec3 = -camera.view.inverse().z_axis.truncate().normalize_or_zero();
    // The engine's light stands over the camera: the square starts centred on it, and is then moved across the light.
    let near: [_; 4] = rays.near;
    let center: Vec3 = if camera.is_orthographic() {
      let (near_plane, far_plane): (f32, f32) = camera.get_depth_range();

      camera.position + look * ((near_plane + far_plane) / 2.0)
    } else {
      SunCascadePlacement::place(camera.position, look, &mut rays.rays, &near, &basis, width)
    };
    let texel: f32 = width / resolution.max(1) as f32;
    let step: f32 = width * DEPTH_STEP;
    let x: f32 = (center.dot(basis.right) / texel).round() * texel;
    let y: f32 = (center.dot(basis.up) / texel).round() * texel;
    let z: f32 = (center.dot(basis.light) / step).round() * step;
    let half: f32 = width / 2.0;
    // A level holding nothing stands nowhere along the light.
    let (nearest, farthest) = span.unwrap_or((f32::INFINITY, f32::NEG_INFINITY));
    // A step further each way than the rounding strays, towards the sun as far as the level stands and below the centre
    // to its far side, the engine's margin at least: a camera flying high still has its ground in the map.
    let back: f32 = (z - nearest).max(0.0) + step;
    let far: f32 = back + (farthest - z).max(DEPTH * width) + step;
    let snapped: Vec3 = basis.right * x + basis.up * y + basis.light * z;
    let eye: Vec3 = snapped - basis.light * back;
    let key: [f32; 10] = [
      x,
      y,
      z,
      width,
      resolution as f32,
      nearest,
      farthest,
      basis.light.x,
      basis.light.y,
      basis.light.z,
    ];

    if key != self.key {
      self.key = key;
      self.version += 1;
    }

    self.texel = texel;
    self.back = back;
    self.far = far;
    self.view = CameraView {
      position: eye,
      view: glam::camera::rh::view::look_at_mat4(eye, snapped, basis.up),
      // Near and far swapped reverse the depth, as the camera's own is.
      projection: glam::camera::rh::proj::directx::orthographic(-half, half, -half, half, far, 0.0),
    };
  }

  /// Whether a map drawn at `drawn` still holds this fit near enough to wait for its turn: drawn at the same width and
  /// resolution over the same level, strayed from it by a sixteenth of the width at most along each axis, and under a
  /// light turned by no more than moves a caster as far towards the sun as the map reaches `TURN_TEXELS` across. A
  /// sharp turn moves the square further, and its view would fall out of the map until the map's turn came.
  pub fn is_held_by(&self, drawn: &SunCascade) -> bool {
    self.is_held_within(drawn, 1.0)
  }

  /// Whether a map drawn at `drawn` holds this fit within `share` of what `is_held_by` allows: a map about to stop
  /// holding has its successor drawn ahead.
  pub fn is_held_within(&self, drawn: &SunCascade, share: f32) -> bool {
    let [x, y, z, width, resolution, nearest, farthest, light @ ..] = self.key;
    let [
      drawn_x,
      drawn_y,
      drawn_z,
      drawn_width,
      drawn_resolution,
      drawn_nearest,
      drawn_farthest,
      drawn_light @ ..,
    ] = drawn.key;
    let slack: f32 = width * DRIFT * share;
    // The turn's sine, as a cosine this near one rounds to it.
    let turn: f32 = Vec3::from(light).cross(Vec3::from(drawn_light)).length();

    width == drawn_width
      && resolution == drawn_resolution
      && nearest == drawn_nearest
      && farthest == drawn_farthest
      && Vec3::from(light).dot(Vec3::from(drawn_light)) > 0.0
      && turn <= TURN_TEXELS * share * self.texel / self.back.max(1.0)
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
