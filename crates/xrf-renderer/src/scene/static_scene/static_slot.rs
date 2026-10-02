/// One drawn part: a run of clusters, the surface it wears and the batch it is drawn in.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticSlot {
  pub first_cluster: u32,
  pub cluster_count: u32,
  /// The place a single draw stands at; rows name their own.
  pub place: u32,
  /// [`StaticSlot::SINGLE`] or [`StaticSlot::LISTED`].
  pub kind: u32,
  pub batch: u32,
  pub surface: u32,
  pub pad: [u32; 2],
}

impl StaticSlot {
  /// Drawn once, at its own place: a sector's baked geometry.
  pub const SINGLE: u32 = 1;
  /// Drawn at every place its rows name: an instanced tree.
  pub const LISTED: u32 = 2;
}
