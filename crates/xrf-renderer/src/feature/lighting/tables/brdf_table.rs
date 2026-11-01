use std::f32::consts::PI;

use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

/// The split-sum table of the one BRDF (`common/brdf`): what share of a uniform environment GGX's lobe, under Smith's
/// height-correlated visibility, reflects towards the eye, as a scale and a bias of the reflectance head on, integrated
/// once (Karis, "Real Shading in Unreal Engine 4", SIGGRAPH 2013) and uploaded for every pass that lights by it. Karis's
/// mobile fit of the same table (`split_sum`) stands up to a fifth under it at middle roughness, which the multiple
/// scattering's gift, divided by it, turned into light a white metal does not reflect.
pub struct BrdfTable {
  pub view: wgpu::TextureView,
}

impl BrdfTable {
  /// Texels each way: the cosine of the view to the normal across, the roughness down, each at its texel's centre.
  pub const SIZE: u32 = 32;
  /// Samples of the lobe a texel is integrated over.
  const SAMPLES: u32 = 1024;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let view: wgpu::TextureView = device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("brdf table"),
          size: wgpu::Extent3d {
            width: Self::SIZE,
            height: Self::SIZE,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::Rg32Float,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        bytemuck::cast_slice(&Self::create_table()),
      )
      .create_view(&Default::default());

    Self { view }
  }

  /// The table, row by roughness, column by the view's cosine, each texel the scale and the bias.
  pub fn create_table() -> Vec<[f32; 2]> {
    (0..Self::SIZE * Self::SIZE)
      .map(|texel| {
        let at = |index: u32| (index as f32 + 0.5) / Self::SIZE as f32;

        Self::integrate(at(texel % Self::SIZE), at(texel / Self::SIZE)).to_array()
      })
      .collect()
  }

  /// The scale and the bias at the cosine of the view to the normal (`facing`) and a roughness: GGX's lobe drawn by
  /// importance over a Hammersley set, each sample weighed by its visibility and split by Schlick's Fresnel weight.
  pub fn integrate(facing: f32, roughness: f32) -> Vec2 {
    let alpha: f32 = roughness * roughness;
    let alpha_squared: f32 = alpha * alpha;
    let to_eye: Vec3 = Vec3::new((1.0 - facing * facing).max(0.0).sqrt(), 0.0, facing);
    let mut sum: Vec2 = Vec2::ZERO;

    for sample in 0..Self::SAMPLES {
      let numbers: Vec2 = Vec2::new(
        sample as f32 / Self::SAMPLES as f32,
        sample.reverse_bits() as f32 / 4_294_967_296.0,
      );
      let azimuth: f32 = 2.0 * PI * numbers.x;
      let cosine: f32 = ((1.0 - numbers.y) / (1.0 + (alpha_squared - 1.0) * numbers.y)).sqrt();
      let sine: f32 = (1.0 - cosine * cosine).max(0.0).sqrt();
      let half_way: Vec3 = Vec3::new(sine * azimuth.cos(), sine * azimuth.sin(), cosine);
      let to_light: Vec3 = 2.0 * to_eye.dot(half_way) * half_way - to_eye;
      let lit: f32 = to_light.z;

      if lit <= 0.0 {
        continue;
      }

      let along: f32 = half_way.z.max(1e-6);
      let across: f32 = to_eye.dot(half_way).max(0.0);
      let visibility: f32 = 0.5
        / (lit * (facing * facing * (1.0 - alpha_squared) + alpha_squared).sqrt()
          + facing * (lit * lit * (1.0 - alpha_squared) + alpha_squared).sqrt());
      let weight: f32 = 4.0 * visibility * lit * across / along;
      let fresnel: f32 = (1.0 - across).powi(5);

      sum += Vec2::new((1.0 - fresnel) * weight, fresnel * weight);
    }

    sum / Self::SAMPLES as f32
  }
}
