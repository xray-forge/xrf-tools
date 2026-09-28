use std::collections::HashMap;

use crate::core::transport::TransportRoute;

/// Every route the transport serves, by path.
pub(crate) struct TransportRoutes<C> {
  routes: HashMap<String, TransportRoute<C>>,
}

impl<C: Send + 'static> TransportRoutes<C> {
  /// # Panics
  ///
  /// Panics on two routes at one path, which is a registry that cannot be served.
  pub(crate) fn new(routes: Vec<TransportRoute<C>>) -> Self {
    let mut table: HashMap<String, TransportRoute<C>> = HashMap::with_capacity(routes.len());

    for route in routes {
      let path: String = route.get_path().to_string();

      assert!(
        table.insert(path.clone(), route).is_none(),
        "Transport route '{path}' is declared twice"
      );
    }

    Self { routes: table }
  }

  /// The route at a request's path, with or without its leading slash.
  pub(crate) fn get(&self, path: &str) -> Option<&TransportRoute<C>> {
    self.routes.get(path.strip_prefix('/').unwrap_or(path))
  }

  pub(crate) fn len(&self) -> usize {
    self.routes.len()
  }
}
