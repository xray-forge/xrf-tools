use serde::{Deserialize, Serialize};

/// How a dialog reaches the actor when it talks to an NPC.
///
/// Whether it is then offered is the dialog's own conditions, which a running game judges.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum DialogOffer {
  /// The character's start dialog: the character opens the talk with it and says phrase `0`.
  Start { character: String },
  /// One of the character's actor dialogs, which the actor picks from the topic list and opens.
  Actor { character: String },
  /// Offered by every NPC once the actor knows the info portion.
  Info { info: String },
}
