use crate::plugins::system::diagnostics::memory::WebviewProcessKind;

/// One process the webview runs in, as its environment lists it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WebviewProcess {
  pub kind: WebviewProcessKind,
  pub pid: u32,
}
