use tauri::Runtime;
use tauri::ipc::Invoke;

/// Wrap a generated invoke handler so a command the plugin does not define is reported.
pub fn warn_on_unhandled_command<R, F>(
  plugin: &'static str,
  handler: F,
) -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static
where
  R: Runtime,
  F: Fn(Invoke<R>) -> bool + Send + Sync + 'static,
{
  move |invoke: Invoke<R>| {
    let command: String = invoke.message.command().to_string();

    let handled: bool = handler(invoke);

    if !handled {
      log::warn!("Plugin '{plugin}' was asked for command '{command}', which it does not define");
    }

    handled
  }
}
