use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, StorageValueMut, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::meter_sample::MeterSample;

/// What the spot meter reads and writes: the finished scene, the G-buffer's albedo, the light the sun and the lamps
/// gathered, the depth, the frame's lighting and exposure, and the sample it writes.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct MeterParameters {
  #[texture(d2, unfilterable)]
  pub scene: GraphTexture,
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub light_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[storage]
  pub sample: StorageValueMut<MeterSample>,
}
