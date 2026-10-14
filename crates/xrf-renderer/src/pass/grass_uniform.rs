use glam::{IVec2, IVec4, Vec4};
use xrf_renderer_core::ShaderStruct;

/// Where the camera stands and what the grass's planting is set to, as `shaders/grass/planting.wgsl` reads it:
/// `CDetailManager`'s terms.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "GrassParams")]
pub struct GrassUniform {
  /// The eye, in the engine's space: renderer space with `z` negated.
  pub eye: Vec4,
  /// The view's six planes in renderer space, pointing in.
  pub planes: [Vec4; 6],
  /// The slot the camera stands over, `iFloor(EYE / dm_slot_size + 0.5)`, on each axis.
  pub center: IVec2,
  /// `dm_size`: slots planted each way from the camera's.
  pub reach: i32,
  /// `d_size`: steps a slot's candidates are laid across, one more candidate than steps each way.
  pub steps: i32,
  /// The grid's size in slots, and the world slot its first cell stands for, negated.
  pub grid: IVec4,
  /// `dm_fade`: metres from the eye at which a slot has shrunk to nothing.
  pub fade: f32,
  /// How far a candidate is jittered off its step, `density / 1.7`.
  pub jitter: f32,
  /// What every tuft is scaled by, `ps_current_detail_height`.
  pub height: f32,
  /// `r_ssaDISCARD`, as the static cull compares it.
  pub discard_below: f32,
  /// Counts what a slot plants changing, so every slot held is planted again.
  pub generation: u32,
  /// Tufts a cell of the ring holds, rings of cells around the camera's, items the lists hold, and models.
  pub per_cell: u32,
  pub bands: u32,
  pub capacity: u32,
  pub model_count: u32,
  /// Metres the largest tuft reaches past its ground at a height of one.
  pub tuft_reach: f32,
  /// Radians of slope past which a tuft is not planted, thinning over the last fifth; a right angle for none.
  pub slope_limit: f32,
  /// Metres from the eye within which the cards stand a tuft, the last of them over which it shrinks away, and
  /// the slots each way from the eye's that reach spans (`GrassCards::get_slots`).
  pub card_reach: f32,
  pub card_fade: f32,
  pub card_slots: i32,
  pub _pad: [u32; 2],
}
