//! Resolution of a visual's own references, against small trees built in the test.

use std::path::PathBuf;

use xrf_math::Vector3d;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayLookupScope, XrayMountId, XrayProbe, XrayResolution, XrayVfs};

use crate::data::visual_bounds::VisualBounds;
use crate::data::visual_box::VisualBox;
use crate::data::visual_description::VisualDescription;
use crate::data::visual_sphere::VisualSphere;
use crate::data::visual_submesh::VisualSubmesh;
use crate::data::visual_submesh_content::VisualSubmeshContent;
use crate::resolve::visual_dependencies::VisualDependencies;

fn tree(case: &str, files: &[&str]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("visual_dependencies/{case}"));

  let _ = std::fs::remove_dir_all(&root);

  for file in files {
    let path: PathBuf = root.join(file);

    std::fs::create_dir_all(path.parent().expect("file sits in a directory")).expect("test tree is creatable");
    std::fs::write(&path, case.as_bytes()).expect("test file is writable");
  }

  root
}

fn submesh(index: u32, texture: Option<&str>) -> VisualSubmesh {
  VisualSubmesh {
    index,
    model_type: 5,
    model_type_label: "geomdefSt".into(),
    texture_name: texture.map(str::to_string),
    shader_name: None,
    content: VisualSubmeshContent::Skipped {
      cause: crate::data::visual_skip_cause::VisualSkipCause::Unsupported,
      reason: "geometry is irrelevant to resolution".into(),
    },
  }
}

/// An extent no assertion here reads, since resolution never looks at geometry.
fn bounds() -> VisualBounds {
  let origin: Vector3d = Vector3d { x: 0.0, y: 0.0, z: 0.0 };

  VisualBounds {
    bounding_box: VisualBox {
      min: origin.clone(),
      max: origin.clone(),
    },
    bounding_sphere: VisualSphere {
      center: origin,
      radius: 0.0,
    },
  }
}

fn description(submeshes: Vec<VisualSubmesh>, motion_refs: Vec<String>) -> VisualDescription {
  VisualDescription {
    version: 4,
    model_type: 3,
    model_type_label: "skeletonAnim".into(),
    shader_id: 0,
    source_file: None,
    declared_bounds: bounds(),
    computed_bounds: None,
    submeshes,
    bones: Vec::new(),
    motion_refs,
    embedded_motions: Vec::new(),
    buffer_length: 0,
  }
}

/// One mounted tree and a probe naming it, which is the smallest useful search order.
fn probe_over(vfs: &XrayVfs, id: XrayMountId) -> XrayProbe<'_> {
  vfs.probe().with_step("tree", XrayLookupScope::only([id]))
}

#[test]
fn resolves_a_declared_texture_and_skips_a_submesh_declaring_none() {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs
    .mount_directory("", tree("texture", &["textures/wpn/wpn_ak74.dds"]))
    .expect("tree mounts");

  let dependencies: VisualDependencies = VisualDependencies::resolve(
    &description(vec![submesh(0, None), submesh(1, Some("wpn\\wpn_ak74"))], Vec::new()),
    &probe_over(&vfs, id),
  );

  assert_eq!(dependencies.textures.len(), 1, "a submesh with no texture has no entry");
  assert_eq!(dependencies.find_texture(0), None);

  let texture: &crate::VisualTextureDependency = dependencies.find_texture(1).expect("submesh 1 declares a texture");

  assert_eq!(texture.reference, "wpn\\wpn_ak74");
  assert_eq!(texture.resolution.get_step(), Some("tree"));
  assert_eq!(
    texture.resolution.get_asset().map(|it| it.get_logical_path().as_str()),
    Some("textures\\wpn\\wpn_ak74.dds")
  );
}

