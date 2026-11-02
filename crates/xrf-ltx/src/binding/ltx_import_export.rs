use xrf_error::XrfResult;

use crate::Ltx;

/// Field an exported record declares its own kind in, so a reader knows which type to import it back as.
pub const META_TYPE_FIELD: &str = "$type";

/// A record that is read from and written to one section of a config.
pub trait LtxImportExport: Sized {
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self>;

  fn import_optional(section_name: &str, ltx: &Ltx) -> XrfResult<Option<Self>> {
    if ltx.has_section(section_name) {
      Self::import(section_name, ltx).map(Some)
    } else {
      Ok(None)
    }
  }

  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult;

  fn export_optional(section_name: &str, ltx: &mut Ltx, data: Option<&Self>) -> XrfResult {
    if let Some(data) = data {
      data.export(section_name, ltx)
    } else {
      Ok(())
    }
  }
}
