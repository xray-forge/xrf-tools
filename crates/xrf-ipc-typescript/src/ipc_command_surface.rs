use tauri::Runtime;
use tauri_specta::Builder;

/// One raw command as a registry declares it: its wire name, then each argument with its TypeScript type.
///
/// Spelled by hand because Specta cannot collect a command returning `tauri::ipc::Response`; the return is always
/// `ArrayBuffer`, which is what a raw response arrives as.
pub type RawCommandDeclaration = (&'static str, &'static [(&'static str, &'static str)]);

/// One plugin's mirrored surface: the typed Specta builder, and the raw commands that builder cannot hold.
pub struct IpcCommandSurface<R: Runtime> {
  /// Plugin name, which is both the wire namespace and the generated module's file stem.
  pub name: &'static str,
  pub builder: Builder<R>,
  /// Raw commands of the plugin, empty for one declaring none.
  pub raw: &'static [RawCommandDeclaration],
}

impl<R: Runtime> IpcCommandSurface<R> {
  /// The surface of one plugin.
  pub fn new(name: &'static str, builder: Builder<R>, raw: &'static [RawCommandDeclaration]) -> Self {
    Self { name, builder, raw }
  }
}
