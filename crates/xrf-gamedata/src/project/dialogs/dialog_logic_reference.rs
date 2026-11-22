use xrf_ltx::{Condlist, Ltx};

/// A dialog a logic section names for its NPC, in a field its scheme reads as a dialog id.
///
/// `xr_meet` picks `meet_dialog` from a condlist and hands it to `set_start_dialog`, and `xr_wounded` hands
/// `help_start_dialog` to it as written; both assert in the engine on an id no dialog file declares. `help_dialog` only
/// marks which dialog heals the NPC, so a wrong one leaves the NPC unhealable rather than stopping the game.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct DialogLogicReference {
  pub(crate) section: String,
  pub(crate) field: &'static str,
  pub(crate) dialog: String,
  /// Whether the engine opens the dialog as the NPC's start dialog, which it asserts on when nothing declares it.
  pub(crate) is_start_dialog: bool,
}

impl DialogLogicReference {
  const MEET_SCHEME: &'static str = "meet";
  const WOUNDED_SCHEME: &'static str = "wounded";
  const MEET_DIALOG_FIELD: &'static str = "meet_dialog";
  const HELP_START_DIALOG_FIELD: &'static str = "help_start_dialog";
  const HELP_DIALOG_FIELD: &'static str = "help_dialog";
  /// The value the schemes read as no dialog, restoring the character's own start dialog.
  const NONE: &'static str = "nil";

  /// Every dialog the `meet` and `wounded` sections of a config or an object's custom data name, in section order.
  ///
  /// A condlist that does not parse names nothing here; the LTX check reports it.
  pub(crate) fn list_in(ltx: &Ltx) -> Vec<Self> {
    let mut references: Vec<Self> = Vec::new();

    for (name, section) in ltx {
      match Self::read_scheme(name) {
        Self::MEET_SCHEME => {
          let Some(Ok(condlist)) = section.get(Self::MEET_DIALOG_FIELD).map(Condlist::parse) else {
            continue;
          };

          for branch in condlist.branches {
            if let Some(dialog) = branch.result {
              references.extend(Self::read_value(name, Self::MEET_DIALOG_FIELD, &dialog));
            }
          }
        }
        Self::WOUNDED_SCHEME => {
          for field in [Self::HELP_START_DIALOG_FIELD, Self::HELP_DIALOG_FIELD] {
            if let Some(dialog) = section.get(field) {
              references.extend(Self::read_value(name, field, dialog));
            }
          }
        }
        _ => {}
      }
    }

    references
  }

  /// The reference a field's value makes, or `None` for a value naming no dialog.
  fn read_value(section: &str, field: &'static str, value: &str) -> Option<Self> {
    let dialog: &str = value.trim();

    (!dialog.is_empty() && dialog != Self::NONE).then(|| Self {
      section: section.to_owned(),
      field,
      dialog: dialog.to_owned(),
      is_start_dialog: field != Self::HELP_DIALOG_FIELD,
    })
  }

  /// The scheme a logic section belongs to, which is its name up to the first `@`.
  fn read_scheme(section: &str) -> &str {
    section.split_once('@').map_or(section, |(scheme, _)| scheme)
  }
}
