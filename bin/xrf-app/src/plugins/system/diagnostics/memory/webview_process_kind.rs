use serde::Serialize;

/// What a webview process does, as WebView2 names it (`COREWEBVIEW2_PROCESS_KIND`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum WebviewProcessKind {
  /// The browser process, which owns the others.
  Browser,
  /// Runs the page and its workers: the script heaps and every array buffer.
  Renderer,
  /// Runs the GPU driver on the page's behalf: WebGPU buffers and textures, and their staging.
  Gpu,
  /// A service process, such as the network or audio service.
  Utility,
  /// Helps the sandbox start the others.
  SandboxHelper,
  /// Hosts a plugin, which nothing this application loads uses.
  PpapiPlugin,
  /// Brokers a plugin, which nothing this application loads uses.
  PpapiBroker,
  /// A kind newer than this build knows.
  Other,
}

#[cfg(windows)]
impl From<webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_PROCESS_KIND> for WebviewProcessKind {
  fn from(kind: webview2_com::Microsoft::Web::WebView2::Win32::COREWEBVIEW2_PROCESS_KIND) -> Self {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
      COREWEBVIEW2_PROCESS_KIND_BROWSER, COREWEBVIEW2_PROCESS_KIND_GPU, COREWEBVIEW2_PROCESS_KIND_PPAPI_BROKER,
      COREWEBVIEW2_PROCESS_KIND_PPAPI_PLUGIN, COREWEBVIEW2_PROCESS_KIND_RENDERER,
      COREWEBVIEW2_PROCESS_KIND_SANDBOX_HELPER, COREWEBVIEW2_PROCESS_KIND_UTILITY,
    };

    match kind {
      COREWEBVIEW2_PROCESS_KIND_BROWSER => Self::Browser,
      COREWEBVIEW2_PROCESS_KIND_RENDERER => Self::Renderer,
      COREWEBVIEW2_PROCESS_KIND_GPU => Self::Gpu,
      COREWEBVIEW2_PROCESS_KIND_UTILITY => Self::Utility,
      COREWEBVIEW2_PROCESS_KIND_SANDBOX_HELPER => Self::SandboxHelper,
      COREWEBVIEW2_PROCESS_KIND_PPAPI_PLUGIN => Self::PpapiPlugin,
      COREWEBVIEW2_PROCESS_KIND_PPAPI_BROKER => Self::PpapiBroker,
      _ => Self::Other,
    }
  }
}
