use glam::IVec4;
use xrf_renderer_core::ShaderStruct;

/// What a traced hit reads of the level's grass as the view plants it: the slot grid's size and the world slot its
/// first cell stands for, negated; the candidates a slot's side is stepped into; the metres from the eye at which a
/// slot has shrunk to nothing; what every tuft is scaled by; the models, none where no grass is drawn; the slope past
/// which no tuft stands, in radians; the metres from the eye within which the traced cards stand every tuft whole
/// (`GrassCards::get_whole_reach`), where a water's ray meets them and walks no layer, none where no card is traced.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceGrass")]
pub struct TraceGrassUniform {
  pub grid: IVec4,
  pub steps: i32,
  pub fade: f32,
  pub height: f32,
  pub model_count: u32,
  pub slope_limit: f32,
  pub card_reach: f32,
  pub _pad: [f32; 2],
}
