use xrf_material::{
  XrayBumpMode, XrayMaterialBump, XrayMaterialBumpInput, XraySurfaceDeclaration, XraySurfaceDescriptor,
  XraySurfaceDetail, XraySurfaceDraw,
};
use xrf_vfs::XrayResolution;
use xrf_visual::SectorSurface;

use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_build::StaticSurfaceBuilder;
use crate::scene::static_scene::static_surface_plan::StaticSurfacePlan;
use crate::scene::texture::texture_role::TextureRole;

/// A level surface of shader table entry 1, dressed by `texture`.
fn create_surface(texture: &str) -> SectorSurface {
  SectorSurface {
    shader_id: 1,
    shader_name: None,
    texture_name: Some(texture.to_string()),
    hemi: None,
  }
}

/// A described blender of `class`, drawn as `draw`, named `shader`.
fn create_descriptor(shader: &str, class: &str, draw: XraySurfaceDraw) -> XraySurfaceDescriptor {
  XraySurfaceDescriptor {
    shader: Some(shader.to_string()),
    draw,
    ..XraySurfaceDescriptor::opaque(
      None,
      XraySurfaceDeclaration::Described {
        class: class.to_string(),
        is_alpha_used: None,
        alpha_reference: None,
        is_strict_sorting: false,
      },
    )
  }
}

/// The bump pair a base texture's descriptor declares, in `mode`.
fn create_bump(mode: XrayBumpMode) -> XrayMaterialBump {
  let input = |reference: &str| XrayMaterialBumpInput {
    reference: reference.to_string(),
    resolution: XrayResolution::NoScope,
  };

  XrayMaterialBump {
    mode,
    virtual_height: 0.0,
    bump: input("crete\\crete_bump"),
    companion: input("crete\\crete_bump#"),
  }
}

/// Plans each surface, answering its flags and class.
fn build(cases: &[(SectorSurface, XraySurfaceDescriptor)]) -> Vec<(u32, StaticClass)> {
  cases
    .iter()
    .map(|(surface, descriptor)| {
      let plan: StaticSurfacePlan<'_> = StaticSurfaceBuilder::plan(surface, Some(descriptor)).unwrap();

      (plan.flags, plan.class)
    })
    .collect()
}

#[test]
fn takes_a_blended_glass_shader_for_glass_only_on_a_panes_base() {
  let glass = || {
    create_descriptor(
      "def_shaders\\def_trans",
      "LM_AREF",
      XraySurfaceDraw::Blended { reference: 0 },
    )
  };
  let built: Vec<(u32, StaticClass)> = build(&[
    (create_surface("glas\\glas_window_01"), glass()),
    (create_surface("wind\\wind_frame"), glass()),
    (create_surface("fence\\fence_wire"), glass()),
  ]);

  assert!(built[0].0 & StaticSurface::IS_GLASS != 0, "a base in the glass folder");
  assert!(built[1].0 & StaticSurface::IS_GLASS != 0, "a base in the window folder");
  assert!(
    built[2].0 & StaticSurface::IS_GLASS == 0,
    "wire shares the shader but is no pane"
  );
  assert!(built.iter().all(|(_, class)| *class == StaticClass::Composited));
}

#[test]
fn marches_a_steep_bump_by_its_class_and_a_plain_one_only_on_an_opaque_class() {
  let bumped = |class: &str, mode: XrayBumpMode| XraySurfaceDescriptor {
    bump: Some(create_bump(mode)),
    ..create_descriptor("default", class, XraySurfaceDraw::Opaque)
  };
  let built: Vec<(u32, StaticClass)> = build(&[
    (create_surface("crete\\crete"), bumped("LM", XrayBumpMode::Parallax)),
    (create_surface("crete\\crete"), bumped("LM", XrayBumpMode::Use)),
    (create_surface("crete\\crete"), bumped("LM_AREF", XrayBumpMode::Use)),
    (create_surface("crete\\crete"), bumped("MODEL", XrayBumpMode::Parallax)),
  ]);
  let marks = |flags: u32| {
    (
      flags & StaticSurface::IS_STEEP != 0,
      flags & StaticSurface::IS_RELIEF_BUMPED != 0,
    )
  };

  assert!(
    built.iter().all(|(flags, _)| flags & StaticSurface::HAS_BUMP != 0),
    "a bump pair binds"
  );
  assert_eq!(
    marks(built[0].0),
    (true, false),
    "parallax on a level class is the engines' steep march"
  );
  assert_eq!(
    marks(built[1].0),
    (false, true),
    "a plain bump on an opaque level class is relief-bumped"
  );
  assert_eq!(
    marks(built[2].0),
    (false, false),
    "an alpha-tested class takes no relief"
  );
  assert_eq!(
    marks(built[3].0),
    (false, false),
    "no engine compiles steep parallax for a model class"
  );
}

