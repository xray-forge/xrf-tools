use xrf_error::XrfResult;
use xrf_ltx::Ltx;

use crate::pack::config::ArchivePackHeaderEntry;

/// The LTX sections a packing and a patching configuration share, read and written the one way both mean them.
///
/// Both files carry `[options] exclude_exts` and a verbatim `[header]`, and both list some section as bare names;
/// only which sections those are differs. Each configuration's codec keeps its own sections and asks this for the
/// shared ones, so the two dialects cannot drift apart on what a common section holds.
pub(crate) struct ArchiveConfigLtx;

impl ArchiveConfigLtx {
  /// Section holding the extension patterns that keep a file out, spelled as xrCompress spells it.
  pub(crate) const SECTION_OPTIONS: &str = "options";

  /// The `[options]` key listing those patterns, comma-separated.
  pub(crate) const KEY_EXCLUDE_EXTENSIONS: &str = "exclude_exts";

  /// Section copied into the archive verbatim, which is what tells the engine where to mount it.
  pub(crate) const SECTION_HEADER: &str = "header";

  /// `[options] exclude_exts` split into its patterns, or none when the file does not carry the key.
  pub(crate) fn read_extensions(ltx: &Ltx) -> Option<Vec<String>> {
    let extensions: &str = ltx.section(Self::SECTION_OPTIONS)?.get(Self::KEY_EXCLUDE_EXTENSIONS)?;

    Some(
      extensions
        .split(',')
        .map(str::trim)
        .filter(|pattern| !pattern.is_empty())
        .map(String::from)
        .collect(),
    )
  }

  /// A section's keys as bare names, or none when the file does not carry the section.
  pub(crate) fn read_names(ltx: &Ltx, section_name: &str) -> Option<Vec<String>> {
    let section = ltx.section(section_name)?;

    Some(section.iter().map(|(name, _)| String::from(name)).collect())
  }

  /// `[header]` as the verbatim text chunk 666 stores, or none when the file does not carry the section.
  pub(crate) fn read_header(ltx: &Ltx) -> Option<String> {
    let entries: Vec<ArchivePackHeaderEntry> = ltx
      .section(Self::SECTION_HEADER)?
      .iter()
      .map(|(key, value)| ArchivePackHeaderEntry {
        key: String::from(key),
        value: String::from(value),
      })
      .collect();

    Some(ArchivePackHeaderEntry::join(&entries))
  }

  /// Write `[options] exclude_exts`, leaving the section out when there are no patterns.
  pub(crate) fn write_extensions(ltx: &mut Ltx, extensions: &[String]) {
    if !extensions.is_empty() {
      Self::set(
        ltx,
        Self::SECTION_OPTIONS,
        Self::KEY_EXCLUDE_EXTENSIONS,
        &extensions.join(","),
      );
    }
  }

  /// Write names as the bare keys of a section, which carry no value, the way xrCompress reads `[include_files]`.
  pub(crate) fn write_names(ltx: &mut Ltx, section_name: &str, names: &[String]) {
    for name in names {
      Self::set(ltx, section_name, name, "");
    }
  }

  /// Write the header text back out as the `[header]` section it was read from.
  pub(crate) fn write_header(ltx: &mut Ltx, header: Option<&str>) {
    for entry in header.map(ArchivePackHeaderEntry::split).unwrap_or_default() {
      Self::set(ltx, Self::SECTION_HEADER, &entry.key, &entry.value);
    }
  }

  /// Set one key in a section, creating the section on first use.
  pub(crate) fn set(ltx: &mut Ltx, section_name: &str, key: &str, value: &str) {
    ltx.with_section(section_name).set(key, value);
  }

  /// The bytes of a configuration file.
  ///
  /// # Errors
  ///
  /// Returns an encoding error when a rule cannot be written in the Windows-1251 the reader decodes.
  pub(crate) fn render(ltx: &Ltx) -> XrfResult<Vec<u8>> {
    let mut rendered: Vec<u8> = Vec::new();

    ltx.write_to(&mut rendered)?;

    Ok(rendered)
  }
}
