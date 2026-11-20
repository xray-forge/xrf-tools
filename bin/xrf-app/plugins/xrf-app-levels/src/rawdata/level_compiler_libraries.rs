use std::path::{Path, PathBuf};

/// The libraries the app hands the level compiler: the renderer's bundled DXC, which lets the GPU steps trace on D3D12.
pub(crate) struct LevelCompilerLibraries {
  pub(crate) dxc: Option<PathBuf>,
}

impl LevelCompilerLibraries {
  /// DXC in the app's resources, as `tauri.windows.conf.json` maps it, and in a development checkout.
  const PACKAGED_DXC: &'static str = "resources/renderer/dxc/dxcompiler.dll";
  const CHECKOUT_DXC: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../.sdks/dxc/dxcompiler.dll");

  /// The libraries in the app's resources, else in the checkout; without DXC the steps trace on Vulkan.
  pub(crate) fn locate(resource_dir: Option<&Path>) -> Self {
    let dxc: Option<PathBuf> = resource_dir
      .map(|directory| directory.join(Self::PACKAGED_DXC))
      .filter(|library| library.is_file())
      .or_else(|| Some(PathBuf::from(Self::CHECKOUT_DXC)).filter(|library| library.is_file()));

    Self { dxc }
  }
}
