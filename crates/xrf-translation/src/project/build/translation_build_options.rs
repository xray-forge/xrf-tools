use std::path::PathBuf;

use xrf_utils::HostLinkPolicy;

use crate::language::translation_language::TranslationLanguage;

pub struct TranslationBuildOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub output: xrf_output::OutputOptions,
  pub is_sorted: bool,
  /// Where the string tables are written, which is always a host directory.
  pub output_dir: PathBuf,
  pub language: TranslationLanguage,
  /// What to do with a link already below `output_dir`.
  pub link_policy: HostLinkPolicy,
}
