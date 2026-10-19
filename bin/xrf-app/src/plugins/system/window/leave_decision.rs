use serde::{Deserialize, Serialize};

/// What the page decided when asked to leave what is open or to close the window, which only the page knows.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LeaveDecision {
  /// A leave refused while an operation runs.
  RefusedWhileBusy,
  /// A leave or exit ignored while another waits for an answer.
  IgnoredWhilePending,
  /// Asked before leaving unsaved edits.
  AskedAboutUnsaved,
  /// Left, with nothing unsaved or once the person agreed.
  Left,
  /// The window asked to close.
  ExitAsked,
  /// Asked before closing while an operation runs.
  AskedAboutRunningWork,
  /// Closed anyway while an operation ran, as the person chose.
  ExitedWhileBusy,
  /// Kept open, unsaved edits and all, as the person chose.
  KeptOpen,
  /// Kept the window open while an operation runs, as the person chose.
  KeptOpenWhileBusy,
  /// A stay refused while edits were being saved before leaving.
  StayRefusedWhileSaving,
  /// A discard refused while an operation runs.
  DiscardRefusedWhileBusy,
  /// Unsaved edits discarded before leaving.
  Discarded,
  /// A save before leaving refused, with nothing to save, no leave waiting, or an operation running.
  SaveRefused,
  /// Saving edits before leaving.
  Saving,
  /// Saved, then left.
  Saved,
  /// The save wrote nothing, so the page stayed.
  SaveNotWritten,
  /// The save failed, so the page stayed.
  SaveFailed,
}

impl LeaveDecision {
  /// How the log says it.
  pub fn describe(self) -> &'static str {
    match self {
      Self::RefusedWhileBusy => "refused leaving while an operation runs",
      Self::IgnoredWhilePending => "ignored a request while another waits for an answer",
      Self::AskedAboutUnsaved => "asked before leaving unsaved edits",
      Self::Left => "left what was open",
      Self::ExitAsked => "the window asked to close",
      Self::AskedAboutRunningWork => "asked before closing while an operation runs",
      Self::ExitedWhileBusy => "closing anyway while an operation runs",
      Self::KeptOpen => "kept open with unsaved edits",
      Self::KeptOpenWhileBusy => "kept the window open while an operation runs",
      Self::StayRefusedWhileSaving => "refused staying while saving before leaving",
      Self::DiscardRefusedWhileBusy => "refused discarding while an operation runs",
      Self::Discarded => "discarding unsaved edits",
      Self::SaveRefused => "refused saving before leaving",
      Self::Saving => "saving before leaving",
      Self::Saved => "saved, then leaving",
      Self::SaveNotWritten => "save wrote nothing, staying",
      Self::SaveFailed => "save failed, staying",
    }
  }
}
