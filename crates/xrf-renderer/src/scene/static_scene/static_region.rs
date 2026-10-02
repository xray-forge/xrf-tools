/// A batch's run of the visible list: where it starts and how many entries it may hold.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StaticRegion {
  pub base: u32,
  pub capacity: u32,
}
