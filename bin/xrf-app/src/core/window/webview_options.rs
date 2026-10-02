use serde::{Deserialize, Serialize};

use crate::core::window::WebviewCollectionPace;

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
  /// How early the webview's JavaScript starts marking for a major collection, for every page and worker alike.
  #[serde(default)]
  pub collection_pace: WebviewCollectionPace,
  /// The compositor presents at the display's refresh; lifted, `--disable-gpu-vsync` presents frames as they come.
  #[serde(default = "is_on")]
  pub is_vsync: bool,
  /// The page's animation frames keep to the display's refresh; lifted, `--disable-frame-rate-limit` runs them past it.
  #[serde(default = "is_on")]
  pub is_frame_rate_limited: bool,
}

/// What an option kept on by default reads as where a choice stored before it existed leaves it out.
fn is_on() -> bool {
  true
}

impl Default for WebviewOptions {
  /// A level's pipelines fit the doubled cache and not the default one, frames keep to the display's refresh, and the
  /// major collections come often and small; the developer features are opt-in.
  fn default() -> Self {
    Self {
      is_shader_cache_doubled: true,
      is_webgpu_developer: false,
      collection_pace: WebviewCollectionPace::Frequent,
      is_vsync: true,
      is_frame_rate_limited: true,
    }
  }
}

impl WebviewOptions {
  /// The browser arguments these options build the webview with, wry's default first.
  pub fn to_browser_args(self) -> String {
    let mut args: Vec<String> = vec![WRY_DEFAULT_ARGS.to_owned()];

    if self.is_shader_cache_doubled {
      args.push("--enable-features=AggressiveShaderCacheLimits".to_owned());
    }

    if self.is_webgpu_developer {
      args.push("--enable-webgpu-developer-features".to_owned());
    }

    if !self.is_vsync {
      args.push("--disable-gpu-vsync".to_owned());
    }

    if !self.is_frame_rate_limited {
      args.push("--disable-frame-rate-limit".to_owned());
    }

    if let Some(trigger) = self.collection_pace.soft_trigger() {
      args.push(format!("--js-flags=--incremental-marking-soft-trigger={trigger}"));
    }

    args.join(" ")
  }
}
