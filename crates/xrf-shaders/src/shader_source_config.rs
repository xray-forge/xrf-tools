use xrf_extension::{XrayExtension, XrayExtensionOf};

/// File extensions the engine treats as renderer shader sources.
///
/// A policy of this crate rather than a property of the spellings: `XrayAssetType` calls nine spellings shaders, this
/// compiles seven of them, and the two lists are allowed to differ because they answer different questions.
pub const SHADER_SOURCE_FILE_EXTENSIONS: &[XrayExtension] = &[
  XrayExtension::H,
  XrayExtension::Vs,
  XrayExtension::Ps,
  XrayExtension::Cs,
  XrayExtension::Hs,
  XrayExtension::Ds,
  XrayExtension::Gs,
];

/// Lua renderer-definition script extension.
pub const SHADER_SCRIPT_FILE_EXTENSION: XrayExtension = XrayExtension::S;

/// Whether a name has a shader source extension recognized by the engine.
///
/// Takes the engine identity rather than a `Path`, because that is what every caller holds and what the rule is about:
/// a shader tree is enumerated through the VFS, and `Path::extension` answers `None` for `shaders\r1\.s` — the one
/// name in the tree whose whole spelling is its extension.
pub fn is_shader_source_path(name: &str) -> bool {
  XrayExtensionOf::of(name)
    .known()
    .is_some_and(|extension| SHADER_SOURCE_FILE_EXTENSIONS.contains(&extension))
}

#[cfg(test)]
mod tests {
  use xrf_extension::XrayExtension;

  use super::{SHADER_SCRIPT_FILE_EXTENSION, SHADER_SOURCE_FILE_EXTENSIONS, is_shader_source_path};

  #[test]
  fn recognizes_every_compiled_source_spelling_without_case() {
    for extension in SHADER_SOURCE_FILE_EXTENSIONS {
      assert!(is_shader_source_path(&format!("shaders\\r3\\model.{extension}")));
      assert!(is_shader_source_path(&format!(
        "shaders\\r3\\model.{}",
        extension.as_str().to_uppercase()
      )));
    }
  }

  #[test]
  fn a_script_is_not_a_source_and_neither_is_anything_else_beside_them() {
    // Scripts are Lua the renderer runs, not HLSL it compiles, so the two sets stay apart.
    assert!(!is_shader_source_path("shaders\\r3\\basic.s"));
    assert!(!is_shader_source_path("shaders\\r3\\basic.s_"));
    assert!(!is_shader_source_path("shaders\\r3\\readme.txt"));
    assert!(!is_shader_source_path("shaders\\r3\\notes"));
  }

  #[test]
  fn a_name_merely_ending_in_a_source_spelling_is_not_a_source() {
    assert!(!is_shader_source_path("shaders\\r3\\lightps"));
    assert!(!is_shader_source_path("shaders\\r3\\model.myps"));
  }

  #[test]
  fn the_script_spelling_is_the_one_whose_whole_name_the_engine_ships() {
    // `shaders\r1\.s` is a Lua script the engine loads; `Path::extension` calls it a hidden file.
    assert_eq!(SHADER_SCRIPT_FILE_EXTENSION, XrayExtension::S);
    assert!(SHADER_SCRIPT_FILE_EXTENSION.matches("shaders\\r1\\.s"));
  }
}