#[test]
fn substitutes_the_engine_dummy_for_an_absent_texture() {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs
    .mount_directory("", tree("dummy", &["textures/ed/ed_not_existing_texture.dds"]))
    .expect("tree mounts");

  let dependencies: VisualDependencies = VisualDependencies::resolve(
    &description(vec![submesh(0, Some("wpn\\wpn_absent"))], Vec::new()),
    &probe_over(&vfs, id),
  );

  assert!(
    matches!(
      &dependencies.textures[0].resolution,
      XrayResolution::Substituted { fallback, .. } if fallback == VisualDependencies::MISSING_TEXTURE_REFERENCE
    ),
    "an absent texture falls back to what the renderer loads: {:?}",
    dependencies.textures[0].resolution
  );
}

#[test]
fn reports_a_missing_texture_when_even_the_dummy_is_absent() {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs
    .mount_directory("", tree("missing", &["textures/wpn/wpn_abakan.dds"]))
    .expect("tree mounts");

  let dependencies: VisualDependencies = VisualDependencies::resolve(
    &description(vec![submesh(0, Some("wpn\\wpn_absent"))], Vec::new()),
    &probe_over(&vfs, id),
  );

  assert!(matches!(
    dependencies.textures[0].resolution,
    XrayResolution::Missing { .. }
  ));
}

#[test]
fn resolves_a_motion_reference_without_substituting_anything() {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs
    .mount_directory(
      "",
      tree(
        "motions",
        &["meshes/wpn/wpn_ak74_hud.omf", "textures/ed/ed_not_existing_texture.dds"],
      ),
    )
    .expect("tree mounts");

  let dependencies: VisualDependencies = VisualDependencies::resolve(
    &description(Vec::new(), vec!["wpn\\wpn_ak74_hud".into(), "wpn\\wpn_absent".into()]),
    &probe_over(&vfs, id),
  );

  assert_eq!(dependencies.motions[0].resolution.get_step(), Some("tree"));
  assert!(
    matches!(dependencies.motions[1].resolution, XrayResolution::Missing { .. }),
    "nothing stands in for an absent motion set: {:?}",
    dependencies.motions[1].resolution
  );
}

#[test]
fn a_masked_motion_reference_holds_every_match() {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs
    .mount_directory(
      "",
      tree(
        "mask",
        &[
          "meshes/wpn/wpn_ak74_hud.omf",
          "meshes/wpn/wpn_ak74_world.omf",
          "meshes/wpn/wpn_abakan_hud.omf",
        ],
      ),
    )
    .expect("tree mounts");

  let dependencies: VisualDependencies = VisualDependencies::resolve(
    &description(Vec::new(), vec!["wpn\\wpn_ak74_*.omf".into()]),
    &probe_over(&vfs, id),
  );

  assert_eq!(dependencies.motions[0].resolution.get_assets().len(), 2);
}

#[test]
fn one_rejected_reference_does_not_cost_the_model_its_other_assets() {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs
    .mount_directory("", tree("rejected", &["textures/wpn/wpn_ak74.dds"]))
    .expect("tree mounts");

  // Engine text is untrusted, and a traversal is what a logical path refuses.
  let dependencies: VisualDependencies = VisualDependencies::resolve(
    &description(
      vec![submesh(0, Some("..\\..\\outside")), submesh(1, Some("wpn\\wpn_ak74"))],
      Vec::new(),
    ),
    &probe_over(&vfs, id),
  );

  assert!(matches!(
    dependencies.textures[0].resolution,
    XrayResolution::Rejected { .. }
  ));
  assert_eq!(dependencies.textures[1].resolution.get_step(), Some("tree"));
}

#[test]
fn reports_no_scope_when_no_source_is_mounted() {
  let vfs: XrayVfs = XrayVfs::new();

  let dependencies: VisualDependencies = VisualDependencies::resolve(
    &description(
      vec![submesh(0, Some("wpn\\wpn_ak74"))],
      vec!["wpn\\wpn_ak74_hud".into()],
    ),
    &vfs.probe().with_step("nothing", XrayLookupScope::all()),
  );

  assert_eq!(dependencies.textures[0].resolution, XrayResolution::NoScope);
  assert_eq!(dependencies.motions[0].resolution, XrayResolution::NoScope);
}
