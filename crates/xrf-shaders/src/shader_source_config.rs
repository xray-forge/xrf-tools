use xrf_extension::{XrayExtension, XrayExtensionOf};

/// File extensions the engine treats as renderer shader sources.
pub const SHADER_SOURCE_FILE_EXTENSIONS: &[XrayExtension] = &[
  XrayExtension::H,
  XrayExtension::Vs,
  XrayExtension::Ps,
  XrayExtension::Cs,
  XrayExtension::Hs,
  XrayExtension::Ds,
  XrayExtension::Gs,
];

/// Whether a name has a shader source extension recognized by the engine.
///
/// Takes the engine identity rather than a `Path` because that is what every caller holds: a shader tree is enumerated
/// through the VFS, whose listings are logical paths, and the one caller passes a listing's name straight in.
pub fn is_shader_source_path(name: &str) -> bool {
  XrayExtensionOf::of(name)
    .known()
    .is_some_and(|extension| SHADER_SOURCE_FILE_EXTENSIONS.contains(&extension))
}

#[cfg(test)]
mod tests {
  use xrf_extension::XrayExtension;

  use super::{SHADER_SOURCE_FILE_EXTENSIONS, is_shader_source_path};

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
    assert!(XrayExtension::S.matches("shaders\\r1\\.s"));
    assert!(!SHADER_SOURCE_FILE_EXTENSIONS.contains(&XrayExtension::S));
  }
}
