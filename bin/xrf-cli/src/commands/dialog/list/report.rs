use serde::Serialize;
use xrf_dialog::{
  Dialog, DialogElementDescriptor, DialogFinding, DialogOffer, DialogPhrase, DialogProjectFile, DialogTextLanguage,
};

/// What `dialog list` answers: the dialogs a tree holds, or the ones reaching the actor from one NPC profile.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogListReport {
  /// The language captions were resolved in, echoed back.
  pub language: Option<String>,
  /// The profile the list was narrowed to, when it was.
  pub profile: Option<DialogListProfile>,
  pub dialogs: Vec<DialogListEntry>,
  /// What opening the tree had to say about it, and offered dialogs no file declares.
  pub findings: Vec<DialogFinding>,
}

/// The profile a list was narrowed to, with what an NPC of it can be.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogListProfile {
  pub id: String,
  /// The specific characters the profile resolves to: the pinned one, or every one it may pick by class.
  pub characters: Vec<String>,
}

/// One dialog, with what deciding whether it is offered takes.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogListEntry {
  pub id: String,
  /// Logical path of the file declaring it.
  pub logical_path: String,
  /// Selection priority, which orders the topic list.
  pub priority: Option<i32>,
  pub phrases: usize,
  /// Translation key of the entry phrase, whose line the topic list shows for the dialog.
  pub caption_key: Option<String>,
  /// The entry phrase's line, in the report's language.
  pub caption: Option<String>,
  /// Dialog-level elements: the info gates and preconditions deciding whether it is offered, and its init function.
  pub elements: Vec<DialogElementDescriptor>,
  /// Each way the dialog reaches the actor from the profile. Empty for a list not narrowed to one.
  pub offers: Vec<DialogOffer>,
}

impl DialogListEntry {
  pub fn new(
    file: &DialogProjectFile,
    dialog: &Dialog,
    text: Option<DialogTextLanguage<'_>>,
    offers: Vec<DialogOffer>,
  ) -> Self {
    let caption_key: Option<String> = dialog
      .get_entry_phrase()
      .and_then(DialogPhrase::get_text)
      .map(str::to_owned);

    Self {
      id: dialog.get_id().to_owned(),
      logical_path: file.get_logical_path().to_owned(),
      priority: dialog.get_priority(),
      phrases: dialog.get_phrases().len(),
      caption: caption_key
        .as_deref()
        .and_then(|key| text.and_then(|text| text.resolve(key))),
      caption_key,
      elements: dialog.get_elements().iter().map(Into::into).collect(),
      offers,
    }
  }
}
