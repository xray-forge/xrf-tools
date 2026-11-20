use crate::diagnostics::memory::WebviewProcessKind;

/// One process the webview runs in, as its environment lists it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WebviewProcess {
  pub kind: WebviewProcessKind,
  pub pid: u32,
}
