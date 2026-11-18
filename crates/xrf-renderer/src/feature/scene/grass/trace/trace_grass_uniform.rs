use glam::IVec4;
use xrf_renderer_core::ShaderStruct;

/// What a traced hit reads of the level's grass as the view plants it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceGrass")]
pub(crate) struct TraceGrassUniform {
  /// The slot grid's size, and the world slot its first cell stands for, negated.
  pub grid: IVec4,
  /// The candidates a slot's side is stepped into.
  pub steps: i32,
  /// Metres from the eye at which a slot has shrunk to nothing.
  pub fade: f32,
  /// What every tuft is scaled by.
  pub height: f32,
  /// None where no grass is drawn.
  pub model_count: u32,
  /// The slope past which no tuft stands, in radians.
  pub slope_limit: f32,
  /// Metres within which the traced cards stand every tuft whole (`GrassCards::get_whole_reach`), so a water's ray
  /// meets them and walks no layer; none where no card is traced.
  pub card_reach: f32,
  pub _pad: [f32; 2],
}
