use crate::XrayMount;

/// One source a search reaches, and the step that reaches it.
#[derive(Clone, Copy, Debug)]
pub struct XraySearchedSource<'a> {
  /// Label of the search step that reaches this source.
  pub step: &'a str,
  /// The mounted source itself, which answers for what it holds and how the plan described it.
  pub mount: &'a XrayMount,
}
