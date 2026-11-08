use std::path::Path;

/// The native libraries starting the GPU may load: the DXC library D3D12 compiles shaders with, FXC where none is
/// given; and the directory DLSS's runtime library is loaded from, where the application ships one.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GpuLibraries<'a> {
  pub dxc: Option<&'a Path>,
  pub dlss: Option<&'a Path>,
}
