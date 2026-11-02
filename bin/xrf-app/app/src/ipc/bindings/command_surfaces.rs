//! Every Tauri plugin whose commands are mirrored into the frontend.

use xrf_ipc_typescript::IpcCommandSurface;

/// The command surfaces this application exports, every plugin crate's, in the order their modules are written: by
/// plugin name. A plugin the registry's tests find missing here is missing from the frontend.
pub(crate) fn command_surfaces<R: tauri::Runtime>() -> Vec<IpcCommandSurface<R>> {
  macro_rules! collect_surfaces {
    ($($krate:ident),*) => {
      [$($krate::ipc::list_surfaces::<R>(),)*].into_iter().flatten().collect::<Vec<IpcCommandSurface<R>>>()
    };
  }

  let mut surfaces: Vec<IpcCommandSurface<R>> = for_each_plugin_crate!(collect_surfaces);

  surfaces.sort_by_key(|surface| surface.name);

  surfaces
}
