/// How many packets and single rays a batch holds, and the rules its rays follow (`Batch`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RayGpuBatchSize {
  pub(crate) packets: u32,
  pub(crate) singles: u32,
  pub(crate) rules: u32,
  _pad: u32,
}

impl RayGpuBatchSize {
  pub(crate) fn new(packets: u32, singles: u32, rules: u32) -> Self {
    Self {
      packets,
      singles,
      rules,
      _pad: 0,
    }
  }
}
