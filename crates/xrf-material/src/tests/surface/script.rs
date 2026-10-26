//! What a shader name a renderer script answers comes to, which the engine prefers over the blender library.

use xrf_shaders::ShaderBlenderClass;
use xrf_shaders::fixtures::ShaderBlenderFixture;

use crate::fixtures::FixtureTree;
use crate::tests::surface::surface_search::{describe, describe_dressed, library};
use crate::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw, XraySurfaceSampler};

/// The pass `shaders/r3/effects_lightplanes.s` declares, which is what a lamp's light planes are drawn with.
const LIGHT_PLANES_SCRIPT: &str = r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("base_lplanes","base_lplanes")
      : fog   (false)
      : zb    (true,false)
      : blend (true,blend.srcalpha,blend.one)
      : aref  (true,0)
  shader:sampler ("s_base") :texture (t_base)
end
"#;

// The whole reason this path exists: `effects\lightplanes` is class `V` in every shipped `shaders.xr`, which the
// deferred renderers map to an opaque blender, and its script makes it additive. Reading the class alone drew a lamp's
// light planes as a black cross.
#[test]
fn a_shader_with_a_script_is_drawn_by_the_script_rather_than_by_its_class() {
  let tree: FixtureTree = library(
    "surface_script_wins",
    &[ShaderBlenderFixture::of(
      ShaderBlenderClass::VERT,
      "effects\\lightplanes",
    )],
  )
  .with_shader_script("effects\\lightplanes", LIGHT_PLANES_SCRIPT);

  let descriptor: XraySurfaceDescriptor = describe(&tree, "effects\\lightplanes");

  // `blend(true, blend.srcalpha, blend.one)`: the planes are added by their own alpha, not whole.
  assert_eq!(
    descriptor.draw,
    XraySurfaceDraw::Added {
      is_weighted: true,
      reference: 0
    }
  );
  assert!(matches!(
    descriptor.declaration,
    XraySurfaceDeclaration::Scripted { is_blended: true, .. }
  ));
}

// The engine only looks for a script of the undecorated name, so a class keeps answering for every shader without one.
#[test]
fn a_shader_without_a_script_is_still_drawn_by_its_class() {
  let tree: FixtureTree = library(
    "surface_script_absent",
    &[ShaderBlenderFixture::of(
      ShaderBlenderClass::VERT,
      "effects\\lightplanes",
    )],
  )
  .with_shader_script("something\\else", LIGHT_PLANES_SCRIPT);

  let descriptor: XraySurfaceDescriptor = describe(&tree, "effects\\lightplanes");

  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
  assert!(matches!(
    descriptor.declaration,
    XraySurfaceDeclaration::Described { .. }
  ));
}

// `selflight.s`: source times zero over destination times one keeps the destination, so nothing of the surface lands.
#[test]
fn a_script_that_keeps_the_destination_draws_nothing() {
  let tree: FixtureTree = library(
    "surface_script_invisible",
    &[ShaderBlenderFixture::of(ShaderBlenderClass::VERT, "selflight")],
  )
  .with_shader_script(
    "selflight",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("dumb","dumb") : zb (false,false) : blend (true,blend.zero,blend.one) : aref (false,0)
end
"#,
  );

  assert_eq!(describe(&tree, "selflight").draw, XraySurfaceDraw::Invisible);
}

// `effects_water.s` and `effects_waterstuden.s`: the programs say water, and the scripts' own textures come along.
#[test]
fn a_water_script_is_water_with_the_textures_each_of_its_passes_binds() {
  let soft: FixtureTree = library("surface_script_water_soft", &[]).with_shader_script(
    "effects\\water",
    r#"
local tex_base   = "water\\water_water"
local tex_nmap   = "water\\water_normal"
local tex_dist   = "water\\water_dudv"
local tex_env0   = "$user$sky0"

function normal (shader, t_base, t_second, t_detail)
  shader:begin ("water_soft","water_soft") : sorting (2, false) : blend (true,blend.srcalpha,blend.invsrcalpha)
        : zb (true,false) : distort (true) : fog (true)
  shader:sampler ("s_base")     :texture (tex_base)
  shader:sampler ("s_nmap")     :texture (tex_nmap)
  shader:sampler ("s_env0")     :texture (tex_env0) : clamp()
  shader:sampler ("s_position") :texture ("$user$position") : f_none ()
  shader:sampler ("s_leaves")   :texture ("water\\water_foam") : wrap() : f_anisotropic()
end

function l_special (shader, t_base, t_second, t_detail)
  shader:begin ("waterd_soft","waterd_soft") : blend (true,blend.srcalpha,blend.invsrcalpha)
  shader:sampler ("s_base")    :texture (tex_base)
  shader:sampler ("s_distort") :texture (tex_dist)
end
"#,
  );
  let whole: FixtureTree = library("surface_script_water_whole", &[]).with_shader_script(
    "effects\\waterstuden",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("water","water") : blend (false,blend.srcalpha,blend.invsrcalpha) : zb (true,false)
end
"#,
  );
  let described: XraySurfaceDescriptor = describe(&soft, "effects\\water");
  let named: Vec<(&str, &str, &str)> = described
    .samplers
    .iter()
    .map(|it| (it.element.as_str(), it.name.as_str(), it.texture.as_str()))
    .collect();

  assert_eq!(described.draw, XraySurfaceDraw::Water { is_soft: true });
  assert_eq!(
    named,
    vec![
      ("normal", "s_base", r"water\water_water"),
      ("normal", "s_nmap", r"water\water_normal"),
      ("normal", "s_leaves", r"water\water_foam"),
      ("l_special", "s_base", r"water\water_water"),
      ("l_special", "s_distort", r"water\water_dudv"),
    ]
  );
  assert_eq!(
    describe(&whole, "effects\\waterstuden").draw,
    XraySurfaceDraw::Water { is_soft: false }
  );
}

