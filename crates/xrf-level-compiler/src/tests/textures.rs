//! How the compilers name and find the textures `build.prj` names.

use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::compiler_textures::CompilerTextures;

#[test]
fn names_a_texture_by_what_precedes_its_first_dot_lowered() {
  assert_eq!(CompilerTextures::get_name("Trees\\Leaf.Final.dds"), "trees\\leaf");
  assert_eq!(CompilerTextures::get_name("level_lods"), CompilerTextures::UNLOADED);
  assert_eq!(
    CompilerTextures::get_path(Path::new("textures"), "trees\\leaf", "thm"),
    Path::new("textures").join("trees").join("leaf.thm")
  );
}

#[test]
fn finds_each_file_under_the_first_root_holding_it() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "roots"))?;
  let roots: Vec<PathBuf> = vec![root.join("first"), root.join("second")];

  for (index, extension) in [(0, "thm"), (1, "thm"), (1, "dds")] {
    std::fs::create_dir_all(roots[index].join("trees"))?;
    std::fs::write(roots[index].join("trees").join(format!("leaf.{extension}")), b"")?;
  }

  assert_eq!(
    CompilerTextures::find(&roots, "trees\\leaf", "thm"),
    Some(roots[0].join("trees").join("leaf.thm"))
  );
  assert_eq!(
    CompilerTextures::find(&roots, "trees\\leaf", "dds"),
    Some(roots[1].join("trees").join("leaf.dds"))
  );
  assert_eq!(CompilerTextures::find(&roots, "trees\\bark", "dds"), None);

  Ok(())
}
