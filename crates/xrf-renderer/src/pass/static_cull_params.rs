use glam::Vec4;

/// What the cull is told besides the camera: how much there is to test and the level of detail's thresholds.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticCullParams {
  pub cluster_count: u32,
  pub row_count: u32,
  pub batch_count: u32,
  pub impostor_count: u32,
  /// Screen areas a progressive mesh is drawn whole above and at its coarsest below.
  pub glod_start: f32,
  pub glod_end: f32,
  /// The screen area an instanced place is dropped at or below.
  pub discard_below: f32,
  /// Clusters the early phase can set aside for the late one.
  pub candidate_capacity: u32,
  /// Whether what the depth hides is culled.
  pub is_occluding: u32,
  /// Screen areas an impostor draws below, and its trees above.
  pub lod_a: f32,
  pub lod_b: f32,
  /// Whether impostors stand in for distant trees at all.
  pub is_impostors: u32,
  /// A bit a visibility group, from the lowest, set where the view hides it.
  pub hidden_groups: u32,
  pub pad: [u32; 3],
  /// The camera every view's levels of detail are measured from, a shadow's too.
  pub lod_origin: Vec4,
}
