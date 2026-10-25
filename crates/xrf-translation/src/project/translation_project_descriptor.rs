use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

use crate::project::translation_file::TranslationFile;
use crate::project::translation_finding::TranslationFinding;
use crate::project::translation_project_mode::TranslationProjectMode;
use crate::project::translation_source::TranslationSource;

/// An opened translations root, whichever layout it turned out to have.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationProjectDescriptor {
  pub mode: TranslationProjectMode,
  /// The roots this project was opened over, echoed back so a follow-up read addresses the same trees.
  pub roots: XrayRoots,
  /// Logical prefix the string tables were read from.
  pub prefix: String,
  /// Every language the root offers, in discovery order.
  pub languages: Vec<String>,
  /// The code page each language is written in, which is what limits the characters it can hold.
  ///
  /// Taken from the files themselves in gamedata mode, so a language XRF has never heard of still
  /// reports the encoding its own declaration claims.
  pub encodings: IndexMap<String, String>,
  /// Whether every file this project holds is loose, so an editing session could save all of it.
  ///
  /// One flag rather than a tree of them, so a surface can say up front that a project opened over an
  /// installation is read-only. Which particular file refuses is answered by its source's absent
  /// physical path.
  pub is_editable: bool,
  /// Files keyed by the logical name the layout groups them under.
  pub files: IndexMap<String, TranslationFile>,
  pub findings: Vec<TranslationFinding>,
}

impl TranslationProjectDescriptor {
  /// Recompute the rolled-up editability from the sources that were read.
  ///
  /// Called once, where the read finishes. The flag is derived rather than accumulated so it cannot
  /// disagree with the sources it summarises.
  pub(crate) fn finalize_editable(&mut self) {
    self.is_editable = !self.files.is_empty() && self.files.values().all(TranslationFile::is_editable);
  }

  /// Where one language's copy of a file sits, when the project read one.
  pub fn find_source(&self, file: &str, language: &str) -> Option<&TranslationSource> {
    self.files.get(file)?.sources.get(language)
  }
}
