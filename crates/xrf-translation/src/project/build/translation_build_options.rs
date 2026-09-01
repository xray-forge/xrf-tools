use std::path::PathBuf;

use crate::language::TranslationLanguage;

pub struct TranslationBuildOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub output: xrf_output::OutputOptions,
  pub is_sorted: bool,
  /// Where the string tables are written, which is always a host directory.
  ///
  /// Sources are read through the VFS and may sit inside a volume; output cannot, because a `.db` has
  /// nowhere to put a file.
  pub output_dir: PathBuf,
  pub language: TranslationLanguage,
}
