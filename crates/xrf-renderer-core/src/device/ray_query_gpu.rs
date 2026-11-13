/// A GPU by its PCI vendor and device ids, as an adapter reports them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RayQueryGpu {
  pub vendor: u32,
  pub device: u32,
}
