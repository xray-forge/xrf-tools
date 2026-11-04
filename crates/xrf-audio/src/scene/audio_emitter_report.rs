/// One emitter of a scene as the control thread plays it.
#[derive(Clone, Debug, PartialEq)]
pub struct AudioEmitterReport {
  pub id: u64,
  /// Its linear gain after volume, fade and the distance curve, before walls, its bus and the scene's fade.
  pub gain: f32,
  /// The share of it heard unobstructed, from 0 (wholly behind walls) to 1.
  pub occlusion: f32,
  /// What the walls in its way let through in each band (below 800 Hz, to 8 kHz, above), from 0 to 1.
  pub transmission: [f32; 3],
  /// How much of it the listener's reverb hears.
  pub reverb_send: f32,
  /// Whether it is placed through the HRTF rather than panned.
  pub is_binaural: bool,
  /// Whether it holds a voice on the device.
  pub is_real: bool,
  /// Whether its sound is still being read.
  pub is_loading: bool,
  /// Times its voice was started over where its clock says, having starved or ended early.
  pub restarts: u32,
}
