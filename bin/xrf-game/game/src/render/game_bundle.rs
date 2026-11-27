use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_renderer::RenderBundle;
use xrf_renderer_host::PackagedBundle;

use crate::directories::game_resource::GameResource;

/// The renderer's bundle as the game finds it: its assets from the renderer's sources until the game is packaged, and
/// DXC and DLSS's runtime beside the executable, else where `cargo make fetch-sdks` puts them in the checkout, as a
/// packaged app maps them.
pub struct GameBundle {
  bundle: PackagedBundle,
}

impl GameBundle {
  /// The native libraries the renderer names within its bundle, and where the game finds each. DLSS loads from its
  /// runtime's folder, which holds ray reconstruction's library beside it.
  const LIBRARIES: [GameResource; 2] = [
    GameResource {
      packaged: "dxc/dxcompiler.dll",
      checkout: concat!(env!("CARGO_MANIFEST_DIR"), "/../../../.sdks/dxc/dxcompiler.dll"),
    },
    GameResource {
      packaged: "dlss/nvngx_dlss.dll",
      checkout: concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../.sdks/dlss/lib/Windows_x86_64/rel/nvngx_dlss.dll"
      ),
    },
  ];

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
    match Self::LIBRARIES.iter().find(|library| library.packaged == path) {
      Some(library) => library.locate(),
      None => self.bundle.locate_bundled(path),
    }
  }
}
