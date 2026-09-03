//! Whether one name belongs in the archive, answered by the configuration that carries the rules.
//!
//! The selection dialect is xrCompress's, and it reads the same fields a caller edits, so it lives with them
//! rather than with the walk that asks. `archive_pack_source.rs` decides what exists; this decides what is wanted.

use crate::pack::ArchivePackSkipReason;
use crate::pack::config::ArchivePackConfig;
use crate::path::is_component_prefix;

impl ArchivePackConfig {
  /// A recursive exclusion covers the directory and everything below it; a plain one covers only the directory itself.
  ///
  /// Both match on complete path components and without case, the way the engine resolves a name. A raw `starts_with`
  /// would make `configs` swallow `configs_backup`, and a raw comparison would miss the `Configs` the same tree answers
  /// to.
  pub(crate) fn is_excluded_directory(&self, name: &str) -> bool {
    self.is_recursively_excluded_directory(name)
      || self
        .exclude_directories
        .iter()
        .any(|directory| !directory.is_recursive && name.eq_ignore_ascii_case(&directory.path))
  }

  /// The recursive half on its own, which is what a walk prunes on: nothing below such a directory can be
  /// selected, so the tree beneath it never has to be read.
  pub(crate) fn is_recursively_excluded_directory(&self, name: &str) -> bool {
    self
      .exclude_directories
      .iter()
      .any(|directory| directory.is_recursive && is_component_prefix(name, &directory.path))
  }

  /// Which rule leaves a file out of the archive, and none when it packs.
  ///
  /// The hard-coded half is `testSKIP` in `xrCompress.cpp`: editor intermediates, source control leftovers,
  /// and the texture variants the engine rebuilds. It is optional here because a caller packing something
  /// other than a game build has no reason to inherit it.
  pub(crate) fn get_skip_reason(&self, name: &str) -> Option<ArchivePackSkipReason> {
    let lowered: String = name.to_ascii_lowercase();
    let file: &str = lowered.rsplit('\\').next().unwrap_or(&lowered);
    let (stem, extension) = match file.rsplit_once('.') {
      Some((stem, extension)) => (stem, format!(".{extension}")),
      None => (file, String::new()),
    };

    if self
      .exclude_extensions
      .iter()
      .any(|pattern| matches_pattern(&extension, pattern))
    {
      return Some(ArchivePackSkipReason::ExcludedExtension);
    }

    if !self.is_with_skip_list {
      return None;
    }

    Self::is_on_skip_list(&lowered, stem, &extension).then_some(ArchivePackSkipReason::SkipList)
  }

  /// The built-in list itself, over a lowered name already split into its stem and dotted extension.
  fn is_on_skip_list(lowered: &str, stem: &str, extension: &str) -> bool {
    if lowered.contains("textures\\lod\\") || lowered.contains("textures\\det\\") {
      return true;
    }

    // Terrain tiles are rebuilt from their masks, so only the masks are worth carrying.
    if extension != ".thm" && lowered.contains("textures\\terrain\\terrain_") && !stem.ends_with("_mask") {
      return true;
    }

    if lowered.contains("textures\\") && stem.ends_with("_nmap") && !stem.contains("water_flowing_nmap") {
      return true;
    }

    // Level build intermediates, except the lighting the engine still reads.
    if stem == "build" {
      return matches!(extension, ".aimap" | ".cform" | ".details" | ".prj");
    }

    if stem == "do_light" && extension == ".ltx" {
      return true;
    }

    if matches!(
      extension,
      ".txt" | ".tga" | ".db" | ".smf" | ".vcproj" | ".sln" | ".old" | ".rc"
    ) {
      return true;
    }

    // Editor and backup leftovers, named by their second character the way xrCompress tests it.
    matches!(extension.chars().nth(1), Some('~' | '_'))
  }
}

/// Match a `*` and `?` wildcard pattern, case-insensitively, the way `PatternMatch` does.
fn matches_pattern(text: &str, pattern: &str) -> bool {
  let text: Vec<char> = text.to_ascii_lowercase().chars().collect();
  let pattern: Vec<char> = pattern.to_ascii_lowercase().chars().collect();

  let mut text_index: usize = 0;
  let mut pattern_index: usize = 0;
  let mut star: Option<(usize, usize)> = None;

  while text_index < text.len() {
    match pattern.get(pattern_index) {
      Some('*') => {
        star = Some((pattern_index, text_index));
        pattern_index += 1;
      }
      Some('?') => {
        text_index += 1;
        pattern_index += 1;
      }
      Some(symbol) if *symbol == text[text_index] => {
        text_index += 1;
        pattern_index += 1;
      }
      // Backtrack to the last star and let it swallow one more character.
      _ => match star {
        Some((star_index, matched)) => {
          pattern_index = star_index + 1;
          text_index = matched + 1;
          star = Some((star_index, text_index));
        }
        None => return false,
      },
    }
  }

  pattern[pattern_index..].iter().all(|symbol| *symbol == '*')
}

