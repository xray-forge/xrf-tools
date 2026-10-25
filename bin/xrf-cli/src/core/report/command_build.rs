use serde::Serialize;
use xrf_build_info::{BuildInfo, BuildKind};

/// The binary a run came from, as far as comparing two runs needs it; the rest of its identity is `--version`'s.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandBuild {
  version: &'static str,
  kind: BuildKind,
  #[serde(skip_serializing_if = "Option::is_none")]
  commit: Option<&'static str>,
  /// Whether the binary was built from a working tree with changes, which its commit alone does not say.
  is_dirty: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  profile: Option<&'static str>,
  /// The CI run that built the binary, absent from a local build.
  #[serde(skip_serializing_if = "Option::is_none")]
  run_id: Option<&'static str>,
}

impl From<&BuildInfo> for CommandBuild {
  fn from(build: &BuildInfo) -> Self {
    Self {
      version: build.version,
      kind: build.kind,
      commit: build.commit,
      is_dirty: build.is_dirty,
      profile: build.profile,
      run_id: build.run_id,
    }
  }
}
