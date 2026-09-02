use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use xrf_vfs::XrayRoots;

use crate::types::TranslationEntry;

/// Which layout a translations root is read with.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum TranslationProjectMode {
  /// XRF sources: multi-language JSON and language-suffixed XML side by side in one tree.
  #[default]
  Source,
  /// Shipped gamedata: `text\<language>\*.xml`, where the directory carries the language.
  Gamedata,
}

impl TranslationProjectMode {
  /// Logical prefix this layout keeps its string tables under.
  ///
  /// The same two prefixes `DialogProjectMode` resolves dialog text with, and intentionally so: a
  /// dialog's `<text>` key is looked up in exactly these files, so the two crates disagreeing about
  /// where they live would make a phrase resolve in one tool and not the other.
  pub const fn get_prefix(&self) -> &'static str {
    match self {
      Self::Source => "translations",
      Self::Gamedata => r"configs\text",
    }
  }
}

/// Something worth reporting about a file that was opened anyway.
///
/// The reader refuses nothing on content: an editor that will not open the file you need to fix is
/// no use, and the build and verifier keep their own guards.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationFinding {
  pub rule: String,
  pub subject: Option<String>,
  pub message: String,
}

impl TranslationFinding {
  pub fn new(rule: impl Into<String>, subject: Option<String>, message: impl Into<String>) -> Self {
    Self {
      rule: rule.into(),
      subject,
      message: message.into(),
    }
  }
}

/// Where one language's copy of a file was actually found.
///
/// Two paths because they answer different questions. The logical path is the engine identity, which
/// is what the file is; the physical path is where it happens to sit on this machine, which exists
/// only when the winning mount is a loose directory. An archived winner has none, and that absence is
/// the write guard — bytes inside a `.db` volume cannot be edited in place.
///
/// **The physical path is for showing, never for addressing.** It is portable-formatted, so it has
/// already lost any name that is not valid Unicode and any `\` a host treats as an ordinary character.
/// A write resolves the logical path through the VFS instead and asks the asset, which still holds the
/// real one — see `apply_edits_to_asset`.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationSource {
  pub logical_path: String,
  pub physical_path: Option<String>,
}

impl TranslationSource {
  pub fn new(logical_path: impl Into<String>, physical_path: Option<String>) -> Self {
    Self {
      logical_path: logical_path.into(),
      physical_path,
    }
  }

  /// Whether an edit could write this copy back.
  pub fn is_editable(&self) -> bool {
    self.physical_path.is_some()
  }
}

/// One logical translation file, and where each language's copy of it lives.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationFile {
  /// Language to the source holding it. A JSON source lists every language it carries against the
  /// same one.
  ///
  /// Editability is per language rather than per file: an installation layered under a loose tree can
  /// serve one language from a `.db` volume and the next from disk.
  pub sources: IndexMap<String, TranslationSource>,
  pub entries: IndexMap<String, TranslationEntry>,
}

impl TranslationFile {
  /// Whether every language's copy of this file could be written back.
  ///
  /// Not on the wire. A caller edits one language at a time, so what it needs is whether *that*
  /// language's source is loose, which `sources` already says; a rolled-up per-file flag would be the
  /// weaker answer to the narrower question.
  pub fn is_editable(&self) -> bool {
    !self.sources.is_empty() && self.sources.values().all(TranslationSource::is_editable)
  }
}

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
