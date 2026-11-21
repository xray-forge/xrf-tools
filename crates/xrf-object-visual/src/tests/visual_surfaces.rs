use xrf_material::fixtures::FixtureTree;
use xrf_material::{
  XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw, XraySurfaceResolver, XrayTextureScope,
};
use xrf_rawdata::ObjectCompileOptions;
use xrf_shaders::ShaderBlenderClass;
use xrf_shaders::fixtures::ShaderBlenderFixture;
use xrf_vfs::{XrayLookupScope, XrayMountId, XraySearch, XrayVfs};
use xrf_visual::VisualSurfaces;

use crate::object_visual::ObjectVisual;
use crate::tests::fixtures::{compile_source, object_of, quad, surface};

fn mount_search(tree: &FixtureTree) -> XraySearch {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("the tree mounts");

  XraySearch::new(vfs).with_step("tree", XrayLookupScope::only([id]))
}

#[test]
fn answers_each_submesh_in_its_order_by_the_shader_it_declares() {
  let tree: FixtureTree = FixtureTree::new("object_visual_surfaces")
    .with_shader_library(&[ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "models\\model")]);
  let visual: ObjectVisual = ObjectVisual::compile(
    &object_of(&[
      surface("models\\model", "wood\\planks", &quad(1.0, 1.0)),
      surface("models\\missing", "wood\\nails", &quad(1.0, 1.0)),
    ]),
    &compile_source(),
    &ObjectCompileOptions::default(),
  )
  .expect("the object compiles");
  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(&mount_search(&tree), XrayTextureScope::shared());
  let surfaces: Vec<XraySurfaceDescriptor> = VisualSurfaces::describe(&resolver, &visual.package.description);

  assert_eq!(
    surfaces.len(),
    visual.package.description.submeshes.len(),
    "one answer a submesh"
  );
  assert_eq!(surfaces[0].draw, XraySurfaceDraw::Opaque);
  assert_ne!(surfaces[0].declaration, XraySurfaceDeclaration::Undefined);
  assert_eq!(
    surfaces[1].declaration,
    XraySurfaceDeclaration::Undefined,
    "a shader the library lacks is named so rather than left out"
  );
}
