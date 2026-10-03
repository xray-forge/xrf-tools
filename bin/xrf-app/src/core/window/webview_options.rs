use serde::{Deserialize, Serialize};

use crate::core::window::WebviewCollectionPace;

/// What the webview replaces when it is given arguments of its own: wry's default, which disables Edge's own UI.
const WRY_DEFAULT_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

/// Which of the webview's optional browser capabilities the main window is built with, chosen in the settings.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewOptions {
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
  /// Frames keep to the display's refresh, and the major collections come often and small.
  fn default() -> Self {
    Self {
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
