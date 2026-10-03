use glam::{IVec2, IVec4, Vec4};

/// Where the camera stands and what the grass's planting is set to, as `shaders/grass/records.wgsl` reads it:
/// `CDetailManager`'s terms.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GrassUniform {
  /// The eye, in the engine's space: renderer space with `z` negated.
  pub eye: Vec4,
  /// The view's six planes in renderer space, pointing in.
  pub planes: [Vec4; 6],
  /// The slot the camera stands over, `iFloor(EYE / dm_slot_size + 0.5)`, on each axis.
  pub center: IVec2,
  /// `dm_size`: slots planted each way from the camera's.
  pub reach: i32,
  /// `d_size`: steps a slot's candidates are laid across.
  pub steps: i32,
  /// The grid's size in slots, and the world slot its first cell stands for, negated.
  pub grid: IVec4,
  /// `dm_fade`: metres from the eye at which a slot has shrunk to nothing.
  pub fade: f32,
  /// How far a candidate is jittered off its step, `density / 1.7`.
  pub jitter: f32,
  pub height: f32,
  /// `r_ssaDISCARD`, as the static cull compares it.
  pub discard_below: f32,
  /// Counts what a slot plants changing, so every slot held is planted again.
  pub generation: u32,
  pub per_cell: u32,
  pub bands: u32,
  pub capacity: u32,
  pub model_count: u32,
  /// Metres the largest tuft reaches past its ground at a height of one.
  pub tuft_reach: f32,
  pub pad: [u32; 2],
}
