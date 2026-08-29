use std::collections::HashSet;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxProject};
use xrf_utils::format_path;
use xrf_vfs::XrayAssetType;
use xrf_vfs::XrayLogicalPath;
use xrf_xml::{XmlDocument, XmlParseOptions};

use crate::GamedataFindingFactory;
use crate::project::gamedata_project::CONFIGS_DIRECTORY;
use crate::project::sounds::sound_references_verification_result::GamedataSoundReferencesVerificationResult;
use crate::{GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

pub(crate) struct SoundReferencesVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
  sound_paths: &'a [String],
}

impl<'a> SoundReferencesVerifier<'a> {
  pub(crate) fn new(
    project: &'a GamedataProject,
    options: &'a GamedataProjectVerifyOptions,
    sound_paths: &'a [String],
  ) -> Self {
    Self {
      options,
      project,
      sound_paths,
    }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataSoundReferencesVerificationResult> {
    let sound_names: HashSet<String> = Self::read_sound_names(self.sound_paths);
    let sound_roots: HashSet<String> = Self::read_sound_roots(&sound_names);
    let mut result: GamedataSoundReferencesVerificationResult = Default::default();

    self.verify_references_in_configs(&sound_names, &sound_roots, &mut result);
    self.verify_references_in_xml(&sound_names, &sound_roots, &mut result);

    Ok(result)
  }

  fn read_sound_names(sound_paths: &[String]) -> HashSet<String> {
    sound_paths
      .iter()
      .filter_map(|path| path.strip_prefix("sounds\\").map(XrayAssetType::sound_reference_name))
      .collect()
  }

  fn read_sound_roots(sound_names: &HashSet<String>) -> HashSet<String> {
    sound_names
      .iter()
      .filter_map(|name| name.split_once('\\').map(|(root, _)| root.to_string()))
      .collect()
  }

  fn verify_references_in_configs(
    &self,
    sound_names: &HashSet<String>,
    sound_roots: &HashSet<String>,
    result: &mut GamedataSoundReferencesVerificationResult,
  ) {
    for path in &self.project.ltx_project.ltx_file_entries {
      if LtxProject::is_ltx_scheme_path(path) {
        continue;
      }

      // Read through the project: its entries are logical paths, which a filesystem read cannot resolve.
      let reported: PathBuf = self.project.ltx_project.path_of(path);

      match self.project.ltx_project.read_full(path) {
        Ok(ltx) => self.verify_references_in_ltx(sound_names, sound_roots, &ltx, &reported, result),
        Err(error) => xrf_output::verbose!(
          self.options.output,
          "Skipping ltx entry in sound reference check: {} - {}",
          format_path(&reported),
          error
        ),
      }
    }
  }

  fn verify_references_in_xml(
    &self,
    sound_names: &HashSet<String>,
    sound_roots: &HashSet<String>,
    result: &mut GamedataSoundReferencesVerificationResult,
  ) {
    // Enumerated and read through the VFS, so an installation's archived UI XML is inspected too. Reported by the path a
    // person can act on, which for an archived entry is its logical path.
    for location in self.project.entries() {
      let logical_path: &XrayLogicalPath = location.get_logical_path();

      if !logical_path.is_under(CONFIGS_DIRECTORY).unwrap_or(false) || !logical_path.has_extension(".xml") {
        continue;
      }

      let path: PathBuf = location
        .to_physical_path()
        .unwrap_or_else(|| PathBuf::from(logical_path.as_str()));

      let contents: Vec<u8> = match self.project.read_bytes(logical_path.as_str()) {
        Ok(contents) => contents,
        Err(error) => {
          result.checked_references_count += 1;
          result.invalid_references_count += 1;
          result.findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::SoundsReferences,
            &path,
            format!("Could not inspect XML sound references: {error}"),
          ));
          continue;
        }
      };

      let document: XmlDocument = match XmlDocument::parse_bytes(&contents, XmlParseOptions { allow_dtd: true }) {
        Ok(document) => document,
        Err(error) => {
          result.checked_references_count += 1;
          result.invalid_references_count += 1;
          result.findings.push(GamedataFindingFactory::for_asset(
            GamedataVerificationRule::SoundsReferences,
            &path,
            format!("Could not inspect XML sound references: {error}"),
          ));
          continue;
        }
      };

      for sound in document.elements_named("sound") {
        let reference: &str = sound.text();
        if Self::is_direct_sound_reference(sound_roots, reference) {
          self.verify_reference(sound_names, reference, &path, "<sound>", result);
        }
      }
    }
  }

  fn verify_references_in_ltx(
    &self,
    sound_names: &HashSet<String>,
    sound_roots: &HashSet<String>,
    ltx: &Ltx,
    path: &Path,
    result: &mut GamedataSoundReferencesVerificationResult,
  ) {
    for (section_name, section) in &ltx.sections {
      for (key, value) in section.iter() {
        if !Self::is_sound_reference_key(key) {
          continue;
        }

        for reference in value.split(',') {
          let reference: &str = reference.trim();

          if !Self::is_direct_sound_reference(sound_roots, reference) {
            continue;
          }

          self.verify_reference(sound_names, reference, path, &format!("[{section_name}] {key}"), result);
        }
      }
    }

    xrf_output::verbose!(
      self.options.output,
      "Verified sound references in {}",
      format_path(path)
    );
  }

  fn verify_reference(
    &self,
    sound_names: &HashSet<String>,
    reference: &str,
    path: &Path,
    location: &str,
    result: &mut GamedataSoundReferencesVerificationResult,
  ) {
    let sound_name: String = XrayAssetType::sound_reference_name(reference);
    result.checked_references_count += 1;

    if Self::sound_reference_exists(sound_names, &sound_name) {
      return;
    }

    result.invalid_references_count += 1;
    result.findings.push(GamedataFindingFactory::for_asset(
      GamedataVerificationRule::SoundsReferences,
      path,
      format!("Unknown sound reference: {location} = {reference}"),
    ));
  }

  fn is_sound_reference_key(key: &str) -> bool {
    key.to_ascii_lowercase().starts_with("snd_")
  }

  fn is_direct_sound_reference(sound_roots: &HashSet<String>, reference: &str) -> bool {
    XrayAssetType::sound_reference_name(reference)
      .split_once('\\')
      .is_some_and(|(root, _)| sound_roots.contains(root))
  }

  fn sound_reference_exists(sound_names: &HashSet<String>, reference: &str) -> bool {
    sound_names.contains(reference)
      || (reference.ends_with('_') && sound_names.iter().any(|name| name.starts_with(reference)))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashSet;

  use super::SoundReferencesVerifier;
  use xrf_vfs::XrayAssetType;

  #[test]
  fn resolves_exact_and_randomized_sound_references() {
    let names: HashSet<String> = [
      String::from("weapons\\ak74_shot"),
      String::from("monsters\\boar\\boar_idle_1"),
    ]
    .into_iter()
    .collect();

    assert!(SoundReferencesVerifier::sound_reference_exists(
      &names,
      &XrayAssetType::sound_reference_name("sounds/weapons/ak74_shot.ogg")
    ));
    assert!(SoundReferencesVerifier::sound_reference_exists(
      &names,
      &XrayAssetType::sound_reference_name("monsters\\boar\\boar_idle_")
    ));
    assert!(!SoundReferencesVerifier::sound_reference_exists(
      &names,
      &XrayAssetType::sound_reference_name("weapons\\missing")
    ));
  }

  #[test]
  fn only_treats_sound_fields_with_paths_as_direct_references() {
    let sound_roots: HashSet<String> = [String::from("weapons")].into_iter().collect();

    assert!(SoundReferencesVerifier::is_sound_reference_key("snd_shoot"));
    assert!(!SoundReferencesVerifier::is_sound_reference_key("Sound_Vampire_Hit"));
    assert!(SoundReferencesVerifier::is_direct_sound_reference(
      &sound_roots,
      "weapons\\ak74_shot"
    ));
    assert!(!SoundReferencesVerifier::is_direct_sound_reference(
      &sound_roots,
      "fight\\enemy\\enemy_"
    ));
  }
}
