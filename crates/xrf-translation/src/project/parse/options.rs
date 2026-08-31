use std::path::PathBuf;

use xrf_vfs::XrayRoots;

use crate::language::TranslationLanguage;

/// What one import run was asked to do.
pub struct ProjectParseOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub output: xrf_output::OutputOptions,
  /// Roots holding the raw XML, read through the VFS so an installation imports like a loose tree.
  pub roots: XrayRoots,
  /// Where inside those roots to look, or nothing to let the run resolve it.
  ///
  /// See `scope::resolve`: an absent prefix falls back to the gamedata text prefix when the roots
  /// have one, and then descends into a directory named for the language.
  pub prefix: Option<String>,
  /// The language every entry this run reads is filed under. Never `All`.
  pub language: TranslationLanguage,
  /// Directory the JSON sources are written to, which may already hold some.
  pub output_dir: PathBuf,
  /// Restrict the run to one table, by the file name it has in the scope.
  pub file: Option<String>,
  /// Let incoming text replace existing text that differs, instead of keeping what is there.
  pub is_overwrite: bool,
  /// Do everything except write, so a caller can see what a run would change.
  pub is_dry_run: bool,
}
