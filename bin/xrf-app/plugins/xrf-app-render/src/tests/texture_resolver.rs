use std::fs;
use std::path::PathBuf;

use xrf_app_core::assets::AssetMountState;
use xrf_material::XrayTextureScope;
use xrf_renderer::get_texture_name;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayMountMode, XrayRoots, XraySearch, XrayTransaction};

use crate::TextureResolver;

const REFERENCE: &str = "prop\\prop_fence";

/// A reference is resolved once, a texture it found nothing for included, until its file name is forgotten: then the
/// next read finds what a commit added since.
#[test]
fn resolves_a_reference_once_and_again_after_forget() {
  let root: PathBuf = build_absolute_generated_test_resource_path("render/texture_resolver");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&root).expect("tree made");

  let assets: AssetMountState = AssetMountState::new();
  let roots: XrayRoots = XrayRoots::one(root, XrayMountMode::Directory);
  let resolver: TextureResolver = TextureResolver::new(roots.clone(), XrayTextureScope::shared(), []);

  assert_eq!(resolver.read(&assets, REFERENCE).expect("the read"), None);

  let search: XraySearch = assets.search(&roots).expect("the tree mounts");

  assets
    .commit(
      &XrayTransaction::new(&search)
        .with_write("textures\\prop\\prop_fence.dds", "texels")
        .expect("the write"),
    )
    .expect("the write commits");

  assert_eq!(
    resolver.read(&assets, REFERENCE).expect("the read"),
    None,
    "what was found for it is kept"
  );

  resolver.forget(&[get_texture_name(REFERENCE)]);

  assert_eq!(
    resolver.read(&assets, REFERENCE).expect("the read"),
    Some(b"texels".to_vec())
  );
}