#[test]
fn tests_a_composited_surface_by_its_own_reference_and_lays_a_wall_mark_as_one() {
  let wallmark: XraySurfaceDescriptor = XraySurfaceDescriptor {
    shader: Some("effects\\wallmark".to_string()),
    draw: XraySurfaceDraw::Blended { reference: 0 },
    ..XraySurfaceDescriptor::opaque(
      None,
      XraySurfaceDeclaration::Scripted {
        script: "gamedata\\shaders\\r1\\effects_wallmark.s".to_string(),
        function: "normal".to_string(),
        program: "wmark".to_string(),
        is_blended: true,
        is_alpha_tested: false,
        is_depth_written: false,
        is_wallmark: true,
      },
    )
  };
  let built: Vec<(u32, StaticClass)> = build(&[
    (
      create_surface("fx\\fx_smoke"),
      create_descriptor("models\\model", "MODEL", XraySurfaceDraw::Blended { reference: 0 }),
    ),
    (
      create_surface("fx\\fx_smoke"),
      create_descriptor("models\\model", "MODEL", XraySurfaceDraw::Blended { reference: 16 }),
    ),
    (
      create_surface("trees\\trees_leaves"),
      create_descriptor("default", "LM_AREF", XraySurfaceDraw::AlphaTested { reference: 200 }),
    ),
    (create_surface("wm\\wm_blood"), wallmark),
  ]);

  assert!(
    built[0].0 & StaticSurface::IS_CUT_OUT == 0,
    "a blend with no reference blends every texel"
  );
  assert!(
    built[1].0 & StaticSurface::IS_CUT_OUT != 0,
    "a blend with a reference tests it"
  );
  assert_eq!(built[2], (built[2].0 | StaticSurface::IS_CUT_OUT, StaticClass::CutOut));
  assert_eq!(built[3].1, StaticClass::Wallmark);
}

#[test]
fn lights_a_blended_level_surface_by_the_lightmap_its_entry_names() {
  let lit = |draw: XraySurfaceDraw| XraySurfaceDescriptor {
    textures: vec![String::from("fx\\fx_smoke"), String::from("lmap#3_1")],
    detail: Some(XraySurfaceDetail {
      reference: String::from("detail\\detail_grnd_grass"),
      scale: 4.0,
      bump: None,
    }),
    ..create_descriptor("default", "LM_AREF", draw)
  };
  let surface: SectorSurface = create_surface("fx\\fx_smoke");
  let blended: XraySurfaceDescriptor = lit(XraySurfaceDraw::Blended { reference: 0 });
  let added: XraySurfaceDescriptor = lit(XraySurfaceDraw::Added {
    reference: 0,
    is_weighted: false,
  });
  let blended: StaticSurfacePlan<'_> = StaticSurfaceBuilder::plan(&surface, Some(&blended)).unwrap();
  let added: StaticSurfacePlan<'_> = StaticSurfaceBuilder::plan(&surface, Some(&added)).unwrap();

  assert_eq!(blended.lightmap, Some("lmap#3_1"));
  assert!(blended.flags & StaticSurface::HAS_LIGHTMAP != 0);
  assert_eq!(
    blended.slots[StaticSurface::LIGHTMAP],
    None,
    "the lightmap takes the detail's slot, so no detail is asked for"
  );
  assert_eq!(added.lightmap, None, "an added surface is lit by nothing of its own");
  assert!(added.flags & StaticSurface::HAS_LIGHTMAP == 0);
}

#[test]
fn names_each_texture_at_its_slot_with_the_role_it_is_asked_for_as() {
  let surface: SectorSurface = SectorSurface {
    hemi: Some(String::from("lmap#3_2")),
    ..create_surface("crete\\crete")
  };
  let descriptor: XraySurfaceDescriptor = XraySurfaceDescriptor {
    bump: Some(create_bump(XrayBumpMode::Use)),
    ..create_descriptor("default", "LM", XraySurfaceDraw::Opaque)
  };
  let plan: StaticSurfacePlan<'_> = StaticSurfaceBuilder::plan(&surface, Some(&descriptor)).unwrap();

  assert_eq!(
    plan.slots[StaticSurface::BASE],
    Some(("crete\\crete", TextureRole::Base))
  );
  assert_eq!(plan.slots[StaticSurface::DETAIL], None);
  assert_eq!(
    plan.slots[StaticSurface::BUMP],
    Some(("crete\\crete_bump", TextureRole::Bump))
  );
  assert_eq!(
    plan.slots[StaticSurface::BUMP_COMPANION],
    Some(("crete\\crete_bump#", TextureRole::BumpCompanion))
  );
  assert_eq!(plan.slots[StaticSurface::HEMI], Some(("lmap#3_2", TextureRole::Hemi)));
}
