use std::sync::Arc;

use glam::{UVec4, Vec3};
use xrf_material::{
  XrayBumpMode, XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw, XraySurfaceSampler, XraySurfaceTerrain,
};
use xrf_visual::SectorSurface;

use crate::host::render_asset_source::RenderAssetSource;
use crate::lighting::material_slice::material_slice;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_plan::StaticSurfacePlan;
use crate::scene::static_scene::static_terrain_slots::StaticTerrainSlots;
use crate::scene::static_scene::static_water_surface::StaticWaterSurfaceBuilder;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;

/// What a level's lightmaps are named by, `lmap#<n>_1` and `lmap#<n>_2`.
const LIGHTMAP_PREFIX: &str = "lmap#";

/// What `details_lod.s` appends to the atlas for its `s_hemi`.
const IMPOSTOR_COMPANION_SUFFIX: &str = "_nm";

/// The texture descriptor's default lighting model: Blinn, at full weight.
const DEFAULT_MATERIAL: f32 = 1.0;

/// The environment-mapped model class's tag.
const ENVIRONMENT_MAPPED_CLASS: &str = "MODELEbB";
/// The level's flat and alpha-tested classes, the only ones the engines compile steep parallax for: `B_DEFAULT`,
/// `B_DEFAULT_AREF`, `B_VERT` and `B_VERT_AREF` (`r2_blenders.cpp`).
const STEEP_CLASSES: [&str; 4] = ["LM", "LM_AREF", "V", "V_AREF"];

/// The level's opaque classes, whose bumped surfaces the enhanced parallax marches where the engines leave them flat.
const RELIEF_CLASSES: [&str; 2] = ["LM", "V"];

/// The engines' glass shaders: the level's blended panes (`def_trans`, `def_trans_v`, `def_refl_trans`) and the models'
/// windows (`models\window`, `models\xwindows`), as their shader table names them. The level's blended fences and
/// wire share them, so a pane is told apart by its base too (`GLASS_TEXTURES`).
const GLASS_SHADERS: [&str; 5] = [
  "def_shaders\\def_trans",
  "def_shaders\\def_trans_v",
  "def_shaders\\def_refl_trans",
  "models\\window",
  "models\\xwindows",
];

/// What a pane's base is named by, lower case: in the glass or window folders, or glass by name.
const GLASS_TEXTURES: [&str; 3] = ["glas\\", "wind\\", "glass"];

/// Turns of the golden angle between consecutive shader ids, which spreads their colours rather than grouping them.
const HUE_STEP: f32 = 137.508;

/// Builds the surfaces the static draws wear from shader table entries.
pub struct StaticSurfaceBuilder;

impl StaticSurfaceBuilder {
  /// The material whose slice `lod.ps` writes: the default lighting model.
  pub const IMPOSTOR_MATERIAL: f32 = 0.0;

  /// A shader table entry as the static draws wear it, asking for its textures; `None` for an invisible one, which no
  /// static draw draws. A composited wall mark is laid into the G-buffer's albedo rather than over the lit frame.
  pub fn build(
    surface: &SectorSurface,
    descriptor: Option<&XraySurfaceDescriptor>,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) -> Option<(StaticSurface, StaticClass)> {
    if let Some(descriptor) = descriptor
      && let XraySurfaceDraw::Water { is_soft } = descriptor.draw
    {
      let color: Vec3 = Self::to_surface_color(surface.shader_id);

      return Some((
        StaticWaterSurfaceBuilder::build(surface, descriptor, is_soft, textures, source, color),
        StaticClass::Water,
      ));
    }

    let plan: StaticSurfacePlan<'_> = Self::plan(surface, descriptor)?;
    let mut texture_slots: [u32; 8] = [0; 8];

    for (slot, named) in texture_slots.iter_mut().zip(plan.slots) {
      *slot = named.map_or(0, |(reference, role)| textures.request(reference, role, source));
    }

    let terrain: StaticTerrainSlots = plan.terrain.map_or_else(StaticTerrainSlots::default, |terrain| {
      Self::request_terrain(terrain, textures, source)
    });

    texture_slots[StaticSurface::ENVIRONMENT] = plan
      .environment
      .map_or(0, |reference| textures.request_environment(reference));

    if let Some(lightmap) = plan.lightmap {
      texture_slots[StaticSurface::LIGHTMAP] = textures.request(lightmap, TextureRole::Hemi, source);
    }

    let distortion: u32 = plan
      .distortion
      .map_or(0, |reference| textures.request(reference, TextureRole::Detail, source));

    Some((
      StaticSurface {
        tiling: 1.0,
        detail_scale: plan.detail_scale,
        alpha_reference: plan.alpha_reference as f32 / 255.0,
        slice: material_slice(plan.material),
        color: Self::to_surface_color(surface.shader_id),
        flags: plan.flags,
        textures: texture_slots,
        terrain,
        distortion,
        _pad: [0; 3],
      },
      plan.class,
    ))
  }

