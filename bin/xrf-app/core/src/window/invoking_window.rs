use tauri::Runtime;
use tauri::ipc::{CommandArg, CommandItem, InvokeError};

/// The window a command was invoked from, by its label: an argument tauri hands the command as it hands its state, never
/// one the page sends, so no page can act in another window's name.
///
/// Read off the invoke rather than taken as a `WebviewWindow`, which a command dispatched for every runtime cannot take:
/// a domain's handler is generic over the runtime, and the window type is bound to one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvokingWindow(String);

impl InvokingWindow {
  /// The window's label.
  pub fn get_label(&self) -> &str {
    &self.0
  }
}

impl<'de, R: Runtime> CommandArg<'de, R> for InvokingWindow {
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    Ok(Self(command.message.webview_ref().window().label().to_owned()))
  }
}

/// Left out of the command's signature in the bindings, as tauri's own injected arguments are.
#[cfg(feature = "typescript-bindings")]
impl specta::function::FunctionArg for InvokingWindow {
  fn to_datatype(_: &mut specta::Types) -> Option<specta::datatype::DataType> {
    None
  }
}
