use std::path::{Path, PathBuf};

use xrf_db::fixtures::{ShaderBlenderFixture, ShaderLibraryFixture};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::fixtures::thm_fixture::ThmFixture;
use crate::resolve::xray_surface_resolver::XraySurfaceResolver;

/// A gamedata-shaped tree under the generated test resources, with descriptors that parse and textures that exist.
///
/// Texture files are placeholders — resolution looks for existence, never inside a DDS — while descriptor files are
/// real chunked bytes, because a descriptor that does not parse is one of the states under test rather than an
/// accident. The root is cleared first, so a tree means exactly what its calls say.
pub struct FixtureTree {
  root: PathBuf,
}

impl FixtureTree {
  /// The empty tree for `case`, its root cleared of any earlier run.
  ///
  /// # Panics
  ///
  /// When the generated test resources directory cannot be created, which no test can proceed past.
  pub fn new(case: &str) -> Self {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xrf_material/{case}"));

    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("fixture tree root is creatable");

    Self { root }
  }

  pub fn root(&self) -> &Path {
    &self.root
  }

  /// A placeholder `.dds` at `textures\<reference>.dds`.
  pub fn with_texture(self, reference: &str) -> Self {
    self.write(
      &format!("textures/{}.dds", reference.replace('\\', "/")),
      reference.as_bytes(),
    );
    self
  }

  /// A descriptor at `textures\<reference>.thm`.
  pub fn with_descriptor(self, reference: &str, fixture: &ThmFixture) -> Self {
    self.write(
      &format!("textures/{}.thm", reference.replace('\\', "/")),
      &fixture.to_bytes(),
    );
    self
  }

  /// Bytes at `textures\<reference>.thm` that are not a descriptor.
  pub fn with_unreadable_descriptor(self, reference: &str) -> Self {
    self.write(&format!("textures/{}.thm", reference.replace('\\', "/")), b"not a thm");
    self
  }

  /// A shader library at the tree's root, defining exactly these blenders.
  pub fn with_shader_library(self, blenders: &[ShaderBlenderFixture]) -> Self {
    let bytes: Vec<u8> = ShaderLibraryFixture::library(blenders).expect("shader library is writable");

    self.write(XraySurfaceResolver::SHADER_LIBRARY_LOGICAL_PATH, &bytes);
    self
  }

  /// Bytes at the library's path that are not a library.
  pub fn with_unreadable_shader_library(self) -> Self {
    self.write(XraySurfaceResolver::SHADER_LIBRARY_LOGICAL_PATH, b"not a library");
    self
  }

  /// The three dummies `texture_load` substitutes, so a substitution has something to land on.
  pub fn with_engine_dummies(self) -> Self {
    self
      .with_texture("ed\\ed_dummy_bump")
      .with_texture("ed\\ed_dummy_bump#")
      .with_texture("ed\\ed_not_existing_texture")
  }

  fn write(&self, relative: &str, bytes: &[u8]) {
    let path: PathBuf = self.root.join(relative);

    std::fs::create_dir_all(path.parent().expect("fixture file sits in a directory"))
      .expect("fixture directory is creatable");
    std::fs::write(&path, bytes).expect("fixture file is writable");
  }
}
