use std::time::Duration;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// What one language's build produced.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ProjectBuildLanguageSummary {
  pub language: String,
  /// String tables written for this language, one per source.
  pub files: u32,
  /// Ids compiled into them, counting an id once per file it appears in.
  ///
  /// A missing translation still compiles - to the id itself, which is the engine's own fallback - so
  /// this counts what the tables hold rather than what was translated.
  pub entries: u32,
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBuildResult {
  #[serde(with = "xrf_utils::duration_ms")]
  pub duration: Duration,
  /// Sources read, whatever they built into.
  pub sources: u32,
  /// String tables written, across every language.
  pub files: u32,
  /// Per language, in build order.
  pub languages: Vec<ProjectBuildLanguageSummary>,
  #[serde(skip_serializing)]
  language_index: IndexMap<String, usize>,
}

impl ProjectBuildResult {
  pub fn new() -> Self {
    Self::default()
  }

  /// Record one written string table against the language it belongs to.
  pub(crate) fn record_built_file(&mut self, language: &str, entries: u32) {
    self.files += 1;

    match self.language_index.get(language) {
      Some(&index) => {
        let summary: &mut ProjectBuildLanguageSummary = &mut self.languages[index];

        summary.files += 1;
        summary.entries += entries;
      }
      None => {
        self.language_index.insert(language.to_owned(), self.languages.len());
        self.languages.push(ProjectBuildLanguageSummary {
          language: language.to_owned(),
          files: 1,
          entries,
        });
      }
    }
  }

  pub(crate) fn merge(&mut self, other: Self) {
    self.sources += other.sources;

    // Merged by language rather than appended, so one language built from several sources stays one
    // row however many results were combined to get there.
    for summary in other.languages {
      match self.language_index.get(&summary.language) {
        Some(&index) => {
          let existing: &mut ProjectBuildLanguageSummary = &mut self.languages[index];

          existing.files += summary.files;
          existing.entries += summary.entries;
          self.files += summary.files;
        }
        None => {
          self
            .language_index
            .insert(summary.language.clone(), self.languages.len());
          self.files += summary.files;
          self.languages.push(summary);
        }
      }
    }
  }
}
