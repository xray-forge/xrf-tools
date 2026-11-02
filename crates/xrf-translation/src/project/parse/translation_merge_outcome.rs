use crate::project::parse::translation_parse_census::TranslationParseCensus;

/// What merging one string table into one JSON source changed.
#[derive(Debug, Default)]
pub(crate) struct TranslationMergeOutcome {
  pub inserted: u32,
  pub filled: u32,
  pub unchanged: u32,
  pub conflicted: u32,
  pub placeholders_added: u32,
}

impl TranslationMergeOutcome {
  /// Add this merge's counts to a run's census.
  pub(crate) fn record(&self, census: &mut TranslationParseCensus) {
    census.entries_inserted += self.inserted;
    census.entries_filled += self.filled;
    census.entries_unchanged += self.unchanged;
    census.entries_conflicted += self.conflicted;
    census.placeholders_added += self.placeholders_added;
  }
}
