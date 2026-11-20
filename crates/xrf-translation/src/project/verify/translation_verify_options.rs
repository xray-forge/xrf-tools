use crate::language::translation_language::TranslationLanguage;
use crate::project::events::translation_listener::TranslationListener;

pub struct TranslationVerifyOptions {
  /// Where progress goes and where cancellation comes from.
  pub job: xrf_job::JobHandle,
  pub is_strict: bool,
  /// What hears the run's events as they happen, or nothing.
  pub listener: Option<TranslationListener>,
  pub language: TranslationLanguage,
  /// Whether to describe every missing translation, rather than only counting them.
  ///
  /// The aggregate is always produced; this decides whether the per-id findings behind it are built.
  /// Verifying an imported two-language mod against all eight languages means 149,979 of them, so a
  /// caller that wants the summary should not pay to assemble the detail on its way to dropping it.
  pub is_detailed: bool,
}
