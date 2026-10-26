//! The detail texture a surface modulates its diffuse with, by the class it compiles as and the pass it takes.

use xrf_shaders::fixtures::ShaderBlenderFixture;
use xrf_shaders::{ShaderBlender, ShaderBlenderClass};
use xrf_thm::ThmTextureFlag;
use xrf_vfs::{XrayMountId, XraySearch, XrayVfs};

use crate::fixtures::{FixtureTree, ThmFixture};
use crate::tests::material_search::{BASE, BUMP, COMPANION, search_over, used_bump};
use crate::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};

const DETAIL: &str = "detail\\detail_grnd_grass";
const DECLARED: &str = "detail\\detail_grnd_earth";

/// A tree whose library defines `blenders` and whose base texture declares a located pair and a diffuse detail.
fn detailed_tree(case: &str, blenders: &[ShaderBlenderFixture]) -> FixtureTree {
  FixtureTree::new(case)
    .with_shader_library(blenders)
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(
      BASE,
      &used_bump().with_detail(DETAIL, 4.0, &[ThmTextureFlag::DiffuseDetail]),
    )
}

/// Describes `shader` dressed with the base texture alone.
fn describe(tree: &FixtureTree, shader: &str) -> XraySurfaceDescriptor {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let search: XraySearch = search_over(vfs, id);

  XraySurfaceResolver::open(&search, XrayTextureScope::shared()).describe(shader, &[BASE.to_owned()])
}

/// The detail a surface binds, by name and tiling.
fn detail_of(descriptor: &XraySurfaceDescriptor) -> Option<(&str, f32)> {
  descriptor
    .detail
    .as_ref()
    .map(|detail| (detail.reference.as_str(), detail.scale))
}

/// The terrain class `B_LmBmmD` naming `detail` under its own marker.
fn lightmapped_terrain(name: &str, detail: &str) -> ShaderBlenderFixture {
  let mut fixture: ShaderBlenderFixture = ShaderBlenderFixture::level_detailed(name, detail);

  fixture.blender.class = ShaderBlenderClass::LM_BMM_D;
  fixture
}

// `canBeDetailed` answers `TRUE` for every class `uber_deffer` compiles with a detail slot: a model is detailed as a
// level's own geometry is.
#[test]
fn every_detailed_class_binds_the_detail_its_base_associates() {
  let tree: FixtureTree = detailed_tree(
    "surface_detail_classes",
    &[
      ShaderBlenderFixture::model("models\\model"),
      ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "default"),
      ShaderBlenderFixture::of(ShaderBlenderClass::VERT_AREF, "def_shaders\\def_vertex_aref"),
      ShaderBlenderFixture::tree("trees\\trees"),
    ],
  );

  for shader in [
    "models\\model",
    "default",
    "def_shaders\\def_vertex_aref",
    "trees\\trees",
  ] {
    assert_eq!(detail_of(&describe(&tree, shader)), Some((DETAIL, 4.0)), "{shader}");
  }
}

#[test]
fn a_class_that_cannot_be_detailed_binds_none() {
  let tree: FixtureTree = detailed_tree(
    "surface_detail_undetailed",
    &[ShaderBlenderFixture::model_environment("models\\weapons")],
  );

  assert_eq!(describe(&tree, "models\\weapons").detail, None);
}

// The forward passes a blended surface takes, `model_def_lq` and its kin, sample the base alone: neither the pair nor
// the detail its descriptor names reaches the screen.
#[test]
fn a_blended_surface_binds_neither_its_pair_nor_its_detail() {
  let tree: FixtureTree = detailed_tree(
    "surface_detail_forward",
    &[
      ShaderBlenderFixture::model("models\\pautina")
        .with_alpha_channel(true)
        .with_strict_sorting(true),
      ShaderBlenderFixture::level_aref("def_shaders\\def_aref_blend").with_alpha_channel(true),
      ShaderBlenderFixture::level_aref("def_shaders\\def_aref"),
    ],
  );

  for shader in ["models\\pautina", "def_shaders\\def_aref_blend"] {
    let descriptor: XraySurfaceDescriptor = describe(&tree, shader);

    assert_eq!(descriptor.bump, None, "{shader}");
    assert_eq!(descriptor.detail, None, "{shader}");
  }

  // The same class cut out of the g-buffer binds both.
  let deferred: XraySurfaceDescriptor = describe(&tree, "def_shaders\\def_aref");

  assert!(deferred.bump.is_some());
  assert_eq!(detail_of(&deferred), Some((DETAIL, 4.0)));
}

