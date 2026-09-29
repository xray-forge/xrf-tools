use std::sync::{Arc, Mutex};

use xrf_environment::EnvironmentCatalog;

use crate::plugins::environment::request::EnvironmentRequest;

/// The catalogs read lately, each for the roots, dialect and engine it was read with.
/// Held so a browser moving between cycles reads nothing twice; a few, not every one, since a catalog is megabytes.
pub struct EnvironmentState {
  catalogs: Mutex<Vec<(EnvironmentRequest, Arc<EnvironmentCatalog>)>>,
}

impl EnvironmentState {
  /// How many catalogs are held at once, newest last.
  const HELD: usize = 4;

  pub fn new() -> Self {
    Self {
      catalogs: Mutex::new(Vec::new()),
    }
  }

  /// The catalog held for a request, if one is.
  pub fn get(&self, request: &EnvironmentRequest) -> Option<Arc<EnvironmentCatalog>> {
    self
      .catalogs
      .lock()
      .expect("environment catalogs to not be poisoned")
      .iter()
      .find(|(held, _)| held == request)
      .map(|(_, catalog)| Arc::clone(catalog))
  }

  /// Holds a catalog for a request, letting the oldest go past the limit.
  pub fn put(&self, request: EnvironmentRequest, catalog: Arc<EnvironmentCatalog>) {
    let mut catalogs = self.catalogs.lock().expect("environment catalogs to not be poisoned");

    catalogs.retain(|(held, _)| *held != request);
    catalogs.push((request, catalog));

    let excess: usize = catalogs.len().saturating_sub(Self::HELD);

    catalogs.drain(..excess);
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::sync::Arc;

  use xrf_engine_target::XrayEngine;
  use xrf_environment::{EnvironmentCatalog, WeatherGraphs};
  use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

  use super::EnvironmentState;
  use crate::plugins::environment::request::EnvironmentRequest;

  fn catalog(engine: XrayEngine) -> Arc<EnvironmentCatalog> {
    Arc::new(EnvironmentCatalog {
      ambient_effects: Vec::new(),
      ambients: Vec::new(),
      configs: Vec::new(),
      cycles: Vec::new(),
      effects: Vec::new(),
      engine,
      findings: Vec::new(),
      graphs: WeatherGraphs::default(),
      level_ambients: Vec::new(),
      sound_channels: Vec::new(),
      sun_table: None,
      suns: Vec::new(),
      thunderbolt_collections: Vec::new(),
      thunderbolt_settings: None,
      thunderbolts: Vec::new(),
    })
  }

  fn request(root: &str, engine: XrayEngine) -> EnvironmentRequest {
    EnvironmentRequest {
      engine,
      is_dltx: false,
      roots: XrayRoots::new([XrayRoot::new(PathBuf::from(root), XrayMountMode::Auto)]),
    }
  }

  #[test]
  fn holds_a_catalog_for_the_roots_dialect_and_engine_it_was_read_with() {
    let state: EnvironmentState = EnvironmentState::new();

    state.put(request("a", XrayEngine::Vanilla), catalog(XrayEngine::Vanilla));

    assert!(state.get(&request("a", XrayEngine::Vanilla)).is_some());
    assert!(state.get(&request("a", XrayEngine::Extended)).is_none());
    assert!(state.get(&request("b", XrayEngine::Vanilla)).is_none());
  }

  #[test]
  fn lets_the_oldest_catalog_go_past_its_limit() {
    let state: EnvironmentState = EnvironmentState::new();

    for root in ["a", "b", "c", "d"] {
      state.put(request(root, XrayEngine::Vanilla), catalog(XrayEngine::Vanilla));
    }

    // Read again, so it is the newest and another is the oldest.
    state.put(request("a", XrayEngine::Vanilla), catalog(XrayEngine::Vanilla));
    state.put(request("e", XrayEngine::Vanilla), catalog(XrayEngine::Vanilla));

    assert!(state.get(&request("a", XrayEngine::Vanilla)).is_some());
    assert!(state.get(&request("b", XrayEngine::Vanilla)).is_none());
    assert!(state.get(&request("e", XrayEngine::Vanilla)).is_some());
  }
}
