use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use xrf_error::XrfResult;

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory};

/// One file selected for packing.
#[derive(Clone, Debug)]
pub(crate) struct ArchivePackEntry {
  /// Name as the engine will see it: relative to the source root, with X-Ray separators.
  pub(crate) name: String,
  pub(crate) path: PathBuf,
  pub(crate) size: u64,
}

/// Everything one packing run will write.
#[derive(Debug, Default)]
pub(crate) struct ArchivePackSource {
  pub(crate) entries: Vec<ArchivePackEntry>,
  /// Directory names, which the writer encodes as zero-size rows with a trailing separator so the engine can list them.
  pub(crate) directories: Vec<String>,
  /// Files the rules rejected, reported so a surprising omission is visible rather than silent.
  pub(crate) skipped: usize,
}

impl ArchivePackSource {
  /// Walk the configured roots and decide what goes into the archive.
  ///
  /// Entries come out sorted by name. xrCompress emitted them in filesystem enumeration order; the
  /// engine indexes by name and does not care, and a stable order makes output reproducible.
  pub(crate) fn collect(config: &ArchivePackConfig) -> XrfResult<Self> {
    let mut source: Self = Self::default();

    // A config that names nothing packs the whole tree, which is what xrCompress does when handed a
    // directory and no LTX. Naming nothing is far more likely to mean "everything" than "an empty archive".
    if config.include_directories.is_empty() && config.include_files.is_empty() {
      source.collect_directory(
        config,
        &ArchivePackDirectory {
          path: String::new(),
          is_recursive: true,
        },
      )?;
    }

    for directory in &config.include_directories {
      source.collect_directory(config, directory)?;
    }

    for name in &config.include_files {
      let name: String = normalize_name(name);
      let path: PathBuf = config.source.join(name.replace('\\', "/"));

      // Listed files bypass the directory rules, exactly as `[include_files]` does in xrCompress, but a
      // name that does not resolve is a configuration error rather than something to pass over.
      let size: u64 = path.metadata()?.len();

      source.entries.push(ArchivePackEntry { name, path, size });
    }

    source.entries.sort_by(|left, right| left.name.cmp(&right.name));
    source.entries.dedup_by(|left, right| left.name == right.name);
    source.directories.sort();
    source.directories.dedup();

    Ok(source)
  }

  fn collect_directory(&mut self, config: &ArchivePackConfig, directory: &ArchivePackDirectory) -> XrfResult<()> {
    let root: PathBuf = if directory.path.is_empty() {
      config.source.clone()
    } else {
      config.source.join(directory.path.replace('\\', "/"))
    };

    if !root.exists() {
      return Ok(());
    }

    // A non-recursive include still names its immediate subdirectories, matching `FS_ListFolders` without
    // `FS_RootOnly`, so the archive lists them even when their contents stay out.
    let walk: WalkDir = if directory.is_recursive {
      WalkDir::new(&root)
    } else {
      WalkDir::new(&root).max_depth(1)
    };

    for entry in walk.into_iter().filter_map(Result::ok) {
      let path: &Path = entry.path();
      let Some(name) = relative_name(&config.source, path) else {
        continue;
      };

      if name.is_empty() || is_excluded_directory(config, &name) {
        continue;
      }

      if entry.file_type().is_dir() {
        self.directories.push(name);
      } else if entry.file_type().is_file() {
        if is_skipped_file(config, &name) {
          self.skipped += 1;

          continue;
        }

        self.entries.push(ArchivePackEntry {
          name,
          path: path.into(),
          size: path.metadata()?.len(),
        });
      }
    }

    Ok(())
  }
}

/// Express a host path as the engine-facing name relative to the source root.
fn relative_name(source: &Path, path: &Path) -> Option<String> {
  Some(normalize_name(path.strip_prefix(source).ok()?.to_str()?))
}

fn normalize_name(name: &str) -> String {
  name.replace('/', "\\").trim_matches('\\').to_string()
}

/// An excluded directory matches by prefix when recursive, and by exact name otherwise.
fn is_excluded_directory(config: &ArchivePackConfig, name: &str) -> bool {
  config.exclude_directories.iter().any(|directory| {
    if directory.is_recursive {
      name.starts_with(&directory.path)
    } else {
      name == directory.path
    }
  })
}

/// Decide whether a file is left out of the archive.
///
/// The hard-coded half is `testSKIP` in `xrCompress.cpp`: editor intermediates, source control leftovers,
/// and the texture variants the engine rebuilds. It is optional here because a caller packing something
/// other than a game build has no reason to inherit it.
fn is_skipped_file(config: &ArchivePackConfig, name: &str) -> bool {
  let lowered: String = name.to_ascii_lowercase();
  let file: &str = lowered.rsplit('\\').next().unwrap_or(&lowered);
  let (stem, extension) = match file.rsplit_once('.') {
    Some((stem, extension)) => (stem, format!(".{extension}")),
    None => (file, String::new()),
  };

  if config
    .exclude_extensions
    .iter()
    .any(|pattern| matches_pattern(&extension, pattern))
  {
    return true;
  }

  if !config.is_with_skip_list {
    return false;
  }

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
    return matches!(extension.as_str(), ".aimap" | ".cform" | ".details" | ".prj");
  }

  if stem == "do_light" && extension == ".ltx" {
    return true;
  }

  if matches!(
    extension.as_str(),
    ".txt" | ".tga" | ".db" | ".smf" | ".vcproj" | ".sln" | ".old" | ".rc"
  ) {
    return true;
  }

  // Editor and backup leftovers, named by their second character the way xrCompress tests it.
  matches!(extension.chars().nth(1), Some('~' | '_'))
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
  use super::{is_skipped_file, matches_pattern};
  use crate::pack::archive_pack_config::ArchivePackConfig;

  fn config() -> ArchivePackConfig {
    ArchivePackConfig::new("gamedata", "db", "configs")
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
      assert!(is_skipped_file(&config(), name), "{name} should be skipped");
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
      assert!(!is_skipped_file(&config(), name), "{name} should be kept");
    }
  }

  #[test]
  fn applies_configured_extension_patterns() {
    let mut config: ArchivePackConfig = config();

    config.exclude_extensions = vec![String::from("*.json")];

    assert!(is_skipped_file(&config, "configs\\data.json"));
    assert!(!is_skipped_file(&config, "configs\\data.ltx"));
  }

  #[test]
  fn configured_patterns_apply_without_the_built_in_list() {
    let mut config: ArchivePackConfig = config();

    config.is_with_skip_list = false;
    config.exclude_extensions = vec![String::from("*.json")];

    assert!(is_skipped_file(&config, "configs\\data.json"));
    assert!(!is_skipped_file(&config, "readme.txt"), "the built-in list is off");
  }
}
