/// One emitter of a scene as the control thread plays it.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioEmitterReport {
  pub id: u64,
  /// Its linear gain after volume, attenuation and smoothing, before its bus and the scene's fade.
  pub gain: f32,
  /// Whether it holds a voice on the device.
  pub is_real: bool,
  /// Whether its sound is still being read.
  pub is_loading: bool,
  /// Times its voice was started over where its clock says, having starved or ended early.
  pub restarts: u32,
}