#[cfg(test)]
mod tests {
  use super::matches_pattern;
  use crate::pack::ArchivePackSkipReason;
  use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory};

  fn config() -> ArchivePackConfig {
    ArchivePackConfig::new("gamedata", "db", "configs")
  }

  /// Whether any rule leaves `name` out, for the checks that do not care which one did.
  fn is_skipped(config: &ArchivePackConfig, name: &str) -> bool {
    config.get_skip_reason(name).is_some()
  }

  fn config_excluding(path: &str, is_recursive: bool) -> ArchivePackConfig {
    let mut config: ArchivePackConfig = config();

    config.exclude_directories = vec![ArchivePackDirectory {
      path: String::from(path),
      is_recursive,
    }];

    config
  }

  #[test]
  fn matches_wildcards_case_insensitively() {
    assert!(matches_pattern(".txt", "*.txt"));
    assert!(matches_pattern(".TXT", "*.txt"));
    assert!(matches_pattern(".json", "*.js?n"));
    assert!(matches_pattern(".anything", "*"));
    assert!(!matches_pattern(".ltx", "*.txt"));
    assert!(!matches_pattern("", "*.txt"));
  }

  #[test]
  fn skips_what_the_engine_rebuilds_or_never_reads() {
    for name in [
      "textures\\lod\\lod_wall.dds",
      "textures\\det\\det_grass.dds",
      "textures\\terrain\\terrain_swamp.dds",
      "textures\\wall_nmap.dds",
      "levels\\l01_escape\\build.cform",
      "configs\\do_light.ltx",
      "readme.txt",
      "textures\\wall.tga",
      "database\\nested.db",
      // The engine tests the character after the dot, so a backup marker leads the extension.
      "configs\\system.~ltx",
      "configs\\system._ltx",
    ] {
      assert!(is_skipped(&config(), name), "{name} should be skipped");
    }
  }

  #[test]
  fn keeps_what_the_engine_loads() {
    for name in [
      "configs\\system.ltx",
      "scripts\\xr_logic.script",
      "textures\\wall.dds",
      "textures\\terrain\\terrain_swamp_mask.dds",
      "textures\\terrain\\terrain_swamp.thm",
      "textures\\water\\water_flowing_nmap.dds",
      "levels\\l01_escape\\build.lights",
      "meshes\\actor.ogf",
      "configs\\system.ltx~",
    ] {
      assert!(!is_skipped(&config(), name), "{name} should be kept");
    }
  }

  #[test]
  fn applies_configured_extension_patterns() {
    let mut config: ArchivePackConfig = config();

    config.exclude_extensions = vec![String::from("*.json")];

    // Which rule answered matters here: it is what a run says beside the name it left out.
    assert_eq!(
      config.get_skip_reason("configs\\data.json"),
      Some(ArchivePackSkipReason::ExcludedExtension)
    );
    assert_eq!(config.get_skip_reason("configs\\data.ltx"), None);
    assert_eq!(
      config.get_skip_reason("readme.txt"),
      Some(ArchivePackSkipReason::SkipList),
      "and the built-in list is the other answer"
    );
  }

  #[test]
  fn a_recursive_exclusion_takes_the_directory_and_everything_below_it() {
    let config: ArchivePackConfig = config_excluding("configs", true);

    assert!(config.is_excluded_directory("configs"));
    assert!(config.is_excluded_directory("configs\\system.ltx"));
    assert!(config.is_excluded_directory("configs\\weapons\\w_ak74.ltx"));

    // The defect this rule closes: a byte prefix reaches every sibling spelled like the excluded directory.
    assert!(!config.is_excluded_directory("configs_backup"));
    assert!(!config.is_excluded_directory("configs_backup\\system.ltx"));
    assert!(!config.is_excluded_directory("configs2\\system.ltx"));
  }

  #[test]
  fn a_plain_exclusion_takes_only_the_directory_it_names() {
    let config: ArchivePackConfig = config_excluding("configs", false);

    assert!(config.is_excluded_directory("configs"));
    assert!(!config.is_excluded_directory("configs\\weapons"));
    assert!(!config.is_excluded_directory("configs_backup"));
  }

  #[test]
  fn an_exclusion_matches_the_case_the_engine_folds() {
    for is_recursive in [true, false] {
      let config: ArchivePackConfig = config_excluding("Configs", is_recursive);

      assert!(
        config.is_excluded_directory("configs"),
        "a mixed-case rule names the same directory (recursive: {is_recursive})"
      );
    }

    assert!(config_excluding("Configs", true).is_excluded_directory("CONFIGS\\system.ltx"));
  }

  #[test]
  fn configured_patterns_apply_without_the_built_in_list() {
    let mut config: ArchivePackConfig = config();

    config.is_with_skip_list = false;
    config.exclude_extensions = vec![String::from("*.json")];

    assert!(is_skipped(&config, "configs\\data.json"));
    assert!(!is_skipped(&config, "readme.txt"), "the built-in list is off");
  }
}
