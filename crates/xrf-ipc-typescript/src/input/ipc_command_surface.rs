use tauri::Runtime;
use tauri_specta::Builder;

/// One raw command as a registry declares it: its wire name, then each argument with its TypeScript type.
pub type RawCommandDeclaration = (&'static str, &'static [(&'static str, &'static str)]);

/// One bulk route as a registry declares it: its name, then each argument with its TypeScript type.
pub type BulkRouteDeclaration = (&'static str, &'static [(&'static str, &'static str)]);

/// One plugin's mirrored surface: the typed Specta builder, and the raw commands and bulk routes it cannot hold.
pub struct IpcCommandSurface<R: Runtime> {
  /// Plugin name, which is both the wire namespace and the generated module's file stem.
  pub name: &'static str,
  pub builder: Builder<R>,
  /// Raw commands of the plugin, empty for one declaring none.
  pub raw: &'static [RawCommandDeclaration],
  /// Bulk routes of the plugin, empty for one declaring none.
  pub bulk: &'static [BulkRouteDeclaration],
}

impl<R: Runtime> IpcCommandSurface<R> {
  /// The surface of one plugin.
  pub fn new(
    name: &'static str,
    builder: Builder<R>,
    raw: &'static [RawCommandDeclaration],
    bulk: &'static [BulkRouteDeclaration],
  ) -> Self {
    Self {
      name,
      builder,
      raw,
      bulk,
    }
  }
}
