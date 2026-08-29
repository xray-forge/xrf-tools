use std::collections::{HashMap, HashSet};

use xrf_ltx::{Ltx, LtxProject};
use xrf_utils::format_path;
use xrf_vfs::XrayLogicalPath;

/// Definitions that weather sections may reference.
///
/// Read failures are retained for each definition family. This lets the weather validator mark
/// affected cycles invalid instead of treating an unavailable definition file as an empty set.
pub struct WeatherDefinitions {
  /// Ambient section names read from `environment/ambients.ltx` and its includes.
  pub ambient_sections: Result<HashSet<String>, String>,
  /// Sun section names read from `environment/suns.ltx`.
  pub sun_sections: Result<HashSet<String>, String>,
  /// Thunderbolt collection names mapped to member definitions that could not be resolved.
  ///
  /// An empty member list means that every member of the collection has a definition.
  pub thunderbolt_collections: Result<HashMap<String, Vec<String>>, String>,
  /// Parsed legacy definitions from `system.ltx`.
  legacy_system: Result<Ltx, String>,
}

impl WeatherDefinitions {
  /// Reads weather definitions through the project that holds them.
  ///
  /// Read through the project rather than the host filesystem, so an installation's definitions are read from its `db\`
  /// volumes instead of reported unreadable.
  ///
  /// The returned value retains definition-family errors so validation can continue and report
  /// every affected weather cycle in one run.
  pub fn read(project: &LtxProject) -> Self {
    Self {
      ambient_sections: Self::read_sections(project, "environment\\ambients.ltx"),
      sun_sections: Self::read_sections(project, "environment\\suns.ltx"),
      thunderbolt_collections: Self::read_thunderbolt_collections(project),
      legacy_system: Self::read_ltx(project, "system.ltx"),
    }
  }

  /// Resolves a sun name through the primary catalog, then the legacy `system.ltx` fallback.
  pub fn has_sun(&self, sun: &str) -> Result<bool, String> {
    let sun_sections: &HashSet<String> = self.sun_sections.as_ref().map_err(Clone::clone)?;

    if sun_sections.contains(sun) {
      return Ok(true);
    }

    self
      .legacy_system
      .as_ref()
      .map(|system| system.has_section(sun))
      .map_err(Clone::clone)
  }

  /// Resolves a thunderbolt collection and returns its unresolved member definitions.
  ///
  /// Primary collections use `environment/thunderbolts.ltx`. The legacy `system.ltx` fallback is
  /// consulted only when the collection name is absent from the primary catalog.
  pub fn missing_thunderbolt_definitions(&self, collection_name: &str) -> Result<Option<Vec<String>>, String> {
    let collections: &HashMap<String, Vec<String>> = self.thunderbolt_collections.as_ref().map_err(Clone::clone)?;

    if let Some(missing_definitions) = collections.get(collection_name) {
      return Ok(Some(missing_definitions.clone()));
    }

    let legacy_system: &Ltx = self.legacy_system.as_ref().map_err(Clone::clone)?;
    let Some(collection) = legacy_system.section(collection_name) else {
      return Ok(None);
    };
    let missing_definitions: Vec<String> = collection
      .iter()
      .map(|(thunderbolt_name, _)| thunderbolt_name.to_string())
      .filter(|thunderbolt_name| !legacy_system.has_section(thunderbolt_name))
      .collect();

    Ok(Some(missing_definitions))
  }

  fn read_sections(project: &LtxProject, relative_path: &str) -> Result<HashSet<String>, String> {
    Self::read_ltx(project, relative_path).map(|ltx| {
      ltx
        .iter()
        .map(|(section_name, _)| section_name.to_string())
        .filter(|section_name| !section_name.is_empty())
        .collect()
    })
  }

  /// Reads one definition config named relative to the project's configs.
  ///
  /// Failures name the path a person can act on, which for a loose config is its file and for an archived one its engine
  /// identity.
  fn read_ltx(project: &LtxProject, relative_path: &str) -> Result<Ltx, String> {
    let logical_path: XrayLogicalPath = project
      .config_path(relative_path)
      .map_err(|error| format!("Could not address weather definitions at {relative_path}: {error}"))?;

    project.read_full(&logical_path).map_err(|error| {
      format!(
        "Could not read weather definitions from {}: {error}",
        format_path(&project.path_of(&logical_path))
      )
    })
  }

  fn read_thunderbolt_collections(project: &LtxProject) -> Result<HashMap<String, Vec<String>>, String> {
    let collections: Ltx = Self::read_ltx(project, "environment\\thunderbolt_collections.ltx")?;
    let thunderbolts: Ltx = Self::read_ltx(project, "environment\\thunderbolts.ltx")?;

    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    for (collection_name, collection) in &collections {
      if !collection_name.is_empty() {
        result.insert(
          collection_name.to_string(),
          collection
            .iter()
            .map(|(thunderbolt_name, _)| thunderbolt_name.to_string())
            .filter(|thunderbolt_name| !thunderbolts.has_section(thunderbolt_name))
            .collect(),
        );
      }
    }

    Ok(result)
  }
}
