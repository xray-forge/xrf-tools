use std::collections::HashMap;

use xrf_translation::{TranslationProjectMode, TranslationSource};
use xrf_vfs::XrayRoots;

use crate::plugins::translations::state::translation_session_id::TranslationSessionId;

/// What one save was addressed to, taken as a single read before the filesystem work begins.
#[derive(Debug)]
pub struct TranslationSavePlan {
  pub session_id: TranslationSessionId,
  /// The logical file being saved, keyed as the project keys it.
  pub file: String,
  pub roots: XrayRoots,
  pub prefix: String,
  pub mode: TranslationProjectMode,
  /// Language to the file holding it, for the one logical file being saved.
  pub sources: HashMap<String, TranslationSource>,
}
