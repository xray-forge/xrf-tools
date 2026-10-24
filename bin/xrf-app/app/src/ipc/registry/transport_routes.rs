use tauri::AppHandle;
use xrf_app_core::transport::TransportRoutes;

/// Every route each plugin crate's domains serve, as the transport serves them.
pub(crate) fn transport_routes() -> TransportRoutes<AppHandle> {
  macro_rules! collect_routes {
    ($($krate:ident),*) => {
      TransportRoutes::new([$($krate::ipc::list_routes(),)*].into_iter().flatten().collect())
    };
  }

  for_each_plugin_crate!(collect_routes)
}