// Anomaly's `effects_waterstuden.s` draws by a program of its own, blended as vanilla's soft water is.
#[test]
fn a_water_program_of_another_name_is_water_too_and_soft_where_it_blends() {
  let anomaly: FixtureTree = library("surface_script_water_anomaly", &[]).with_shader_script(
    "effects\\waterstuden",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("water_studen","water_studen") : blend (true,blend.srcalpha,blend.invsrcalpha) : zb (true,false)
end
"#,
  );
  let distortion: FixtureTree = library("surface_script_water_distortion_only", &[]).with_shader_script(
    "effects\\waterdistort",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("waterd","waterd") : blend (true,blend.srcalpha,blend.invsrcalpha) : zb (true,false)
end
"#,
  );

  assert_eq!(
    describe(&anomaly, "effects\\waterstuden").draw,
    XraySurfaceDraw::Water { is_soft: true }
  );
  assert!(matches!(
    describe(&anomaly, "effects\\waterstuden").declaration,
    XraySurfaceDeclaration::Scripted { program, .. } if program == "water_studen"
  ));
  assert_eq!(
    describe(&distortion, "effects\\waterdistort").draw,
    XraySurfaceDraw::Blended { reference: 0 }
  );
}

// `effects_wallmarkblend.s` and `effects_wallmarkmult.s`, the two a level lays its marks with.
#[test]
fn a_wall_mark_script_composites_the_way_it_says() {
  let blended: FixtureTree = library("surface_script_wmark_blend", &[]).with_shader_script(
    "effects\\wallmarkblend",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("wmark","simple") : blend (true,blend.srcalpha,blend.invsrcalpha) : aref (true,0) : zb (true,false) : wmark (true)
end
"#,
  );

  assert_eq!(
    describe(&blended, "effects\\wallmarkblend").draw,
    XraySurfaceDraw::Blended { reference: 0 }
  );

  let multiplied: FixtureTree = library("surface_script_wmark_mult", &[]).with_shader_script(
    "effects\\wallmarkmult",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("wmark","simple") : blend (true,blend.destcolor,blend.srccolor) : zb (true,false) : wmark (true)
end
"#,
  );

  assert_eq!(
    describe(&multiplied, "effects\\wallmarkmult").draw,
    XraySurfaceDraw::Multiplied { is_doubled: true }
  );
}

// `aref(true, 0)` grants no discard. DX10 and DX11 have no alpha test state: the reference is bound as a shader
// constant and a pass cuts only where its pixel shader calls `clip`, which the `*_aref_*` deferred shaders do and
// `simple.ps` - the shader every wall mark pass names - does not. A wall mark multiplies anyway, and `MUL_2X` never
// reads alpha.
#[test]
fn a_wall_mark_asks_for_a_test_and_is_granted_none() {
  let tree: FixtureTree = library("surface_script_wmark_aref", &[]).with_shader_script(
    "effects\\wallmarkmult",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("wmark","simple") : blend (true,blend.destcolor,blend.srccolor) : aref (true,0) : zb (true,false) : wmark (true)
end
"#,
  );

  let descriptor: XraySurfaceDescriptor = describe(&tree, "effects\\wallmarkmult");

  let XraySurfaceDeclaration::Scripted { is_alpha_tested, .. } = descriptor.declaration else {
    panic!("Expect the wall mark to be read from its script");
  };

  // Reported, because the pass does ask for one and a reader of the panel should see that it did.
  assert!(is_alpha_tested);
  // Drawn without one, because the renderer the game runs grants it nothing.
  assert_eq!(descriptor.draw, XraySurfaceDraw::Multiplied { is_doubled: true });
}

