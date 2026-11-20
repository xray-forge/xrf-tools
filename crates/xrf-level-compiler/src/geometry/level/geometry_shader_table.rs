use xrf_error::XrfResult;
use xrf_utils::narrow_format_size;

/// A level's shader table, `fsL_SHADERS`, as `RegisterShader` fills it: each `<render shader>/<textures>` once, in
/// first-use order, compared without case.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GeometryShaderTable {
  entries: Vec<String>,
}

impl GeometryShaderTable {
  /// The id of entry `name`, registered where new.
  ///
  /// # Errors
  ///
  /// Returns an error when the id would pass a visual's 16-bit shader id.
  pub(crate) fn register(&mut self, name: &str) -> XrfResult<u16> {
    let index: usize = match self.entries.iter().position(|entry| entry.eq_ignore_ascii_case(name)) {
      Some(index) => index,
      None => {
        self.entries.push(String::from(name));
        self.entries.len() - 1
      }
    };

    narrow_format_size(index, "A shader table id")
  }

  /// The entries, by id.
  pub(crate) fn get_entries(&self) -> &[String] {
    &self.entries
  }
}