  /// What a shader table entry decides of a surface before its textures are asked for: its class, alpha reference and
  /// flags, and the textures it names; `None` for water, which `StaticWaterSurfaceBuilder` builds, and for an
  /// invisible one.
  pub fn plan<'a>(
    surface: &'a SectorSurface,
    descriptor: Option<&'a XraySurfaceDescriptor>,
  ) -> Option<StaticSurfacePlan<'a>> {
    let (class, reference, blend): (StaticClass, u8, u32) = match descriptor.map(|it| it.draw) {
      None | Some(XraySurfaceDraw::Opaque) => (StaticClass::Opaque, XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE, 0),
      Some(XraySurfaceDraw::AlphaTested { reference }) => (StaticClass::CutOut, reference, 0),
      Some(XraySurfaceDraw::Blended { reference }) => (StaticClass::Composited, reference, 0),
      Some(XraySurfaceDraw::Added { reference, is_weighted }) => (
        StaticClass::Composited,
        reference,
        StaticSurface::IS_ADDED | if is_weighted { StaticSurface::IS_WEIGHTED } else { 0 },
      ),
      Some(XraySurfaceDraw::Multiplied { is_doubled }) => (
        StaticClass::Composited,
        0,
        StaticSurface::IS_MULTIPLIED | if is_doubled { StaticSurface::IS_DOUBLED } else { 0 },
      ),
      Some(XraySurfaceDraw::Water { .. } | XraySurfaceDraw::Invisible) => return None,
    };

    let class: StaticClass = match class {
      StaticClass::Composited if descriptor.is_some_and(Self::is_wallmark) => StaticClass::Wallmark,
      StaticClass::Composited if descriptor.is_some_and(Self::is_strictly_sorted) => StaticClass::Sorted,
      class => class,
    };

    let detail = descriptor
      .and_then(|it| it.detail.as_ref())
      .filter(|detail| detail.scale.is_finite());
    // A terrain lays its own four details and bumps, at its detail's tiling; its single detail and bump pair are the
    // low-quality path's, which it does not draw.
    let is_terrain: bool = descriptor.is_some_and(|it| it.terrain.is_some());
    let bump = descriptor.and_then(|it| it.bump.as_ref()).filter(|_| !is_terrain);
    let detail_bump = detail.and_then(|detail| detail.bump.as_ref()).filter(|_| !is_terrain);
    let mut slots: [Option<(&'a str, TextureRole)>; 7] = [None; 7];
    let mut name = |slot: usize, reference: Option<&'a str>, role: TextureRole| {
      slots[slot] = reference.filter(|it| !it.is_empty()).map(|it| (it, role));
    };

    name(StaticSurface::BASE, surface.texture_name.as_deref(), TextureRole::Base);
    name(
      StaticSurface::DETAIL,
      detail.filter(|_| !is_terrain).map(|it| it.reference.as_str()),
      TextureRole::Detail,
    );
    name(
      StaticSurface::BUMP,
      bump.map(|it| it.bump.reference.as_str()),
      TextureRole::Bump,
    );
    name(
      StaticSurface::BUMP_COMPANION,
      bump.map(|it| it.companion.reference.as_str()),
      TextureRole::BumpCompanion,
    );
    name(
      StaticSurface::DETAIL_BUMP,
      detail_bump.map(|it| it.bump.reference.as_str()),
      TextureRole::Bump,
    );
    name(
      StaticSurface::DETAIL_BUMP_COMPANION,
      detail_bump.map(|it| it.companion.reference.as_str()),
      TextureRole::BumpCompanion,
    );
    name(StaticSurface::HEMI, surface.hemi.as_deref(), TextureRole::Hemi);

    let mut flags: u32 = Self::to_flags(descriptor, &slots.map(|it| it.is_some()), (class, reference, blend));

    let terrain: Option<&'a XraySurfaceTerrain> = descriptor
      .and_then(|it| it.terrain.as_ref())
      .filter(|_| flags & StaticSurface::HAS_BASE != 0);

    if terrain.is_some() {
      flags |= StaticSurface::IS_TERRAIN;
    }

    let environment: Option<&'a str> = descriptor
      .and_then(|it| it.environment.as_deref())
      .filter(|_| flags & StaticSurface::IS_ENVIRONMENT_MAPPED != 0);

    // A blended level surface is drawn forward as `lmapE` draws it, by its own lightmap, the shader entry's second
    // texture where it names one.
    let lightmap: Option<&'a str> = descriptor
      .filter(|_| class.is_composited() && blend == 0)
      .and_then(|it| it.textures.get(1))
      .filter(|reference| reference.starts_with(LIGHTMAP_PREFIX))
      .map(String::as_str);

    if lightmap.is_some() {
      flags = (flags & !StaticSurface::HAS_DETAIL) | StaticSurface::HAS_LIGHTMAP;
      // The lightmap takes the detail's slot, so the detail is never asked for, to be read and freed unused.
      slots[StaticSurface::LIGHTMAP] = None;
    }

    // A composited surface whose special pass distorts (`r__dsgraph_build` files it under `mapDistort`) draws that pass
    // into the distortion target too, by the texture it binds there: a window's `pfx\pfx_dist_glass`.
    let distortion: Option<&'a str> = descriptor
      .filter(|it| class.is_composited() && it.is_distorting)
      .and_then(|it| it.find_sampler(XraySurfaceSampler::DISTORTION_ELEMENT, "s_distort"));

    if distortion.is_some() {
      flags |= StaticSurface::IS_DISTORTING;
    }

    if class.is_composited() && blend == 0 && descriptor.is_some_and(Self::is_glass) && Self::is_glass_texture(surface)
    {
      flags |= StaticSurface::IS_GLASS;
    }

    Some(StaticSurfacePlan {
      class,
      alpha_reference: reference,
      flags,
      slots,
      terrain,
      environment,
      lightmap,
      distortion,
      detail_scale: detail.map_or(1.0, |it| it.scale),
      material: descriptor.map_or(DEFAULT_MATERIAL, |it| it.material),
    })
  }

  /// A surface's flags by what its shader entry (`descriptor`) asks for and which of its own slots name a texture
  /// (`named`): its class and alpha `reference` deciding its test, and its `blend`.
  fn to_flags(
    descriptor: Option<&XraySurfaceDescriptor>,
    named: &[bool; 7],
    (class, reference, blend): (StaticClass, u8, u32),
  ) -> u32 {
    let mut flags: u32 = 0;

    for (slot, flag) in [
      (StaticSurface::BASE, StaticSurface::HAS_BASE),
      (StaticSurface::DETAIL, StaticSurface::HAS_DETAIL),
      (StaticSurface::BUMP, StaticSurface::HAS_BUMP),
      (StaticSurface::DETAIL_BUMP, StaticSurface::HAS_DETAIL_BUMP),
      (StaticSurface::HEMI, StaticSurface::HAS_HEMI),
    ] {
      if named[slot] {
        flags |= flag;
      }
    }

    // A bump binds as a pair, and a detail's bump adds only to the surface's own.
    if !named[StaticSurface::BUMP_COMPANION] {
      flags &= !(StaticSurface::HAS_BUMP | StaticSurface::HAS_DETAIL_BUMP);
    }

    if !named[StaticSurface::DETAIL_BUMP_COMPANION] {
      flags &= !StaticSurface::HAS_DETAIL_BUMP;
    }

    // A composited surface tests its own reference where it has one.
    if class == StaticClass::CutOut || (class.is_composited() && reference > 0) {
      flags |= StaticSurface::IS_CUT_OUT;
    }

    flags |= blend;

    if descriptor.is_some_and(Self::is_environment_mapped) {
      flags |= StaticSurface::IS_ENVIRONMENT_MAPPED;
    }

    if descriptor.is_some_and(|it| it.is_object_lod) {
      flags |= StaticSurface::IS_STILL;
    }

    if descriptor.is_some_and(|it| it.is_emissive) {
      flags |= StaticSurface::IS_EMISSIVE;
    }

    if descriptor.is_some_and(|it| it.is_shadowless) {
      flags |= StaticSurface::IS_SHADOWLESS;
    }

    if flags & StaticSurface::HAS_BUMP != 0 && descriptor.is_some_and(Self::is_steep) {
      flags |= StaticSurface::IS_STEEP;
    } else if flags & StaticSurface::HAS_BUMP != 0
      && descriptor.is_some_and(|it| it.terrain.is_none() && Self::is_relief_class(it))
    {
      flags |= StaticSurface::IS_RELIEF_BUMPED;
    }

    flags
  }

  /// A terrain's four details, bumps and heights and its mask, as the texture cache asks for them.
  fn request_terrain(
    terrain: &XraySurfaceTerrain,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) -> StaticTerrainSlots {
    StaticTerrainSlots {
      details: UVec4::from_array(
        terrain
          .layers
          .each_ref()
          .map(|layer| textures.request(&layer.reference, TextureRole::Detail, source)),
      ),
      bumps: UVec4::from_array(
        terrain
          .layers
          .each_ref()
          .map(|layer| textures.request(&layer.bump, TextureRole::Bump, source)),
      ),
      heights: UVec4::from_array(terrain.layers.each_ref().map(|layer| {
        textures.request(
          &format!("{}{}", layer.reference, TextureRole::HEIGHT_SUFFIX),
          TextureRole::TerrainHeight,
          source,
        )
      })),
      mask: textures.request(&terrain.mask, TextureRole::TerrainMask, source),
      _pad: [0; 3],
    }
  }

  /// Whether a shader entry's blender is strictly sorted, which the engine draws in `mapSorted`, back to front.
  fn is_strictly_sorted(descriptor: &XraySurfaceDescriptor) -> bool {
    matches!(
      descriptor.declaration,
      XraySurfaceDeclaration::Described {
        is_strict_sorting: true,
        ..
      }
    )
  }

  /// A run of impostors' surface, asking for its atlas and the atlas's `_nm` companion.
  pub fn build_impostor(
    surface: &SectorSurface,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) -> StaticSurface {
    let mut texture_slots: [u32; 8] = [0; 8];
    let mut flags: u32 = 0;

    if let Some(atlas) = surface.texture_name.as_deref().filter(|it| !it.is_empty()) {
      texture_slots[StaticSurface::BASE] = textures.request(atlas, TextureRole::Base, source);
      texture_slots[StaticSurface::HEMI] = textures.request(
        &format!("{atlas}{IMPOSTOR_COMPANION_SUFFIX}"),
        TextureRole::ImpostorCompanion,
        source,
      );
      flags |= StaticSurface::HAS_BASE | StaticSurface::HAS_HEMI;
    }

    StaticSurface {
      tiling: 1.0,
      detail_scale: 1.0,
      alpha_reference: 0.0,
      slice: material_slice(Self::IMPOSTOR_MATERIAL),
      color: Self::to_surface_color(surface.shader_id),
      flags,
      textures: texture_slots,
      terrain: StaticTerrainSlots::default(),
      distortion: 0,
      _pad: [0; 3],
    }
  }

  /// Whether a shader entry names one of the engines' glass shaders, whatever its case.
  fn is_glass(descriptor: &XraySurfaceDescriptor) -> bool {
    descriptor
      .shader
      .as_deref()
      .is_some_and(|shader| GLASS_SHADERS.iter().any(|glass| shader.eq_ignore_ascii_case(glass)))
  }

  /// Whether a surface's base is named as a pane's.
  fn is_glass_texture(surface: &SectorSurface) -> bool {
    surface.texture_name.as_deref().is_some_and(|name| {
      let name: String = name.to_ascii_lowercase();

      GLASS_TEXTURES.iter().any(|glass| {
        if glass.ends_with('\\') {
          name.starts_with(glass)
        } else {
          name.contains(glass)
        }
      })
    })
  }

  fn is_wallmark(descriptor: &XraySurfaceDescriptor) -> bool {
    matches!(
      descriptor.declaration,
      XraySurfaceDeclaration::Scripted { is_wallmark: true, .. }
    )
  }

  /// A level class's surface whose base's descriptor asks for parallax, `bUseSteepParallax`.
  fn is_steep(descriptor: &XraySurfaceDescriptor) -> bool {
    descriptor
      .bump
      .as_ref()
      .is_some_and(|it| it.mode == XrayBumpMode::Parallax)
      && matches!(&descriptor.declaration, XraySurfaceDeclaration::Described { class, .. } if STEEP_CLASSES.contains(&class.as_str()))
  }

  /// A level surface of an opaque class (`RELIEF_CLASSES`).
  fn is_relief_class(descriptor: &XraySurfaceDescriptor) -> bool {
    match &descriptor.declaration {
      XraySurfaceDeclaration::Described { class, .. } => RELIEF_CLASSES.contains(&class.as_str()),
      _ => false,
    }
  }

  /// `B_MODEL_EbB`'s, which blends toward an environment map where its base is thin.
  fn is_environment_mapped(descriptor: &XraySurfaceDescriptor) -> bool {
    matches!(&descriptor.declaration, XraySurfaceDeclaration::Described { class, .. } if class == ENVIRONMENT_MAPPED_CLASS)
  }

  /// A stable colour per shader table entry, from a hue its id decides, so a surface reads the same in every sector.
  fn to_surface_color(shader_id: u16) -> Vec3 {
    let hue: f32 = (shader_id as f32 * HUE_STEP) % 360.0;
    let (saturation, lightness): (f32, f32) = (0.45, 0.6);
    let reach: f32 = saturation * lightness.min(1.0 - lightness);
    let channel = |offset: f32| -> f32 {
      let turn: f32 = (offset + hue / 30.0) % 12.0;

      lightness - reach * (turn - 3.0).min(9.0 - turn).clamp(-1.0, 1.0)
    };

    Vec3::new(channel(0.0), channel(8.0), channel(4.0))
  }
}
