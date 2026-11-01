use crate::feature::weather::water::enhanced::water_flow::WaterFlow;
use crate::feature::weather::water::reflection::water_reflection_trace::WaterReflectionTrace;
use crate::lighting::render_wind::RenderWind;

/// What the enhanced water plays with this frame besides its settings: the weather's wind and rain, how far it has
/// flowed, and its reflection.
#[derive(Clone, Copy, Debug)]
pub struct EnhancedWaterFrame {
  pub wind: RenderWind,
  /// How hard it rains on it.
  pub rain: f32,
  pub flow: WaterFlow,
  /// Whether its reflection draws this frame, and whether its history holds a frame.
  pub is_reflected: bool,
  pub has_history: bool,
  pub trace: WaterReflectionTrace,
}
