use serde::Serialize;
use xrf_engine_target::XrayEngine;
use xrf_environment::{EnvironmentCatalog, EnvironmentFinding};

use crate::plugins::environment::description::environment_cycle_entry::EnvironmentCycleEntry;
use crate::plugins::environment::description::environment_definition_entry::EnvironmentDefinitionEntry;

/// What a catalog holds, as a browser lists it, and every finding in it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentCatalogDescription {
  pub engine: XrayEngine,
  /// Every config read.
  pub configs: Vec<String>,
  pub cycles: Vec<EnvironmentCycleEntry>,
  pub effects: Vec<EnvironmentCycleEntry>,
  pub suns: Vec<EnvironmentDefinitionEntry>,
  pub thunderbolt_collections: Vec<EnvironmentDefinitionEntry>,
  pub ambients: Vec<EnvironmentDefinitionEntry>,
  pub findings: Vec<EnvironmentFinding>,
}

impl From<&EnvironmentCatalog> for EnvironmentCatalogDescription {
  fn from(catalog: &EnvironmentCatalog) -> Self {
    let entry = |name: &str, file: &str| EnvironmentDefinitionEntry::of(name, file, catalog);

    Self {
      ambients: catalog.ambients.iter().map(|it| entry(&it.name, &it.file)).collect(),
      configs: catalog.configs.clone(),
      cycles: catalog
        .cycles
        .iter()
        .map(|it| EnvironmentCycleEntry::of(it, catalog))
        .collect(),
      effects: catalog
        .effects
        .iter()
        .map(|it| EnvironmentCycleEntry::of(it, catalog))
        .collect(),
      engine: catalog.engine,
      findings: catalog.findings.clone(),
      suns: catalog.suns.iter().map(|it| entry(&it.name, &it.file)).collect(),
      thunderbolt_collections: catalog
        .thunderbolt_collections
        .iter()
        .map(|it| entry(&it.name, &it.file))
        .collect(),
    }
  }
}
