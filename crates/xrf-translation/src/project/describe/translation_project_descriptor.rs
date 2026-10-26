use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;
use xrf_utils::XRayEncoding;
use xrf_vfs::XrayRoots;
use xrf_xml::{XmlEncoding, XmlEscape};

use crate::language::translation_code_page::TranslationCodePage;
use crate::project::describe::translation_file::TranslationFile;
use crate::project::describe::translation_finding::TranslationFinding;
use crate::project::describe::translation_project_mode::TranslationProjectMode;
use crate::project::describe::translation_source::TranslationSource;
use crate::xml::translation_line_limit::TranslationLineLimit;

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
  /// The one the game draws a language XRF builds with; for one it does not, what that language's own files declare.
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
  /// Bytes a string table's text line holds beside the text: the build's `    <text>` and `</text>`.
  ///
  /// A shipped table indented with tabs spends a byte or two less, so a text this passes fits there too.
  const TEXT_LINE_OVERHEAD: usize = 17;

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

  /// Why `text` cannot be written in one language, or nothing when it can: a character the language's code page
  /// lacks, or a text too long for the line a string table holds it on.
  ///
  /// The descriptor answers because the code page is what each language's own files declared when the project was
  /// read, which is the only statement that exists for a language XRF does not build. A language with none is not
  /// judged.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when a language declares a code page there is no encoder for.
  pub fn find_unwritable(&self, language: &str, text: &str) -> XrfResult<Option<String>> {
    let Some(label) = self.encodings.get(language) else {
      return Ok(None);
    };

    let encoding: XRayEncoding = XmlEncoding::from_label(label)?;

    if let Some(character) = TranslationCodePage::find_unencodable(text, encoding) {
      return Ok(Some(format!(
        "{} cannot be written in {language}",
        TranslationCodePage::describe(character)
      )));
    }

    let length: usize = encoding.encode(&XmlEscape::text(text)).0.len() + Self::TEXT_LINE_OVERHEAD;

    if length > TranslationLineLimit::MAX_BYTES {
      return Ok(Some(format!(
        "The text is {length} bytes on its line, past the {} the game's XML reader takes",
        TranslationLineLimit::MAX_BYTES
      )));
    }

    Ok(None)
  }
}