// `CBlender_BmmD` hands `uber_deffer` its own `oT2_Name`, tiled as the base texture's descriptor says.
#[test]
fn a_terrain_binds_the_detail_it_names_in_place_of_the_association() {
  let tree: FixtureTree = detailed_tree(
    "surface_detail_declared",
    &[
      ShaderBlenderFixture::level_detailed("levels\\earth", DECLARED),
      lightmapped_terrain("levels\\earth_lm", DECLARED),
    ],
  );

  assert_eq!(detail_of(&describe(&tree, "levels\\earth")), Some((DECLARED, 4.0)));
  assert_eq!(detail_of(&describe(&tree, "levels\\earth_lm")), Some((DECLARED, 4.0)));
}

// `oT2_Name[0] ? oT2_Name : 0`: an empty name falls back to the association, and `$null` is a name that binds nothing.
#[test]
fn a_terrain_naming_no_detail_falls_back_and_one_naming_null_binds_none() {
  let tree: FixtureTree = detailed_tree(
    "surface_detail_fallback",
    &[
      lightmapped_terrain("levels\\empty", ""),
      lightmapped_terrain("levels\\null", ShaderBlender::NULL_TEXTURE),
    ],
  );

  assert_eq!(detail_of(&describe(&tree, "levels\\empty")), Some((DETAIL, 4.0)));
  assert_eq!(describe(&tree, "levels\\null").detail, None);
}

/// An `r3` script in the DX10 dialect with a detailed variant of its base pass, as `def_shaders_def_vertex.s` ships.
const DETAILED_SCRIPT: &str = r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("deffer_base_flat","deffer_base_flat")
  shader:dx10texture ("s_base", t_base)
end

function normal_hq (shader, t_base, t_second, t_detail)
  shader:begin ("deffer_base_flat_d","deffer_base_flat_d")
  shader:dx10texture ("s_base", t_base)
  shader:dx10texture ("s_detail", "detail\\detail_grnd_earth")
end
"#;

// `_lua_Create` compiles the high quality element from `normal_hq` when `C.bDetail`, which the base texture's
// descriptor decides.
#[test]
fn a_script_compiles_its_detailed_pass_for_a_base_texture_that_associates_a_detail() {
  let detailed: FixtureTree =
    detailed_tree("surface_detail_script", &[]).with_shader_script("def_shaders\\def_vertex", DETAILED_SCRIPT);
  let flat: FixtureTree = FixtureTree::new("surface_detail_script_flat")
    .with_texture(BASE)
    .with_descriptor(BASE, &ThmFixture::image())
    .with_shader_script("def_shaders\\def_vertex", DETAILED_SCRIPT);
  let program = |descriptor: XraySurfaceDescriptor| match descriptor.declaration {
    XraySurfaceDeclaration::Scripted { program, .. } => program,
    declaration => panic!("Expect a scripted shader, got {declaration:?}"),
  };
  let samplers: Vec<(String, String)> = describe(&detailed, "def_shaders\\def_vertex")
    .samplers
    .into_iter()
    .map(|it| (it.name, it.texture))
    .collect();

  assert_eq!(
    program(describe(&detailed, "def_shaders\\def_vertex")),
    "deffer_base_flat_d"
  );
  assert_eq!(program(describe(&flat, "def_shaders\\def_vertex")), "deffer_base_flat");
  assert_eq!(
    samplers,
    vec![
      (String::from("s_base"), String::from(BASE)),
      (String::from("s_detail"), String::from(DECLARED)),
    ]
  );
}
