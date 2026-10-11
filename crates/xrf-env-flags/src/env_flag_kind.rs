use serde::Serialize;

/// Who sets a variable, and when it is read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EnvFlagKind {
  /// Read by a running binary, set by whoever starts it.
  Runtime,
  /// Read by a third-party library; the application only reports it.
  PassThrough,
  /// Read while the workspace builds, by a build script, `cargo make` or CI.
  Build,
  /// Set by a build script for the compiler, never by hand.
  Internal,
}

impl EnvFlagKind {
  /// Whether a running binary reads it, so its value matters for this run.
  pub fn is_read_at_run(self) -> bool {
    matches!(self, Self::Runtime | Self::PassThrough)
  }
}
