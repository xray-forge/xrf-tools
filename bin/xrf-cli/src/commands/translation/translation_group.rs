use crate::commands::translation;
use crate::core::command::{CommandGroup, GenericCommand};

/// The `translation` domain and the operations it exposes.
pub(crate) struct TranslationGroup;

impl TranslationGroup {
  /// The group, as dispatch and `docs generate` register it.
  pub(crate) fn declare() -> CommandGroup {
    CommandGroup {
      slug: "translation",
      label: "Translation",
      about: "Translation file tools",
      commands: vec![
        translation::build::BuildCommand::new_box(),
        translation::format::FormatCommand::new_box(),
        translation::initialize::InitializeCommand::new_box(),
        translation::parse::ParseCommand::new_box(),
        translation::verify::VerifyCommand::new_box(),
      ],
    }
  }
}
