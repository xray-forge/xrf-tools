//! What a shader name comes to, one test per state and one per class the renderer draws a mesh with.

use xrf_db::ShaderBlenderClass;
use xrf_db::fixtures::ShaderBlenderFixture;
use xrf_vfs::{XrayMountId, XrayProbe, XrayVfs};

use crate::fixtures::FixtureTree;
use crate::tests::material_probe::probe_over;
use crate::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw, XraySurfaceResolver};

const CUT_OUT: XraySurfaceDraw = XraySurfaceDraw::AlphaTested {
  reference: XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE,
};

/// Opens `tree` and describes one shader name in it.
fn describe(tree: &FixtureTree, shader: &str) -> XraySurfaceDescriptor {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe<'_> = probe_over(&vfs, id);

  XraySurfaceResolver::open(&probe).describe(shader)
}

/// A tree whose library defines exactly `blenders`.
fn library(case: &str, blenders: &[ShaderBlenderFixture]) -> FixtureTree {
  FixtureTree::new(case).with_shader_library(blenders)
}

#[test]
fn a_tree_without_a_library_answers_no_library_and_draws_opaque() {
  let descriptor: XraySurfaceDescriptor = describe(&FixtureTree::new("surface_no_library"), "models\\model");

  assert_eq!(descriptor.declaration, XraySurfaceDeclaration::NoLibrary);
  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
  assert_eq!(descriptor.library, None);
}

#[test]
fn a_library_that_does_not_parse_is_unreadable_and_named() {
  let descriptor: XraySurfaceDescriptor = describe(
    &FixtureTree::new("surface_unreadable").with_unreadable_shader_library(),
    "models\\model",
  );

  assert!(
    matches!(descriptor.declaration, XraySurfaceDeclaration::Unreadable { .. }),
    "{:?}",
    descriptor.declaration
  );
  assert!(descriptor.library.is_some(), "the file that failed is named");
  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
}

#[test]
fn a_name_no_blender_defines_is_undefined_rather_than_absent() {
  // What the engine reports as `! Shader not found in library` and then draws with the default shader.
  let descriptor: XraySurfaceDescriptor = describe(
    &library("surface_undefined", &[ShaderBlenderFixture::model("models\\model")]),
    "models\\invented_by_a_mod",
  );

  assert_eq!(descriptor.declaration, XraySurfaceDeclaration::Undefined);
  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
  assert!(descriptor.library.is_some(), "the library that lacks it is named");
}

#[test]
fn a_model_with_its_switch_off_reads_no_alpha() {
  // `models\model` itself: what 791 of the 879 measured meshes name, and it ignores alpha entirely.
  let descriptor: XraySurfaceDescriptor = describe(
    &library("surface_model", &[ShaderBlenderFixture::model("models\\model")]),
    "models\\model",
  );

  assert_eq!(
    descriptor.declaration,
    XraySurfaceDeclaration::Described {
      class: String::from("MODEL"),
      is_alpha_used: Some(false),
      alpha_reference: Some(32),
      is_strict_sorting: false,
    }
  );
  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
}

#[test]
fn a_model_cut_out_tests_against_the_shader_constant_and_not_its_authored_reference() {
  // `models\model_aref` is authored at 128 and cuts out at 200: the authored value only decides that the deferred
  // path is taken at all, and the `_aref` pixel shader clips against `def_aref`.
  let descriptor: XraySurfaceDescriptor = describe(
    &library(
      "surface_model_aref",
      &[ShaderBlenderFixture::model("models\\model_aref")
        .with_alpha_channel(true)
        .with_alpha_reference(128)],
    ),
    "models\\model_aref",
  );

  assert_eq!(descriptor.draw, CUT_OUT);
  assert_eq!(
    descriptor.declaration,
    XraySurfaceDeclaration::Described {
      class: String::from("MODEL"),
      is_alpha_used: Some(true),
      alpha_reference: Some(128),
      is_strict_sorting: false,
    }
  );
}

#[test]
fn a_model_authored_below_the_forward_limit_is_blended_against_its_own_reference() {
  // `models\lightplanes`: a reference of 4 keeps almost every texel, which is translucency and not a cut-out.
  let descriptor: XraySurfaceDescriptor = describe(
    &library(
      "surface_model_translucent",
      &[ShaderBlenderFixture::model("models\\lightplanes")
        .with_alpha_channel(true)
        .with_alpha_reference(4)],
    ),
    "models\\lightplanes",
  );

  assert_eq!(descriptor.draw, XraySurfaceDraw::Blended { reference: 4 });
}

#[test]
fn strict_sorting_takes_a_model_forward_whatever_its_reference() {
  // `models\pautina`, the cobwebs: authored at 32, which would cut out, but asked to be sorted.
  let descriptor: XraySurfaceDescriptor = describe(
    &library(
      "surface_model_strict",
      &[ShaderBlenderFixture::model("models\\pautina")
        .with_alpha_channel(true)
        .with_alpha_reference(32)
        .with_strict_sorting(true)],
    ),
    "models\\pautina",
  );

  assert_eq!(descriptor.draw, XraySurfaceDraw::Blended { reference: 32 });
}

