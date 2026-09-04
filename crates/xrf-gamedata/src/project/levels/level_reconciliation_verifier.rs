use std::collections::BTreeSet;

use xrf_error::XrfResult;

use crate::GamedataFindingFactory;
use crate::project::levels::level_bundle::LevelBundle;
use crate::project::levels::level_engine_constants::{
  LEVELS_DIRECTORY, MULTIPLAYER_MAPS_FILE, MULTIPLAYER_MAPS_SECTION, SINGLE_PLAYER_MAPS_FILE,
  SINGLE_PLAYER_MAPS_SECTION,
};
use crate::project::levels::level_roster::LevelRoster;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Reconciles the game graph roster, the bundles on disk, and level map declarations.
pub(crate) struct LevelReconciliationVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> LevelReconciliationVerifier<'a> {
  pub(crate) fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  /// Collect level bundle directory names from indexed assets.
  ///
  /// Files stored directly under the levels root, such as `root.ltx`, are not bundles.
  pub(crate) fn bundle_names(&self) -> XrfResult<BTreeSet<String>> {
    // The directories directly inside the levels root are the bundles; a file sitting there, such as `root.ltx`, is not one.
    // `children` answers exactly that distinction, where a prefix enumeration would have to rediscover it.
    Ok(
      self
        .project
        .vfs()
        .scoped(self.project.scope())
        .list_children(LEVELS_DIRECTORY)?
        .directories
        .into_iter()
        .collect(),
    )
  }

  pub(crate) fn verify(&self, roster: &LevelRoster, bundles: &BTreeSet<String>) -> XrfResult<Vec<Finding>> {
    let mut findings: Vec<Finding> = Vec::new();
    let declared_maps: BTreeSet<String> = self.declared_map_levels()?;

    for level in &roster.levels {
      if !bundles.contains(&level.name) {
        findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsMissingBundle,
          LevelBundle::path_of(&level.name),
          format!(
            "Game graph declares level [{}] with id {}, but no level bundle exists for it",
            level.name, level.id
          ),
        ));
      }

      if !declared_maps.contains(&level.name) {
        findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsUndeclaredMap,
          LevelBundle::path_of(&level.name),
          format!(
            "Game graph declares level [{}], but no level map is declared for it in [{SINGLE_PLAYER_MAPS_SECTION}] or [{MULTIPLAYER_MAPS_SECTION}]",
            level.name
          ),
        ));
      }
    }

    let roster_names: BTreeSet<&str> = roster.names();

    for bundle in bundles {
      if !roster_names.contains(bundle.as_str()) {
        findings.push(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::LevelsOrphanBundle,
          LevelBundle::path_of(bundle),
          format!("Level bundle [{bundle}] is not reachable from any game graph"),
        ));
      }
    }

    Ok(findings)
  }

  /// Levels declared in single player and multiplayer map configurations.
  fn declared_map_levels(&self) -> XrfResult<BTreeSet<String>> {
    let mut declared: BTreeSet<String> = BTreeSet::new();

    for (file, section_name) in [
      (SINGLE_PLAYER_MAPS_FILE, SINGLE_PLAYER_MAPS_SECTION),
      (MULTIPLAYER_MAPS_FILE, MULTIPLAYER_MAPS_SECTION),
    ] {
      for location in self.project.entries_with_suffix(file)? {
        let path: &str = location.get_logical_path().as_str();

        // Malformed configurations are reported by the ltx check, not this one. Resolved under the project's dialect
        // even though these sit outside its `configs` prefix, so one sweep does not read some configs patched and
        // others not.
        let Ok(ltx) = self.project.ltx_project.read_full_in_scope(self.project.scope(), path) else {
          xrf_output::verbose!(
            self.options.output,
            "Skipping unreadable level maps configuration: {path}"
          );

          continue;
        };

        if let Some(section) = ltx.section(section_name) {
          declared.extend(section.iter().map(|(key, _)| key.to_lowercase()));
        }
      }
    }

    Ok(declared)
  }
}
