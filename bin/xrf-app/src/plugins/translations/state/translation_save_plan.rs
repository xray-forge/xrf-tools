use std::collections::HashMap;
use std::sync::Arc;

use xrf_translation::{TranslationProjectDescriptor, TranslationProjectMode, TranslationSource};
use xrf_vfs::XrayRoots;

use crate::core::session::DocumentSnapshot;

/// What one save was addressed to, taken as a single read before the filesystem work begins.
#[derive(Debug)]
pub struct TranslationSavePlan {
  pub project: Arc<DocumentSnapshot<TranslationProjectDescriptor>>,
  /// The logical file being saved, keyed as the project keys it.
  pub file: String,
  pub roots: XrayRoots,
  pub prefix: String,
  pub mode: TranslationProjectMode,
  /// Language to the file holding it, for the one logical file being saved.
  pub sources: HashMap<String, TranslationSource>,
}
