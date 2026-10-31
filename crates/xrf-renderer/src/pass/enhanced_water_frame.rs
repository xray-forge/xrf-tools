use crate::lighting::render_wind::RenderWind;
use crate::pass::water_reflection_trace::WaterReflectionTrace;
use crate::scene::level::water_flow::WaterFlow;

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
