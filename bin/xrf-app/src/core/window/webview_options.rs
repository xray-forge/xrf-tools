use serde::{Deserialize, Serialize};

/// What the webview replaces when it is given arguments of its own: wry's default, which disables Edge's own UI.
const WRY_DEFAULT_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

/// Which of the webview's optional browser capabilities the main window is built with, chosen in the settings.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewOptions {
  /// Chromium's `AggressiveShaderCacheLimits`: the GPU process's pipeline cache doubled, 6 MB to 12 MB on desktop.
  pub is_shader_cache_doubled: bool,
  /// `--enable-webgpu-developer-features`: GPU timestamps unquantized, where WebView2 rounds them to 65.5 µs.
  pub is_webgpu_developer: bool,
}

impl Default for WebviewOptions {
  /// A level's pipelines fit the doubled cache and not the default one; the developer features are opt-in.
  fn default() -> Self {
    Self {
      is_shader_cache_doubled: true,
      is_webgpu_developer: false,
    }
  }
}

impl WebviewOptions {
  /// The browser arguments these options build the webview with, wry's default first.
  pub fn to_browser_args(self) -> String {
    let mut args: Vec<&str> = vec![WRY_DEFAULT_ARGS];

    if self.is_shader_cache_doubled {
      args.push("--enable-features=AggressiveShaderCacheLimits");
    }

    if self.is_webgpu_developer {
      args.push("--enable-webgpu-developer-features");
    }

    args.join(" ")
  }
}
