use std::path::{Path, PathBuf};

/// The libraries the app hands the level compiler: DXC's, which the details step's GPU backend traces on D3D12 with,
/// as the app bundles it for the renderer.
pub(crate) struct LevelCompilerLibraries {
  pub(crate) dxc: Option<PathBuf>,
}

impl LevelCompilerLibraries {
  /// Where the app bundles DXC, as `tauri.windows.conf.json` maps it; and where a development build finds it in the
  /// checkout.
  const PACKAGED_DXC: &'static str = "resources/renderer/dxc/dxcompiler.dll";
  const CHECKOUT_DXC: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../../.sdks/dxc/dxcompiler.dll");

  /// The libraries under the app's resources, or the checkout's where nothing was packaged; DXC none where neither
  /// holds it, and the details then trace on Vulkan.
  pub(crate) fn locate(resource_dir: Option<&Path>) -> Self {
    let dxc: Option<PathBuf> = resource_dir
      .map(|directory| directory.join(Self::PACKAGED_DXC))
      .filter(|library| library.is_file())
      .or_else(|| Some(PathBuf::from(Self::CHECKOUT_DXC)).filter(|library| library.is_file()));

    Self { dxc }
  }
}
