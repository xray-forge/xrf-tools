use serde::Serialize;
use xrf_dialog::DialogFinding;

use crate::commands::dialog::list::dialog_list_entry::DialogListEntry;
use crate::commands::dialog::list::dialog_list_profile::DialogListProfile;

/// What `dialog list` answers: the dialogs a tree holds, or the ones reaching the actor from one NPC profile.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogListReport {
  /// The language captions were resolved in, echoed back.
  language: Option<String>,
  /// The profile the list was narrowed to, when it was.
  profile: Option<DialogListProfile>,
  dialogs: Vec<DialogListEntry>,
  /// What opening the tree had to say about it, and offered dialogs no file declares.
  findings: Vec<DialogFinding>,
}

impl DialogListReport {
  pub fn new(
    language: Option<String>,
    profile: Option<DialogListProfile>,
    dialogs: Vec<DialogListEntry>,
    findings: Vec<DialogFinding>,
  ) -> Self {
    Self {
      language,
      profile,
      dialogs,
      findings,
    }
  }

  pub fn profile(&self) -> Option<&DialogListProfile> {
    self.profile.as_ref()
  }

  pub fn dialogs(&self) -> &[DialogListEntry] {
    &self.dialogs
  }

  pub fn findings(&self) -> &[DialogFinding] {
    &self.findings
  }
}
