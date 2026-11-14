/// A level's shader table, `fsL_SHADERS`, as xrLC registers its entries (`RegisterShader`, `xrLC/xrSaveOGF.cpp:29`):
/// each `<render shader>/<textures>` once, in the order first asked for, compared without case.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GeometryShaderTable {
  entries: Vec<String>,
}

impl GeometryShaderTable {
  /// The entry `name` has, registered where it has none.
  pub fn register(&mut self, name: &str) -> u16 {
    let index: usize = match self.entries.iter().position(|entry| entry.eq_ignore_ascii_case(name)) {
      Some(index) => index,
      None => {
        self.entries.push(String::from(name));
        self.entries.len() - 1
      }
    };

    index as u16
  }

  /// The entries, by id.
  pub fn get_entries(&self) -> &[String] {
    &self.entries
  }
}
