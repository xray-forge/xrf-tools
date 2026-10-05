/// Where a shared struct is bound, which decides the layout rules it must also meet: uniform buffers add a sixteen-byte
/// array stride and struct alignment that storage buffers do not ask.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderAddressSpace {
  Uniform,
  Storage,
}

impl ShaderAddressSpace {
  pub fn get_wgsl(self) -> &'static str {
    match self {
      Self::Uniform => "uniform",
      Self::Storage => "storage, read",
    }
  }
}
