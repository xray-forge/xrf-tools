use std::sync::Arc;

use xrf_material::{
  XrayBumpMode, XrayMaterialBump, XrayMaterialBumpInput, XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw,
};
use xrf_test_utils::gpu::skip_without_gpu;
use xrf_vfs::XrayResolution;
use xrf_visual::SectorSurface;

use crate::context::gpu_context::GpuContext;
use crate::contract::render_backend::RenderBackend;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_build::StaticSurfaceBuilder;
use crate::scene::texture::texture_cache::TextureCache;
use crate::tests::test_bundle::checkout_dxc;
use crate::tests::test_source::EmptySource;
use crate::tests::test_workers::create_workers;

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

/// Builds each surface on a headless device, answering its flags and class; none where no GPU is there.
fn build(cases: &[(SectorSurface, XraySurfaceDescriptor)]) -> Option<Vec<(u32, StaticClass)>> {
  let Ok(context) = GpuContext::create_headless(RenderBackend::D3d12, checkout_dxc().as_deref())
    .or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan, None))
  else {
    skip_without_gpu("no GPU to hold textures");

    return None;
  };
  let source: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let mut textures: TextureCache = TextureCache::new(&context.device, &context.queue, &create_workers());

  Some(
    cases
      .iter()
      .map(|(surface, descriptor)| {
        let (built, class): (StaticSurface, StaticClass) =
          StaticSurfaceBuilder::build(surface, Some(descriptor), &mut textures, &source).unwrap();

        (built.flags, class)
      })
      .collect(),
  )
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
  let Some(built) = build(&[
    (create_surface("glas\\glas_window_01"), glass()),
    (create_surface("wind\\wind_frame"), glass()),
    (create_surface("fence\\fence_wire"), glass()),
  ]) else {
    return;
  };

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
  let Some(built) = build(&[
    (create_surface("crete\\crete"), bumped("LM", XrayBumpMode::Parallax)),
    (create_surface("crete\\crete"), bumped("LM", XrayBumpMode::Use)),
    (create_surface("crete\\crete"), bumped("LM_AREF", XrayBumpMode::Use)),
    (create_surface("crete\\crete"), bumped("MODEL", XrayBumpMode::Parallax)),
  ]) else {
    return;
  };
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
  let Some(built) = build(&[
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
  ]) else {
    return;
  };

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
