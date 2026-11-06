use xrf_error::XrfResult;

use crate::binding::{LtxRead, LtxWrite};

/// Field an exported record declares its own kind in, so a reader knows which type to import it back as.
pub const META_TYPE_FIELD: &str = "$type";

/// A record that is read from and written to one section of a config or a record file.
///
/// `import` names its reader's type, unlike `export`'s writer, so an import can name the section type it reads
/// (`&L::Section`).
pub trait LtxImportExport: Sized {
  fn import<L: LtxRead>(section_name: &str, ltx: &L) -> XrfResult<Self>;

  fn import_optional<L: LtxRead>(section_name: &str, ltx: &L) -> XrfResult<Option<Self>> {
    if ltx.has_section(section_name) {
      Self::import(section_name, ltx).map(Some)
    } else {
      Ok(None)
    }
  }

  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite) -> XrfResult;

  fn export_optional(section_name: &str, ltx: &mut impl LtxWrite, data: Option<&Self>) -> XrfResult {
    if let Some(data) = data {
      data.export(section_name, ltx)
    } else {
      Ok(())
    }
  }
}
