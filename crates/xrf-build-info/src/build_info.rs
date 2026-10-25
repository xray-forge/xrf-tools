use std::fmt;

use serde::{Deserialize, Serialize};

use crate::build_kind::BuildKind;

/// Where a binary came from, as recorded when it was compiled.
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
  /// Whether `git status` reported anything when the build ran: uncommitted changes or untracked files, in whichever
  /// checkout built it.
  pub is_dirty: bool,
  /// RFC 3339 instant the build script ran.
  pub built_at: Option<&'static str>,
  /// Target triple the binary runs on.
  pub target: Option<&'static str>,
  /// Compiler that produced it.
  pub rustc: Option<&'static str>,
  /// Cargo profile name, as opposed to the optimisation settings below.
  pub profile: Option<&'static str>,
  /// The optimisation level cargo built with, and the link-time optimisation and codegen unit count the environment
  /// overrode, `profile default` where it overrode neither: cargo hands a build script the level, not the profile.
  pub optimization: Option<&'static str>,
  /// Identifier of the workflow run that produced the binary, absent for a local build.
  pub run_id: Option<&'static str>,
}

impl BuildInfo {
  /// First seven characters of the commit, which is what a human quotes, or the whole commit when it is shorter.
  pub fn short_commit(&self) -> Option<&'static str> {
    // Cut at a character boundary: a `GITHUB_SHA` override is not validated as a hash.
    self
      .commit
      .map(|commit| commit.char_indices().nth(7).map_or(commit, |(end, _)| &commit[..end]))
  }
}

impl fmt::Display for BuildInfo {
  /// One `field: value` per line, skipping what this build could not record.
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

/// Read back the description [`BuildInfoEmitter::emit`](crate::BuildInfoEmitter::emit) wrote for the binary being
/// compiled.
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

#[cfg(test)]
mod tests {
  use crate::{BuildInfo, BuildKind};

  fn build(commit: Option<&'static str>) -> BuildInfo {
    BuildInfo {
      version: "1.2.3",
      kind: BuildKind::Development,
      commit,
      reference: Some("main"),
      is_dirty: true,
      built_at: Some("2026-10-25T12:00:00Z"),
      target: None,
      rustc: None,
      profile: Some("release"),
      optimization: None,
      run_id: None,
    }
  }

  #[test]
  fn a_short_commit_is_the_first_seven_characters() {
    assert_eq!(build(Some("1c84ad228f00")).short_commit(), Some("1c84ad2"));
    assert_eq!(
      build(Some("1c84")).short_commit(),
      Some("1c84"),
      "a shorter value is kept whole"
    );
    assert_eq!(build(None).short_commit(), None);
  }

  #[test]
  fn a_short_commit_cuts_an_unvalidated_override_at_a_character() {
    assert_eq!(build(Some("коміт-без-хешу")).short_commit(), Some("коміт-б"));
  }

  #[test]
  fn the_display_skips_what_the_build_could_not_record() {
    assert_eq!(
      build(Some("1c84ad228f00")).to_string(),
      "1.2.3 (development)\n\
       commit:       1c84ad228f00 on main (dirty)\n\
       built at:     2026-10-25T12:00:00Z\n\
       profile:      release\n"
    );
  }

  #[test]
  fn the_display_drops_the_reference_and_dirty_flag_without_a_commit() {
    assert_eq!(
      build(None).to_string(),
      "1.2.3 (development)\n\
       built at:     2026-10-25T12:00:00Z\n\
       profile:      release\n"
    );
  }
}
