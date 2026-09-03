use xrf_error::XrfResult;
use xrf_ltx::Ltx;

use crate::pack::config::ArchivePackHeaderEntry;
use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory};

/// Section holding the extension patterns that keep a file out.
const SECTION_OPTIONS: &str = "options";

/// Section listing files by name rather than by directory.
const SECTION_INCLUDE_FILES: &str = "include_files";

/// Section names keep the engine's `folders` spelling: they are the xrCompress dialect, not ours to rename.
const SECTION_INCLUDE_DIRECTORIES: &str = "include_folders";
const SECTION_EXCLUDE_DIRECTORIES: &str = "exclude_folders";

/// Section copied into the archive verbatim, which is what tells the engine where to mount it.
const SECTION_HEADER: &str = "header";

impl ArchivePackConfig {
  /// Apply an xrCompress configuration already in hand.
  ///
  /// Reads the dialect `ProcessLTX` accepts: `[options] exclude_exts`, `[include_files]` as bare names,
  /// `[include_folders]` and `[exclude_folders]` as `path = <bool>`, and a `[header]` copied verbatim. A section the
  /// file does not carry leaves what the caller already holds, which is what makes an import a layering step.
  ///
  /// Reading one from disk is [`ArchivePackConfig::with_config_file`]'s, which also accepts JSON.
  pub fn with_ltx(mut self, ltx: &Ltx) -> XrfResult<Self> {
    if let Some(section) = ltx.section("options")
      && let Some(extensions) = section.get("exclude_exts")
    {
      self.exclude_extensions = extensions
        .split(',')
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(String::from)
        .collect();
    }

    if let Some(section) = ltx.section("include_files") {
      // Files are listed as bare names, so only the key carries meaning.
      self.include_files = section.iter().map(|(name, _)| String::from(name)).collect();
    }

    if let Some(section) = ltx.section("include_folders") {
      self.include_directories = section.iter().map(Self::directory_from_entry).collect();
    }

    if let Some(section) = ltx.section("exclude_folders") {
      self.exclude_directories = section.iter().map(Self::directory_from_entry).collect();
    }

    if let Some(section) = ltx.section("header") {
      let mut header: String = String::from("[header]\r\n");

      for (key, value) in section.iter() {
        header.push_str(key);
        header.push_str(" = ");
        header.push_str(value);
        header.push_str("\r\n");
      }

      self.header = Some(header);
    }

    Ok(self)
  }

  /// Render the selection rules as the bytes of an xrCompress configuration.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when a rule cannot be written in the Windows-1251 the reader decodes.
  pub(crate) fn to_ltx_bytes(&self) -> XrfResult<Vec<u8>> {
    let mut rendered: Vec<u8> = Vec::new();

    self.to_ltx().write_to(&mut rendered)?;

    Ok(rendered)
  }

  /// Write the selection rules back out as an xrCompress configuration.
  ///
  /// The inverse of [`ArchivePackConfig::with_ltx`], covering the same sections and no others: source,
  /// destination, name, mode, and volume size stay with the caller, because a configuration file never
  /// carried them. A file written here reads back through `with_ltx` unchanged.
  pub fn to_ltx(&self) -> Ltx {
    let mut ltx: Ltx = Ltx::new();

    if !self.exclude_extensions.is_empty() {
      Self::set_entry(
        &mut ltx,
        SECTION_OPTIONS,
        "exclude_exts",
        &self.exclude_extensions.join(","),
      );
    }

    // Listed files carry no value, matching how xrCompress reads the section as bare names.
    for name in &self.include_files {
      Self::set_entry(&mut ltx, SECTION_INCLUDE_FILES, name, "");
    }

    Self::write_directories(&mut ltx, SECTION_INCLUDE_DIRECTORIES, &self.include_directories);
    Self::write_directories(&mut ltx, SECTION_EXCLUDE_DIRECTORIES, &self.exclude_directories);

    if let Some(header) = &self.header {
      for entry in ArchivePackHeaderEntry::split(header) {
        Self::set_entry(&mut ltx, SECTION_HEADER, &entry.key, &entry.value);
      }
    }

    ltx
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

      Self::set_entry(
        ltx,
        section_name,
        path,
        if directory.is_recursive { "true" } else { "false" },
      );
    }
  }

  /// Set one key in a section, creating the section on first use.
  ///
  /// Written per entry rather than through a section setter, which borrows for its whole lifetime and so
  /// cannot be driven from a loop.
  fn set_entry(ltx: &mut Ltx, section_name: &str, key: &str, value: &str) {
    ltx
      .entry(section_name.into())
      .or_insert_with(Default::default)
      .insert(key, value);
  }
}

#[cfg(test)]
mod tests {
  use xrf_ltx::Ltx;

  use crate::pack::config::ArchivePackConfig;

  fn config_from_ltx(source: &str) -> ArchivePackConfig {
    ArchivePackConfig::new("gamedata", "db", "configs")
      .with_ltx(&Ltx::read_from_str(source).expect("ltx parses"))
      .expect("ltx applies")
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
