use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_renderer::RenderBundle;
use xrf_renderer_host::PackagedBundle;

use crate::directories::game_resource::GameResource;

/// The renderer's bundle as the game finds it: its assets from the renderer's sources until the game is packaged, and
/// DXC beside the executable, else where `cargo make fetch-sdks` puts it in the checkout, as a packaged app maps it.
pub struct GameBundle {
  bundle: PackagedBundle,
}

impl GameBundle {
  /// DXC's library as the renderer names it within its bundle, and where the game finds it.
  const DXC: GameResource = GameResource {
    packaged: "dxc/dxcompiler.dll",
    checkout: concat!(env!("CARGO_MANIFEST_DIR"), "/../../../.sdks/dxc/dxcompiler.dll"),
  };

  pub fn locate() -> Self {
    Self {
      bundle: PackagedBundle::locate(None),
    }
  }
}

impl RenderBundle for GameBundle {
  fn read_bundled(&self, path: &str) -> XrfResult<Vec<u8>> {
    self.bundle.read_bundled(path)
  }

  fn locate_bundled(&self, path: &str) -> Option<PathBuf> {
    if path == Self::DXC.packaged {
      return Self::DXC.locate();
    }

    self.bundle.locate_bundled(path)
  }
}
