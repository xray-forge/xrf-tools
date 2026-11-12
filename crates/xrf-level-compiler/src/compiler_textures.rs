use std::path::{Path, PathBuf};

/// How the SDK's compilers name and find the textures `build.prj` names, under `$game_textures$`.
pub(crate) struct CompilerTextures;

impl CompilerTextures {
  /// The texture the compilers take without loading its `.thm` or image, the merged impostors' atlas
  /// (`xrLC/Build_Load.cpp:304`, `xrLC_Light/global_calculation_data.cpp:140`).
  pub(crate) const UNLOADED: &'static str = "level_lods";

  /// A texture's name as the compilers look it up: `build.prj`'s, cut at its first dot and lowered
  /// (`xrLC_Light/global_calculation_data.cpp:137`).
  pub(crate) fn get_name(name: &str) -> String {
    name.split('.').next().unwrap_or_default().to_ascii_lowercase()
  }

  /// A texture's file under a root, `<root>\<name>.<extension>`, for a name [`Self::get_name`] gave.
  pub(crate) fn get_path(root: &Path, name: &str, extension: &str) -> PathBuf {
    name
      .split(['\\', '/'])
      .fold(root.to_path_buf(), |path, part| path.join(part))
      .with_extension(extension)
  }

  /// A texture's file under the first of `roots` holding it.
  pub(crate) fn find(roots: &[PathBuf], name: &str, extension: &str) -> Option<PathBuf> {
    roots
      .iter()
      .map(|root| Self::get_path(root, name, extension))
      .find(|path| path.is_file())
  }
}
