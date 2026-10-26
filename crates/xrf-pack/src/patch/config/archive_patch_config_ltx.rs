use xrf_ltx::Ltx;

use crate::archive_config_ltx::ArchiveConfigLtx;
use crate::patch::config::ArchivePatchConfig;

impl ArchivePatchConfig {
  /// Prefixes the comparison is restricted to, listed as bare names.
  ///
  /// Not `include_folders`: that section of the xrCompress dialect means directories on disk with a recursion flag,
  /// and these are logical prefixes the engine addresses. Borrowing the name would read like that dialect while
  /// meaning something else, so the sections say what they hold.
  const SECTION_INCLUDE: &str = "include";

  /// Prefixes dropped from the comparison, listed as bare names.
  const SECTION_IGNORE: &str = "ignore";

  /// Apply a patching configuration already parsed as LTX.
  ///
  /// A section the file does not carry leaves what the caller already holds, which is what makes an import a
  /// layering step rather than a replacement.
  pub fn with_ltx(mut self, ltx: &Ltx) -> Self {
    if let Some(extensions) = ArchiveConfigLtx::read_extensions(ltx) {
      self.exclude_extensions = extensions;
    }

    if let Some(prefixes) = ArchiveConfigLtx::read_names(ltx, Self::SECTION_INCLUDE) {
      self.include = prefixes;
    }

    if let Some(prefixes) = ArchiveConfigLtx::read_names(ltx, Self::SECTION_IGNORE) {
      self.ignore = prefixes;
    }

    if let Some(header) = ArchiveConfigLtx::read_header(ltx) {
      self.header = Some(header);
    }

    self
  }
}

impl From<&ArchivePatchConfig> for Ltx {
  /// Write the comparison scope and header back out as an LTX configuration.
  ///
  /// The inverse of [`ArchivePatchConfig::with_ltx`], covering the same sections and no others. A file written here
  /// reads back through `with_ltx` unchanged.
  fn from(config: &ArchivePatchConfig) -> Self {
    let mut ltx: Ltx = Ltx::new();

    ArchiveConfigLtx::write_extensions(&mut ltx, &config.exclude_extensions);
    ArchiveConfigLtx::write_names(&mut ltx, ArchivePatchConfig::SECTION_INCLUDE, &config.include);
    ArchiveConfigLtx::write_names(&mut ltx, ArchivePatchConfig::SECTION_IGNORE, &config.ignore);
    ArchiveConfigLtx::write_header(&mut ltx, config.header.as_deref());

    ltx
  }
}
