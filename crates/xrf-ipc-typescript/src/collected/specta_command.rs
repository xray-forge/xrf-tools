use specta::datatype::Function;
use xrf_utils::to_camel_case;

/// One typed command as Tauri Specta exports it: its wire name, and its arguments named as the wrapper names them.
#[derive(Clone, Debug)]
pub(crate) struct SpectaCommand {
  /// The command's wire name (`open_level`), its name after any `specta(rename)`.
  pub name: String,
  /// Each argument's name, camel-cased as Tauri Specta writes it into the wrapper and its invoke payload.
  pub arguments: Vec<String>,
}

impl From<&Function> for SpectaCommand {
  fn from(function: &Function) -> Self {
    Self {
      name: function.name().to_string(),
      arguments: function.args().iter().map(|(name, _)| to_camel_case(name)).collect(),
    }
  }
}
