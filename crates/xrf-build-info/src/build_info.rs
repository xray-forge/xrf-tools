use std::fmt;

use serde::{Deserialize, Serialize};

use crate::build_kind::BuildKind;

/// Where a binary came from, as recorded when it was compiled.
///
/// Every field is `Option` because a build script may not have run, or a value may be unavailable - a
/// build outside a Git checkout has no commit, and a local build has no workflow run. Reporting the
/// absence is more useful than substituting a plausible-looking default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct BuildInfo {
  /// Crate version of the binary itself.
  pub version: &'static str,
  /// How the binary was produced, which is what separates a fast nightly from a size-optimised release.
  pub kind: BuildKind,
  /// Full commit the sources were at.
  pub commit: Option<&'static str>,
  /// Branch or tag the build ran from.
  pub reference: Option<&'static str>,
  /// Whether the checkout carried uncommitted changes, which only a local build can.
  pub is_dirty: bool,
  /// RFC 3339 instant the build script ran.
  pub built_at: Option<&'static str>,
  /// Target triple the binary runs on.
  pub target: Option<&'static str>,
  /// Compiler that produced it.
  pub rustc: Option<&'static str>,
  /// Cargo profile name, as opposed to the optimisation settings below.
  pub profile: Option<&'static str>,
  /// Optimisation level, link-time optimisation and codegen unit count, as cargo resolved them.
  pub optimization: Option<&'static str>,
  /// Identifier of the workflow run that produced the binary, absent for a local build.
  pub run_id: Option<&'static str>,
}

impl BuildInfo {
  /// First seven characters of the commit, which is what a human quotes.
  pub fn short_commit(&self) -> Option<&'static str> {
    self.commit.map(|commit| &commit[..commit.len().min(7)])
  }
}

impl fmt::Display for BuildInfo {
  /// One `field: value` per line, skipping what this build could not record.
  ///
  /// The first line carries no label because clap prints the binary name in front of it.
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(formatter, "{} ({})", self.version, self.kind.as_str())?;

    if let Some(commit) = self.commit {
      let dirty: &str = if self.is_dirty { " (dirty)" } else { "" };
      let reference: String = self.reference.map(|it| format!(" on {it}")).unwrap_or_default();

      writeln!(formatter, "commit:       {commit}{reference}{dirty}")?;
    }

    for (label, value) in [
      ("built at:    ", self.built_at),
      ("target:      ", self.target),
      ("rustc:       ", self.rustc),
      ("profile:     ", self.profile),
      ("optimization:", self.optimization),
      ("workflow run:", self.run_id),
    ] {
      if let Some(value) = value {
        writeln!(formatter, "{label} {value}")?;
      }
    }

    Ok(())
  }
}

/// Read back the description this crate's [`emit`] wrote for the binary being compiled.
///
/// `option_env!` throughout, so a crate whose build script never ran still compiles and simply reports
/// less rather than failing to build.
#[macro_export]
macro_rules! build_info {
  () => {
    $crate::BuildInfo {
      version: env!("CARGO_PKG_VERSION"),
      kind: $crate::BuildKind::from_recorded(option_env!("XRF_BUILD_KIND")),
      commit: option_env!("XRF_BUILD_COMMIT"),
      reference: option_env!("XRF_BUILD_REF"),
      is_dirty: matches!(option_env!("XRF_BUILD_DIRTY"), Some("true")),
      built_at: option_env!("XRF_BUILD_TIMESTAMP"),
      target: option_env!("XRF_BUILD_TARGET"),
      rustc: option_env!("XRF_BUILD_RUSTC"),
      profile: option_env!("XRF_BUILD_PROFILE"),
      optimization: option_env!("XRF_BUILD_OPTIMIZATION"),
      run_id: option_env!("XRF_BUILD_RUN_ID"),
    }
  };
}
