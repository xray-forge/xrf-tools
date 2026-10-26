/// The textures the game loads a step coarser at the same texture quality, as `system.ltx`'s
/// `reduce_lod_texture_list` names them: a reference is reduced where it holds any part the list names, letter case
/// aside.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct XrayReducedTextures {
  parts: Vec<String>,
}

impl XrayReducedTextures {
  /// The `system.ltx` section listing them, a part a key.
  pub const SECTION: &'static str = "reduce_lod_texture_list";

  /// The list of `parts`, as the section's keys name them.
  pub fn new<T: AsRef<str>>(parts: impl IntoIterator<Item = T>) -> Self {
    Self {
      parts: parts.into_iter().map(|part| part.as_ref().to_lowercase()).collect(),
    }
  }

  /// Whether a texture reference is loaded a step coarser.
  pub fn is_reduced(&self, reference: &str) -> bool {
    let reference: String = reference.to_lowercase();

    self.parts.iter().any(|part| reference.contains(part.as_str()))
  }
}
