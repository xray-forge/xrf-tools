use glam::{Vec3, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::lighting::material_slice::material_slice;
use crate::lighting::srgb::srgb_encoded;

/// Where the calibration props stand and what they are, as `shaders/frame/calibration.wgsl` reads them: three spheres,
/// then the colour chart.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Calibration")]
pub struct CalibrationUniform {
  /// Each prop's centre in the world, and its radius (a sphere) or half its width (the chart).
  pub props: [Vec4; CalibrationUniform::PROPS],
  /// Each prop's albedo as a colour texture holds it (sRGB encoded), and its gloss.
  pub surfaces: [Vec4; CalibrationUniform::PROPS],
  /// Each prop's material slice, as `material_slice` packs it.
  pub slices: Vec4,
  /// The chart's axis across, and nought.
  pub right: Vec4,
  /// The chart's axis up, and half its height.
  pub up: Vec4,
}

impl CalibrationUniform {
  /// The props drawn: the spheres, then the chart.
  pub const PROPS: usize = 4;
  /// The props drawn as spheres, by their index; the chart follows them.
  pub const SPHERES: u32 = 3;
  /// The grey card's albedo, middle grey.
  pub const GREY: f32 = 0.18;
  /// The white sphere's albedo.
  pub const WHITE: f32 = 0.8;
  /// The mirror sphere's albedo, a polished metal's reflectance head on.
  pub const MIRROR: f32 = 0.9;
  /// The spheres' radius, metres.
  pub const RADIUS: f32 = 0.25;
  /// The chart's half width and half height, metres: the reference chart's proportions, larger, so it reads from
  /// metres away.
  pub const CHART_HALF: (f32, f32) = (0.6, 0.4);
  /// Metres ahead of the camera the props stand, between the spheres' centres, and between the grey sphere and the
  /// chart's edge.
  pub const AHEAD: f32 = 3.0;
  pub const SPACING: f32 = 0.65;
  pub const CHART_CLEARANCE: f32 = 0.45;
  /// A matte surface's gloss and the mirror's.
  pub const MATTE_GLOSS: f32 = 2.0 / 255.0;
  pub const MIRROR_GLOSS: f32 = 1.0;
  /// The engine's material a dielectric prop is lit by (`Blinn`) and the mirror's (`metal`).
  pub const DIELECTRIC_MATERIAL: f32 = 1.0;
  pub const METAL_MATERIAL: f32 = 3.0;

  /// The props standing `AHEAD` metres along `forward` from `eye`, the camera's view levelled, in a row across it: the
  /// grey sphere straight ahead, where the spot meter reads, the white and the mirror sphere to its left, and the
  /// chart to its right, facing back towards the eye.
  pub fn new(eye: Vec3, forward: Vec3) -> Self {
    let level: Vec3 = Vec3::new(forward.x, 0.0, forward.z)
      .try_normalize()
      .unwrap_or(Vec3::NEG_Z);
    let right: Vec3 = level.cross(Vec3::Y).normalize();
    let centre: Vec3 = eye + level * Self::AHEAD;
    let sphere = |step: f32| -> Vec4 { (centre + right * (step * Self::SPACING)).extend(Self::RADIUS) };
    let chart: Vec3 = centre + right * (Self::RADIUS + Self::CHART_CLEARANCE + Self::CHART_HALF.0);
    let matte = |albedo: f32| -> Vec4 { Vec3::splat(srgb_encoded(albedo)).extend(Self::MATTE_GLOSS) };
    let mirror: f32 = srgb_encoded(Self::MIRROR);
    let dielectric: f32 = material_slice(Self::DIELECTRIC_MATERIAL);

    Self {
      props: [
        sphere(0.0),
        sphere(-1.0),
        sphere(-2.0),
        chart.extend(Self::CHART_HALF.0),
      ],
      surfaces: [
        matte(Self::GREY),
        matte(Self::WHITE),
        Vec3::splat(mirror).extend(Self::MIRROR_GLOSS),
        Vec3::ONE.extend(Self::MATTE_GLOSS),
      ],
      slices: Vec4::new(dielectric, dielectric, material_slice(Self::METAL_MATERIAL), dielectric),
      // The chart faces back along `level`: its axis across runs from the eye's left to its right as seen.
      right: right.extend(0.0),
      up: Vec3::Y.extend(Self::CHART_HALF.1),
    }
  }
}