// The same for a pass that writes rather than composites: a reference of zero leaves nothing to cut.
#[test]
fn an_opaque_pass_tested_against_zero_discards_nothing() {
  let tree: FixtureTree = library("surface_script_opaque_aref", &[]).with_shader_script(
    "details\\blend",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("detail","detail") : aref (true,0) : zb (true,true)
end
"#,
  );

  assert_eq!(describe(&tree, "details\\blend").draw, XraySurfaceDraw::Opaque);
}

// `_lua_HasShader` takes a script declaring only `l_special` as the shader: it distorts and draws nothing of its own.
#[test]
fn a_script_with_only_a_distortion_pass_is_the_shader_and_draws_nothing() {
  let tree: FixtureTree = library(
    "surface_script_distortion_only",
    &[ShaderBlenderFixture::particle("particles\\xdistort", 1)],
  )
  .with_shader_script(
    "particles\\xdistort",
    r#"
function l_special (shader, t_base, t_second, t_detail)
  shader:begin ("particle","particle_distort") : blend (true, blend.srcalpha, blend.invsrcalpha) : zb (true, false)
    : distort (true)
  shader:sampler ("s_distort") :texture (t_base)
end
"#,
  );
  let descriptor: XraySurfaceDescriptor = describe_dressed(&tree, "particles\\xdistort", &["pfx\\pfx_dist_flame"]);

  assert!(matches!(
    descriptor.declaration,
    XraySurfaceDeclaration::Scripted { ref function, .. } if function == "l_special"
  ));
  assert_eq!(descriptor.draw, XraySurfaceDraw::Invisible);
  assert!(descriptor.is_distorting);
  assert_eq!(descriptor.samplers[0].texture, "pfx\\pfx_dist_flame");
}

// `models_selflight.s`: a self-lit base pass, whose special pass fills the light it gathers rather than distorting.
#[test]
fn a_self_lit_script_fills_its_light_and_distorts_nothing() {
  let tree: FixtureTree = library("surface_script_self_lit", &[]).with_shader_script(
    "models\\selflight",
    r#"
function normal (shader, t_base, t_second, t_detail)
  shader:begin ("deffer_model_flat","deffer_base_flat") : fog (false) : emissive (true)
end

function l_special (shader, t_base, t_second, t_detail)
  shader:begin ("shadow_direct_model","accum_emissive") : zb (true,false) : fog (false) : emissive (true)
end
"#,
  );
  let descriptor: XraySurfaceDescriptor = describe_dressed(&tree, "models\\selflight", &["prop\\prop_lampa_g"]);

  assert_eq!(descriptor.draw, XraySurfaceDraw::Opaque);
  assert!(descriptor.is_emissive);
  assert!(!descriptor.is_distorting);
  // Without an `l_point` it has no shadow element, so the lamp never shadows its own light.
  assert!(descriptor.is_shadowless);
}

// `_lua_Compile` hands a function its first two textures after the compiler, whatever the script names them.
#[test]
fn a_script_parameter_binds_the_surfaces_texture_at_its_position() {
  let tree: FixtureTree = library("surface_script_parameters", &[]).with_shader_script(
    "particles\\xadd",
    r#"
function normal (shader, first, second, detail)
  shader:begin ("particle","particle") : blend (true, blend.one, blend.one) : zb (true, false)
  shader:sampler ("s_base") :texture (first)
end

function l_special (shader, first, second, detail)
  shader:begin ("particle","particle_distort") : blend (true, blend.srcalpha, blend.invsrcalpha) : zb (true, false)
    : distort (true)
  shader:sampler ("s_distort") :texture (second)
  shader:sampler ("s_detail") :texture (detail)
end
"#,
  );
  let descriptor: XraySurfaceDescriptor =
    describe_dressed(&tree, "particles\\xadd", &["pfx\\pfx_flame", "pfx\\pfx_distortion"]);
  let samplers: Vec<(&str, &str, &str)> = descriptor
    .samplers
    .iter()
    .map(|it| (it.element.as_str(), it.name.as_str(), it.texture.as_str()))
    .collect();

  assert_eq!(
    descriptor.draw,
    XraySurfaceDraw::Added {
      is_weighted: false,
      reference: 0
    }
  );
  assert!(descriptor.is_distorting);
  assert_eq!(
    samplers,
    vec![
      ("normal", "s_base", "pfx\\pfx_flame"),
      ("l_special", "s_distort", "pfx\\pfx_distortion"),
    ]
  );
  assert_eq!(
    descriptor.find_sampler(XraySurfaceSampler::DISTORTION_ELEMENT, "s_distort"),
    Some("pfx\\pfx_distortion")
  );
  assert_eq!(
    descriptor.find_sampler(XraySurfaceSampler::BASE_ELEMENT, "s_distort"),
    None
  );
}
