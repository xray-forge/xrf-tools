use xrf_ltx::Ltx;

use crate::archive_config_ltx::ArchiveConfigLtx;
use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory};

impl ArchivePackConfig {
  /// Section listing files by name rather than by directory.
  const SECTION_INCLUDE_FILES: &str = "include_files";

  /// Section names keep the engine's `folders` spelling: they are the xrCompress dialect, not ours to rename.
  const SECTION_INCLUDE_DIRECTORIES: &str = "include_folders";
  const SECTION_EXCLUDE_DIRECTORIES: &str = "exclude_folders";

  /// Apply an xrCompress configuration already in hand.
  pub fn with_ltx(mut self, ltx: &Ltx) -> Self {
    if let Some(extensions) = ArchiveConfigLtx::read_extensions(ltx) {
      self.exclude_extensions = extensions;
    }

    if let Some(files) = ArchiveConfigLtx::read_names(ltx, Self::SECTION_INCLUDE_FILES) {
      self.include_files = files;
    }

    if let Some(section) = ltx.section(Self::SECTION_INCLUDE_DIRECTORIES) {
      self.include_directories = section.iter().map(Self::directory_from_entry).collect();
    }

    if let Some(section) = ltx.section(Self::SECTION_EXCLUDE_DIRECTORIES) {
      self.exclude_directories = section.iter().map(Self::directory_from_entry).collect();
    }

    if let Some(header) = ArchiveConfigLtx::read_header(ltx) {
      self.header = Some(header);
    }

    self
  }

  fn directory_from_entry((path, value): (&str, &str)) -> ArchivePackDirectory {
    ArchivePackDirectory {
      // `.\` names the source root itself, which is more readable as an empty prefix.
      path: if path == ".\\" || path == "./" {
        String::new()
      } else {
        path.replace('/', "\\").trim_end_matches('\\').to_string()
      },
      is_recursive: matches!(value, "true" | "on" | "yes" | "1"),
    }
  }

  fn write_directories(ltx: &mut Ltx, section_name: &str, directories: &[ArchivePackDirectory]) {
    for directory in directories {
      // An empty path names the packed root, which the dialect spells `.\`.
      let path: &str = if directory.path.is_empty() {
        ".\\"
      } else {
        &directory.path
      };

      ArchiveConfigLtx::set(
        ltx,
        section_name,
        path,
        if directory.is_recursive { "true" } else { "false" },
      );
    }
  }
}

impl From<&ArchivePackConfig> for Ltx {
  /// Write the selection rules back out as an xrCompress configuration.
  ///
  /// The inverse of [`ArchivePackConfig::with_ltx`], covering the same sections and no others: source,
  /// destination, name, mode, and volume size stay with the caller, because a configuration file never
  /// carried them. A file written here reads back through `with_ltx` unchanged.
  fn from(config: &ArchivePackConfig) -> Self {
    let mut ltx: Ltx = Ltx::new();

    ArchiveConfigLtx::write_extensions(&mut ltx, &config.exclude_extensions);
    ArchiveConfigLtx::write_names(
      &mut ltx,
      ArchivePackConfig::SECTION_INCLUDE_FILES,
      &config.include_files,
    );
    ArchivePackConfig::write_directories(
      &mut ltx,
      ArchivePackConfig::SECTION_INCLUDE_DIRECTORIES,
      &config.include_directories,
    );
    ArchivePackConfig::write_directories(
      &mut ltx,
      ArchivePackConfig::SECTION_EXCLUDE_DIRECTORIES,
      &config.exclude_directories,
    );
    ArchiveConfigLtx::write_header(&mut ltx, config.header.as_deref());

    ltx
  }
}

#[cfg(test)]
mod tests {
  use xrf_ltx::Ltx;

  use crate::pack::config::ArchivePackConfig;

  fn config_from_ltx(source: &str) -> ArchivePackConfig {
    ArchivePackConfig::new("gamedata", "db", "configs").with_ltx(&Ltx::read_from_str(source).expect("ltx parses"))
  }

  #[test]
  fn reads_the_xrcompress_dialect() {
    let config: ArchivePackConfig = config_from_ltx(
      "[options]\nexclude_exts = *.txt, *.json\n\n\
       [include_files]\ngamemtl.xr\nshaders.xr\n\n\
       [include_folders]\nconfigs = true\nspawns = false\n\n\
       [exclude_folders]\nlevels\\build = true\n",
    );

    assert_eq!(config.exclude_extensions, vec!["*.txt", "*.json"]);
    assert_eq!(config.include_files, vec!["gamemtl.xr", "shaders.xr"]);
    assert_eq!(config.include_directories.len(), 2);
    assert_eq!(config.include_directories[0].path, "configs");
    assert!(config.include_directories[0].is_recursive);
    assert!(!config.include_directories[1].is_recursive);
    assert_eq!(config.exclude_directories[0].path, "levels\\build");
  }

  #[test]
  fn reads_the_source_root_as_an_empty_prefix() {
    let config: ArchivePackConfig = config_from_ltx("[include_folders]\n.\\ = false\n");

    assert_eq!(config.include_directories[0].path, "");
  }

  #[test]
  fn keeps_the_header_verbatim_for_the_engine_to_parse() {
    let config: ArchivePackConfig =
      config_from_ltx("[header]\nauto_load = true\nentry_point = $fs_root$\\gamedata\\\n");
    let header: &str = config.header.as_deref().expect("header is carried");

    assert!(header.starts_with("[header]\r\n"), "the section names itself");
    assert!(header.contains("auto_load = true\r\n"));
    assert!(header.contains("entry_point = $fs_root$\\gamedata\\\r\n"));
  }
}
