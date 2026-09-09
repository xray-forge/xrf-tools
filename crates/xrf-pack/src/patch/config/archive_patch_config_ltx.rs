use xrf_error::XrfResult;
use xrf_ltx::Ltx;

use crate::pack::config::ArchivePackHeaderEntry;
use crate::patch::config::ArchivePatchConfig;

/// Section holding the extension patterns that keep a file out, spelled as xrCompress spells it.
const SECTION_OPTIONS: &str = "options";

/// Prefixes the comparison is restricted to, listed as bare names.
///
/// Not `include_folders`: that section of the xrCompress dialect means directories on disk with a recursion flag,
/// and these are logical prefixes the engine addresses. Borrowing the name would read like that dialect while
/// meaning something else, so the sections say what they hold.
const SECTION_INCLUDE: &str = "include";

/// Prefixes dropped from the comparison, listed as bare names.
const SECTION_IGNORE: &str = "ignore";

/// Section copied into the published volumes verbatim, which is what tells the engine where to mount them.
const SECTION_HEADER: &str = "header";

impl ArchivePatchConfig {
  /// Apply a patching configuration already parsed as LTX.
  ///
  /// A section the file does not carry leaves what the caller already holds, which is what makes an import a
  /// layering step rather than a replacement.
  pub fn with_ltx(mut self, ltx: &Ltx) -> XrfResult<Self> {
    if let Some(section) = ltx.section(SECTION_OPTIONS)
      && let Some(extensions) = section.get("exclude_exts")
    {
      self.exclude_extensions = extensions
        .split(',')
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(String::from)
        .collect();
    }

    if let Some(section) = ltx.section(SECTION_INCLUDE) {
      self.include = section.iter().map(|(name, _)| String::from(name)).collect();
    }

    if let Some(section) = ltx.section(SECTION_IGNORE) {
      self.ignore = section.iter().map(|(name, _)| String::from(name)).collect();
    }

    if let Some(section) = ltx.section(SECTION_HEADER) {
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

  /// Render the comparison scope and header as the bytes of an LTX configuration.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when a rule cannot be written in the Windows-1251 the reader decodes.
  pub(crate) fn to_ltx_bytes(&self) -> XrfResult<Vec<u8>> {
    let mut rendered: Vec<u8> = Vec::new();

    self.to_ltx().write_to(&mut rendered)?;

    Ok(rendered)
  }

  /// Write the comparison scope and header back out as an LTX configuration.
  ///
  /// The inverse of [`ArchivePatchConfig::with_ltx`], covering the same sections and no others. A file written here
  /// reads back through `with_ltx` unchanged.
  pub fn to_ltx(&self) -> Ltx {
    let mut ltx: Ltx = Ltx::new();

    if !self.exclude_extensions.is_empty() {
      set_entry(
        &mut ltx,
        SECTION_OPTIONS,
        "exclude_exts",
        &self.exclude_extensions.join(","),
      );
    }

    // Prefixes carry no value, the way xrCompress reads `[include_files]` as bare names.
    for prefix in &self.include {
      set_entry(&mut ltx, SECTION_INCLUDE, prefix, "");
    }

    for prefix in &self.ignore {
      set_entry(&mut ltx, SECTION_IGNORE, prefix, "");
    }

    if let Some(header) = &self.header {
      for entry in ArchivePackHeaderEntry::split(header) {
        set_entry(&mut ltx, SECTION_HEADER, &entry.key, &entry.value);
      }
    }

    ltx
  }
}

/// Write one entry, creating its section on first use.
fn set_entry(ltx: &mut Ltx, section_name: &str, key: &str, value: &str) {
  ltx.with_section(section_name).set(key, value);
}