#[test]
fn an_environment_mapped_model_blends_against_nothing_or_stays_opaque() {
  // `models\weapons` and `models\window`: the class has a switch and no reference at all.
  let tree: FixtureTree = library(
    "surface_model_ebb",
    &[
      ShaderBlenderFixture::model_environment("models\\weapons"),
      ShaderBlenderFixture::model_environment("models\\window").with_alpha_channel(true),
    ],
  );

  assert_eq!(describe(&tree, "models\\weapons").draw, XraySurfaceDraw::Opaque);
  assert_eq!(
    describe(&tree, "models\\window").draw,
    XraySurfaceDraw::Blended { reference: 0 }
  );
  assert_eq!(
    describe(&tree, "models\\window").declaration,
    XraySurfaceDeclaration::Described {
      class: String::from("MODELEbB"),
      is_alpha_used: Some(true),
      alpha_reference: None,
      is_strict_sorting: false,
    }
  );
}

#[test]
fn a_level_aref_surface_cuts_out_until_its_switch_asks_for_blending() {
  // The one class whose switch means the opposite of `B_MODEL`'s: off is the cut-out, on leaves the deferred path.
  let tree: FixtureTree = library(
    "surface_level_aref",
    &[
      ShaderBlenderFixture::level_aref("def_shaders\\def_aref"),
      ShaderBlenderFixture::level_aref("def_shaders\\def_aref_blend").with_alpha_channel(true),
    ],
  );

  assert_eq!(describe(&tree, "def_shaders\\def_aref").draw, CUT_OUT);
  assert_eq!(
    describe(&tree, "def_shaders\\def_aref_blend").draw,
    XraySurfaceDraw::Blended { reference: 200 }
  );
}

#[test]
fn a_tree_surface_cuts_out_from_its_switch_alone_and_detail_always_does() {
  let tree: FixtureTree = library(
    "surface_scenery",
    &[
      ShaderBlenderFixture::tree("def_shaders\\def_trans"),
      ShaderBlenderFixture::tree("def_shaders\\def_trans_aref").with_alpha_channel(true),
      ShaderBlenderFixture::detail("def_shaders\\detail"),
    ],
  );

  assert_eq!(describe(&tree, "def_shaders\\def_trans").draw, XraySurfaceDraw::Opaque);
  assert_eq!(describe(&tree, "def_shaders\\def_trans_aref").draw, CUT_OUT);
  assert_eq!(describe(&tree, "def_shaders\\detail").draw, CUT_OUT);
}

#[test]
fn a_class_with_no_alpha_knobs_is_described_as_having_none() {
  let descriptor: XraySurfaceDescriptor = describe(
    &library(
      "surface_flat_class",
      &[ShaderBlenderFixture::of(
        ShaderBlenderClass::DEFAULT,
        "def_shaders\\def_vertex",
      )],
    ),
    "def_shaders\\def_vertex",
  );

  assert_eq!(
    descriptor.declaration,
    XraySurfaceDeclaration::Described {
      class: String::from("LM"),
      is_alpha_used: None,
      alpha_reference: None,
      is_strict_sorting: false,
    }
  );
  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
}

#[test]
fn a_class_whose_pass_is_not_modelled_says_so_rather_than_guessing() {
  // A particle class on a mesh surface: the engine compiles a pass this crate derives nothing from.
  let descriptor: XraySurfaceDescriptor = describe(
    &library(
      "surface_unmodelled",
      &[ShaderBlenderFixture::of(
        ShaderBlenderClass::PARTICLE,
        "particles\\smoke",
      )],
    ),
    "particles\\smoke",
  );

  assert_eq!(
    descriptor.declaration,
    XraySurfaceDeclaration::Unmodelled {
      class: String::from("PARTICLE")
    }
  );
  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
}

#[test]
fn a_switch_its_class_writes_and_a_file_omits_reads_as_off() {
  // A library written by something other than the SDK, missing a property the class defines.
  let descriptor: XraySurfaceDescriptor = describe(
    &library(
      "surface_missing_switch",
      &[ShaderBlenderFixture::model("models\\model")
        .with_alpha_channel(true)
        .without_property("Use alpha-channel")],
    ),
    "models\\model",
  );

  assert_eq!(
    descriptor.declaration,
    XraySurfaceDeclaration::Described {
      class: String::from("MODEL"),
      is_alpha_used: Some(false),
      alpha_reference: Some(32),
      is_strict_sorting: false,
    }
  );
  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
}

#[test]
fn the_library_is_read_once_and_answers_every_surface_of_a_model() {
  let tree: FixtureTree = library(
    "surface_shared",
    &[
      ShaderBlenderFixture::model("models\\model"),
      ShaderBlenderFixture::model("models\\model_fur")
        .with_alpha_channel(true)
        .with_alpha_reference(32),
    ],
  );

  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe<'_> = probe_over(&vfs, id);
  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(&probe);

  let opaque: XraySurfaceDescriptor = resolver.describe("models\\model");

  assert_eq!(opaque.draw, XraySurfaceDraw::Opaque);
  assert_eq!(resolver.describe("models\\model_fur").draw, CUT_OUT);
  // Every answer names the library it came from, which is what a panel says once for the whole model.
  assert_eq!(
    opaque.library.map(|it| it.get_logical_path().to_string()),
    Some(String::from(XraySurfaceResolver::SHADER_LIBRARY_LOGICAL_PATH))
  );
}
