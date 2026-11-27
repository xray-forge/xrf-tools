use env_filter::Filter;
use log::{Level, Metadata, Record};
use xrf_logging::LogFilter;

use crate::logging::ui::UiLog;

/// The backend's filter rules as the UI log asks them: whether a record is written, and the most verbose level any UI
/// record can be written at, which the rules fix for the run.
pub(super) struct UiLogFilter {
  filter: Filter,
  level: Option<Level>,
}

impl UiLogFilter {
  /// The levels the webview logs at, most verbose first.
  const LEVELS: [Level; 4] = [Level::Debug, Level::Info, Level::Warn, Level::Error];

  /// The filter `rules` make for the UI log.
  pub(super) fn new(rules: &LogFilter) -> Self {
    let filter: Filter = rules.build_filter();
    let level: Option<Level> = Self::find_level(&filter, &rules.list_names());

    Self { filter, level }
  }

  /// The most verbose level any UI record can pass at, none where none can ([`Self::find_level`]); it may over-answer,
  /// never under-answer, and [`Self::matches`] decides each record.
  pub(super) fn get_level(&self) -> Option<Level> {
    self.level
  }

  /// Whether `record` is written.
  pub(super) fn matches(&self, record: &Record<'_>) -> bool {
    self.filter.matches(record)
  }

  /// The most verbose level at which `filter` lets some UI record through, given the directive `names` its rules list.
  /// A record is ruled by the directive with the longest name its target starts with. For a scoped target that is
  /// either a directive whose name is a prefix of `xrf_ui::`, the one asking for `xrf_ui::` itself finds, or one whose
  /// name starts with `xrf_ui::`, which rules the target spelled as its name. Asking for `xrf_ui`, `xrf_ui::` and each
  /// such name therefore reaches every directive that can rule a UI record. A `/` message filter is not asked, which
  /// can only make the answer more verbose than any record turns out to be written at.
  fn find_level(filter: &Filter, names: &[&str]) -> Option<Level> {
    let scoped = names.iter().filter(|name| name.starts_with(UiLog::SCOPE_PREFIX));
    let targets: Vec<&str> = [UiLog::TARGET, UiLog::SCOPE_PREFIX]
      .into_iter()
      .chain(scoped.copied())
      .collect();

    Self::LEVELS.into_iter().find(|level| {
      targets
        .iter()
        .any(|target| filter.enabled(&Metadata::builder().level(*level).target(target).build()))
    })
  }
}
