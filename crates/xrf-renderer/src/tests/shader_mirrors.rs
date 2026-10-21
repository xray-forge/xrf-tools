//! What WGSL declares by hand of what Rust also declares, held to Rust: constants, flags and the sizes they set.

use xrf_renderer_core::{ShaderAddressSpace, ShaderLayoutVerifier};

use crate::contract::render_contact_shadow_settings::RENDER_MAX_CONTACT_SHADOW_LIGHTS;
use crate::contract::render_debug_view::RenderDebugView;
use crate::frame::view_targets::ViewTargets;
use crate::lighting::material_slice::material_slice;
use crate::lighting::srgb::srgb_decoded;
use crate::pass::backdrop_uniform::BackdropUniform;
use crate::pass::calibration_uniform::CalibrationUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::display_mode::DisplayMode;
use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::exposure_pass::ExposurePass;
use crate::pass::flare_pass::FlarePass;
use crate::pass::froxel_pass::FroxelPass;
use crate::pass::fsr_pass::FsrPass;
use crate::pass::grass_pass::GrassPass;
use crate::pass::light_view::LightView;
use crate::pass::lights_pass::LightsPass;
use crate::pass::noise_turn::NoiseTurn;
use crate::pass::particle_surface_record::ParticleSurfaceRecord;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::puddle_sites_pass::PuddleSitesPass;
use crate::pass::puddle_sites_uniform::PuddleSitesUniform;
use crate::pass::sky_irradiance_pass::SkyIrradiancePass;
use crate::pass::sky_prefilter_pass::SkyPrefilterPass;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::surroundings_pass::SurroundingsPass;
use crate::pass::trace_request::TraceRequest;
use crate::pass::trace_sway_pass::TraceSwayPass;
use crate::scene::level::feature::exposure_feature::ExposureFeature;
use crate::scene::level::level_light_shadows::LevelLightShadows;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_geometry::StaticGeometry;
use crate::scene::static_scene::static_impostor::StaticImpostor;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_row::StaticRow;
use crate::scene::static_scene::static_slot::StaticSlot;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_builder::StaticSurfaceBuilder;
use crate::scene::static_scene::static_surface_pass::StaticSurfacePass;
use crate::shader::shader_library::ShaderLibrary;

/// The physical lamps' intensity gives what the engine's falloff, decoded, gave a third of the way to their reach,
/// where their window has already taken `(1 - (1/3)⁴)²`.
#[test]
fn the_physical_lamps_meet_the_engine_a_third_of_the_way_out() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let window: f32 = (1.0 - (1.0_f32 / 9.0).powi(2)).powi(2);
  let share: f32 = get_float_constant(&library, "common/local_lights", "LIGHT_THIRD_SHARE");

  assert!((share * 9.0 * window - srgb_decoded(8.0 / 9.0)).abs() < 1e-5);
}

/// The cull tells a batch's class and layout by its index as Rust lays batches out, a layout's classes in `StaticClass`
/// order, and finds a surface pass's draw list after them as Rust does.
#[test]
fn the_cull_counts_classes_as_rust_does() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "static/cull", "CLASS_COUNT"),
    StaticClass::ALL.len() as u32
  );
  assert_eq!(
    get_constant(&library, "static/cull", "WATER_CLASS"),
    StaticClass::Water.get_index() as u32
  );
  assert_eq!(
    get_constant(&library, "static/cull", "SURFACE_PASS_COUNT"),
    StaticSurfacePass::COUNT as u32
  );
  assert_eq!(
    get_constant(&library, "static/cull", "TREE_LAYOUT"),
    StaticLayout::Tree.get_index() as u32
  );
}

/// The lamps read their shadows in texels of the atlas the renderer allocates their tiles in.
#[test]
fn the_lamps_read_the_shadow_atlas_at_its_allocated_size() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_float_constant(&library, "common/local_lights", "ATLAS_TEXEL"),
    1.0 / LevelLightShadows::ATLAS_SIZE as f32
  );
}

/// The physical light's exposure counts its histogram in the bins, workgroups and strides Rust sizes it by.
#[test]
fn the_exposure_histogram_is_sized_as_rust_sizes_it() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/exposure_histogram", "BINS"),
    ExposureHistogramUniform::BINS
  );
  assert_eq!(
    get_constant(&library, "frame/exposure_histogram", "HISTOGRAM_WORKGROUP"),
    ExposurePass::HISTOGRAM_WORKGROUP
  );
  assert_eq!(
    get_constant(&library, "frame/exposure_histogram", "SAMPLE_STRIDE"),
    ExposurePass::HISTOGRAM_STRIDE
  );
}

/// The present and the debanding tell how a scene is shown by the numbers Rust gives its display modes.
#[test]
fn the_display_modes_are_numbered_as_rust_numbers_them() {
  let library: ShaderLibrary = ShaderLibrary::default();

  for (name, mode) in [
    ("DISPLAY_AS_IS", DisplayMode::AsIs),
    ("DISPLAY_TONEMAPPED", DisplayMode::Tonemapped),
    ("DISPLAY_ENGINE_ENCODED", DisplayMode::EngineEncoded),
    ("DISPLAY_AGX", DisplayMode::Agx),
    ("DISPLAY_PBR_NEUTRAL", DisplayMode::PbrNeutral),
  ] {
    assert_eq!(
      get_constant(&library, "common/display_transform", name),
      mode.get_index(),
      "{name}"
    );
  }
}

/// The calibration props are spheres up to the index Rust counts them to, the chart after them.
#[test]
fn the_calibration_draws_its_spheres_as_rust_counts_them() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/calibration", "SPHERES"),
    CalibrationUniform::SPHERES
  );
}

/// The present tells a debug view by its index as Rust numbers them, FSR's last and from the first it names on.
#[test]
fn the_present_numbers_debug_views_as_rust_does() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/present", "VIEW_HEIGHTS"),
    RenderDebugView::Heights.get_index()
  );

  for (name, view) in [
    ("VIEW_LUMINANCE", RenderDebugView::Luminance),
    ("VIEW_ALBEDO_RANGE", RenderDebugView::AlbedoRange),
    ("VIEW_OVERDRAW", RenderDebugView::Overdraw),
    ("VIEW_CASCADES", RenderDebugView::Cascades),
    ("VIEW_LAMPS", RenderDebugView::LampsPerCluster),
    ("VIEW_SAMPLED_MIP", RenderDebugView::SampledMip),
    ("VIEW_SPECULAR", RenderDebugView::Specular),
  ] {
    assert_eq!(
      get_constant(&library, "frame/present", name),
      view.get_index(),
      "{name}"
    );
  }

  for (name, view) in [
    ("LIGHT_VIEW_CASCADES", LightView::Cascades),
    ("LIGHT_VIEW_LAMPS", LightView::LampsPerCluster),
    ("LIGHT_VIEW_OVERDRAW", LightView::Overdraw),
  ] {
    assert_eq!(get_constant(&library, "frame/sun", name), view as u32, "{name}");
  }
  assert_eq!(
    get_constant(&library, "frame/present", "VIEW_FSR_FIRST"),
    RenderDebugView::FsrMasks.get_index()
  );
  assert!(RenderDebugView::FsrFrameShare.get_index() > RenderDebugView::FsrMasks.get_index());
}

/// The present tells the engine's bloom from the enhanced one by the values Rust writes.
#[test]
fn the_present_tells_the_blooms_apart_as_rust_does() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/present", "BLOOM_ENGINE"),
    PresentUniform::BLOOM_ENGINE
  );
  assert_eq!(
    get_constant(&library, "frame/present", "BLOOM_ENHANCED"),
    PresentUniform::BLOOM_ENHANCED
  );
}

/// The lights are binned into as many clusters, each holding as many lights, as Rust sizes their buffers and
/// dispatches the binning for.
#[test]
fn the_lights_bin_into_the_clusters_rust_sizes() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let get = |constant: &str| get_constant(&library, "common/light_clusters", constant);

  assert_eq!(get("LIGHT_CLUSTERS"), LightsPass::LIGHT_CLUSTERS);
  assert_eq!(
    get("LIGHT_CLUSTERS_X") * get("LIGHT_CLUSTERS_Y") * get("LIGHT_CLUSTERS_Z"),
    LightsPass::LIGHT_CLUSTERS
  );
  assert_eq!(get("LIGHT_CLUSTER_CAPACITY"), LightsPass::LIGHT_CLUSTER_CAPACITY);
}

/// A pixel marches contact shadows towards at most as many lights as the settings allow, and holds as many visibility
/// slots as the visibility target has layers for.
#[test]
fn the_lights_select_as_many_as_rust_makes_room_for() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let get = |constant: &str| get_constant(&library, "frame/light_selection", constant);

  assert_eq!(get("MAX_CONTACT_LIGHTS"), RENDER_MAX_CONTACT_SHADOW_LIGHTS);
  assert_eq!(
    get("VISIBILITY_SLOTS"),
    LightsPass::VISIBILITY_LAYERS * get("VISIBILITY_LAYER_SLOTS")
  );
}

/// The flares draw their gradient at the instance Rust binds its texture to.
#[test]
fn the_flares_draw_the_gradient_at_its_instance() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/flare", "GRADIENT_INSTANCE"),
    FlarePass::GRADIENT_INSTANCE
  );
}

/// The particles read a surface's flags as Rust writes them.
#[test]
fn the_particles_read_a_surface_s_flags_as_rust_writes_them() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/particles", "SURFACE_IS_CLAMPED"),
    ParticleSurfaceRecord::IS_CLAMPED
  );
}

/// The exposure measures as many cells as its state holds, and the puddles read as many fires as their uniform holds.
#[test]
fn the_exposure_and_the_puddles_fill_the_arrays_rust_sizes() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/exposure", "CELLS"),
    ExposureFeature::CELLS
  );
  assert_eq!(
    get_constant(&library, "frame/puddle_sites", "FIRE_PAIRS") as usize,
    PuddleSitesUniform::FIRE_PAIRS
  );
}

/// The sky's haze and the bloom read their targets at the sizes Rust makes them.
#[test]
fn the_haze_and_the_bloom_read_their_targets_at_their_sizes() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let size = |(width, height): (u32, u32)| vec![width as f32, height as f32];

  assert_eq!(
    get_vector_constant(&library, "frame/sky_haze", "HAZE_SIZE"),
    size(ViewTargets::HAZE_SIZE)
  );
  assert_eq!(
    get_vector_constant(&library, "frame/bloom", "BLOOM_SIZE"),
    size(ViewTargets::BLOOM_SIZE)
  );
}

/// The traced passes turn their noise by the R2 steps the uniforms turn theirs by.
#[test]
fn the_traced_passes_turn_their_noise_as_rust_does() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_vector_constant(&library, "common/noise", "R2_STEPS"),
    NoiseTurn::R2_STEPS
  );
}

/// The static draws read a surface's flags as `StaticSurface` sets them.
#[test]
fn the_static_draws_read_a_surface_s_flags_as_rust_sets_them() {
  let library: ShaderLibrary = ShaderLibrary::default();

  for (name, flag) in [
    ("SURFACE_HAS_BASE", StaticSurface::HAS_BASE),
    ("SURFACE_HAS_DETAIL", StaticSurface::HAS_DETAIL),
    ("SURFACE_HAS_BUMP", StaticSurface::HAS_BUMP),
    ("SURFACE_HAS_DETAIL_BUMP", StaticSurface::HAS_DETAIL_BUMP),
    ("SURFACE_HAS_HEMI", StaticSurface::HAS_HEMI),
    ("SURFACE_IS_CUT_OUT", StaticSurface::IS_CUT_OUT),
    ("SURFACE_IS_SOFT_WATER", StaticSurface::IS_SOFT_WATER),
    ("SURFACE_IS_ANOMALY_WATER", StaticSurface::IS_ANOMALY_WATER),
    ("SURFACE_IS_REFLECTING", StaticSurface::IS_REFLECTING),
    ("SURFACE_IS_SPECULAR", StaticSurface::IS_SPECULAR),
    ("SURFACE_IS_TRANSPARENT", StaticSurface::IS_TRANSPARENT),
    ("SURFACE_IS_FOAMED", StaticSurface::IS_FOAMED),
    ("SURFACE_HAS_WATER_NORMAL", StaticSurface::HAS_WATER_NORMAL),
    ("SURFACE_HAS_FOAM", StaticSurface::HAS_FOAM),
    ("SURFACE_HAS_DISTORTION", StaticSurface::HAS_DISTORTION),
    ("SURFACE_IS_ADDED", StaticSurface::IS_ADDED),
    ("SURFACE_IS_WEIGHTED", StaticSurface::IS_WEIGHTED),
    ("SURFACE_IS_MULTIPLIED", StaticSurface::IS_MULTIPLIED),
    ("SURFACE_IS_DOUBLED", StaticSurface::IS_DOUBLED),
    ("SURFACE_IS_MODEL", StaticSurface::IS_MODEL),
    ("SURFACE_IS_ENVIRONMENT_MAPPED", StaticSurface::IS_ENVIRONMENT_MAPPED),
    ("SURFACE_IS_STILL", StaticSurface::IS_STILL),
    ("SURFACE_IS_TERRAIN", StaticSurface::IS_TERRAIN),
    ("SURFACE_IS_EMISSIVE", StaticSurface::IS_EMISSIVE),
    ("SURFACE_IS_SHADOWLESS", StaticSurface::IS_SHADOWLESS),
    ("SURFACE_IS_STEEP", StaticSurface::IS_STEEP),
    ("SURFACE_HAS_LIGHTMAP", StaticSurface::HAS_LIGHTMAP),
    ("SURFACE_IS_DISTORTING", StaticSurface::IS_DISTORTING),
    ("SURFACE_IS_GLASS", StaticSurface::IS_GLASS),
    ("SURFACE_IS_RELIEF_BUMPED", StaticSurface::IS_RELIEF_BUMPED),
  ] {
    assert_eq!(get_constant(&library, "static/records", name), flag, "{name}");
  }
}

/// The static draws find a surface's textures, and tell a slot's kind, at the places Rust gives them.
#[test]
fn the_static_draws_read_a_surface_s_slots_where_rust_puts_them() {
  let library: ShaderLibrary = ShaderLibrary::default();

  for (name, slot) in [
    ("SLOT_BASE", StaticSurface::BASE),
    ("SLOT_DETAIL", StaticSurface::DETAIL),
    ("SLOT_BUMP", StaticSurface::BUMP),
    ("SLOT_BUMP_COMPANION", StaticSurface::BUMP_COMPANION),
    ("SLOT_DETAIL_BUMP", StaticSurface::DETAIL_BUMP),
    ("SLOT_DETAIL_BUMP_COMPANION", StaticSurface::DETAIL_BUMP_COMPANION),
    ("SLOT_HEMI", StaticSurface::HEMI),
    ("SLOT_LIGHTMAP", StaticSurface::LIGHTMAP),
    ("SLOT_ENVIRONMENT", StaticSurface::ENVIRONMENT),
    ("SLOT_WATER_NORMAL", StaticSurface::WATER_NORMAL),
    ("SLOT_FOAM", StaticSurface::FOAM),
    ("SLOT_DISTORTION", StaticSurface::DISTORTION),
  ] {
    assert_eq!(get_constant(&library, "static/records", name) as usize, slot, "{name}");
  }

  assert_eq!(
    get_constant(&library, "static/records", "SLOT_SINGLE"),
    StaticSlot::SINGLE
  );
  assert_eq!(
    get_constant(&library, "static/records", "SLOT_LISTED"),
    StaticSlot::LISTED
  );
}

/// The static draws span a cluster, mark a row no impostor decides and turn an impostor's facets as Rust counts them.
#[test]
fn the_static_draws_count_clusters_rows_and_facets_as_rust_does() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "static/records", "CLUSTER_VERTICES"),
    StaticGeometry::CLUSTER_VERTICES
  );
  assert_eq!(get_constant(&library, "static/records", "NO_LOD"), StaticRow::NO_LOD);
  assert_eq!(
    get_constant(&library, "static/records", "IMPOSTOR_FACETS") as usize,
    StaticImpostor::FACETS
  );
}

/// The static draws read a vertex's words where `StaticLayout` puts them.
#[test]
fn the_static_draws_read_a_vertex_s_words_where_rust_packs_them() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let get = |constant: &str| get_constant(&library, "static/vertex", constant) as usize;

  assert_eq!(get("VERTEX_WORDS"), StaticLayout::STRIDE as usize);
  assert_eq!(get("VERTEX_POSITION"), StaticLayout::POSITION);
  assert_eq!(get("VERTEX_BINORMAL"), StaticLayout::BINORMAL);
  assert_eq!(get("VERTEX_NORMAL"), StaticLayout::NORMAL);
  assert_eq!(get("VERTEX_TANGENT"), StaticLayout::TANGENT);
  assert_eq!(get("VERTEX_UV"), StaticLayout::UV);
  assert_eq!(get("VERTEX_EXTRA"), StaticLayout::EXTRA);
}

/// The impostors and the grass write the material slices Rust gives their surfaces: the default lighting model, and
/// Blinn at full weight.
#[test]
fn the_impostors_and_the_grass_write_the_material_slices_rust_gives_them() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let impostor: f32 = get_float_constant(&library, "static/impostor", "IMPOSTOR_SLICE");
  let grass: f32 = get_float_constant(&library, "grass/records", "GRASS_MATERIAL_SLICE");

  assert!((impostor - material_slice(StaticSurfaceBuilder::IMPOSTOR_MATERIAL)).abs() < 1e-6);
  assert!((grass - material_slice(1.0)).abs() < 1e-6);
}

/// naga lays the camera and the page's backdrop out where the derive does, and takes each as a uniform.
#[test]
fn the_camera_and_the_backdrop_are_laid_out_as_naga_lays_them() {
  ShaderLayoutVerifier::verify::<CameraUniform>(ShaderAddressSpace::Uniform).unwrap();
  ShaderLayoutVerifier::verify::<BackdropUniform>(ShaderAddressSpace::Uniform).unwrap();
}

/// The static draws lay a surface out as large as it is written.
#[test]
fn a_static_surface_is_as_large_as_its_shaders_read() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_struct_size(&library, "static/gbuffer", "Surface"),
    size_of::<StaticSurface>() as u32
  );
}

/// The bytes a struct of a name takes in a module, as the shader lays it out.
fn get_struct_size(library: &ShaderLibrary, name: &str, type_name: &str) -> u32 {
  let module: naga::Module = parse(library, name);
  let mut layouter: naga::proc::Layouter = naga::proc::Layouter::default();

  layouter.update(module.to_ctx()).expect("lays out");

  let (handle, _) = module
    .types
    .iter()
    .find(|(_, it)| it.name.as_deref() == Some(type_name))
    .unwrap_or_else(|| panic!("'{name}' has no '{type_name}'"));

  layouter[handle].size
}

/// A `u32` constant's value in a module.
fn get_constant(library: &ShaderLibrary, name: &str, constant: &str) -> u32 {
  let module: naga::Module = parse(library, name);

  match module.global_expressions[get_declared(&module, name, constant).init] {
    naga::Expression::Literal(naga::Literal::U32(value)) => value,
    ref other => panic!("'{name}' declares {constant} as {other:?}"),
  }
}

/// Each compute entry dispatched by groups of a size Rust counts in declares that size: one side for a line of work,
/// two for a square of pixels.
#[test]
fn the_compute_entries_run_the_workgroups_rust_dispatches() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let line = |size: u32| [size, 1, 1];
  let square = |size: u32| [size, size, 1];
  let entries: [(&str, &[&str], [u32; 3]); 18] = [
    (
      "frame/pyramid",
      &[
        "cs_reduce_depth",
        "cs_reduce_level",
        "cs_copy_nearest",
        "cs_reduce_nearest",
        "cs_reduce_cover",
        "cs_reduce_cover_level",
      ],
      square(DepthPyramidPass::WORKGROUP),
    ),
    ("frame/exposure", &["cs_measure"], line(ExposurePass::MEASURE_WORKGROUP)),
    (
      "frame/froxel_inject",
      &["cs_froxel_inject"],
      square(FroxelPass::WORKGROUP),
    ),
    (
      "frame/froxel_integrate",
      &["cs_froxel_integrate"],
      square(FroxelPass::WORKGROUP),
    ),
    (
      "frame/fsr/reconstruct",
      &["cs_reconstruct"],
      square(FsrPass::RECONSTRUCT_WORKGROUP),
    ),
    (
      "grass/planting",
      &[
        "cs_clear_schedule",
        "cs_rank",
        "cs_decompress",
        "cs_clear_counts",
        "cs_cull",
      ],
      line(GrassPass::WORKGROUP),
    ),
    (
      "grass/cards",
      &["cs_clear_cards", "cs_grass_cards"],
      line(GrassPass::WORKGROUP),
    ),
    ("frame/light_binning", &["cs_bin"], line(LightsPass::BINNING_WORKGROUP)),
    (
      "frame/light_contact",
      &["cs_light_contact"],
      square(LightsPass::VISIBILITY_WORKGROUP),
    ),
    (
      "frame/light_visibility",
      &["cs_visibility"],
      square(LightsPass::VISIBILITY_WORKGROUP),
    ),
    (
      "frame/light_tracing",
      &["cs_visibility_traced"],
      square(LightsPass::VISIBILITY_WORKGROUP),
    ),
    (
      "frame/puddle_sites",
      &["cs_surface_lowest", "cs_surface_sites", "cs_puddle_keep"],
      square(PuddleSitesPass::WORKGROUP),
    ),
    (
      "frame/sky_irradiance",
      &["cs_irradiance"],
      line(SkyIrradiancePass::WORKGROUP),
    ),
    (
      "frame/sky_prefilter",
      &["cs_prefilter"],
      square(SkyPrefilterPass::WORKGROUP),
    ),
    (
      "static/cull",
      &[
        "cs_cull_singles",
        "cs_cull_impostors",
        "cs_cull_rows",
        "cs_clamp_counts",
        "cs_cull_late",
        "cs_clamp_late",
      ],
      line(StaticCullPass::WORKGROUP),
    ),
    (
      "frame/surroundings",
      &["cs_resolve"],
      square(SurroundingsPass::WORKGROUP),
    ),
    ("static/trace_sway", &["cs_trace_sway"], line(TraceSwayPass::WORKGROUP)),
    ("frame/reflection_scene", &["cs_scene"], line(TraceRequest::WORKGROUP)),
  ];

  for (name, names, size) in entries {
    let module: naga::Module = parse(&library, name);

    for entry in names {
      let declared: [u32; 3] = module
        .entry_points
        .iter()
        .find(|point| point.name == *entry)
        .unwrap_or_else(|| panic!("'{name}' declares no {entry}"))
        .workgroup_size;

      assert_eq!(declared, size, "'{name}' {entry}");
    }
  }
}

/// The value a module declares a float constant as.
fn get_float_constant(library: &ShaderLibrary, name: &str, constant: &str) -> f32 {
  let module: naga::Module = parse(library, name);

  to_float(&module, name, get_declared(&module, name, constant).init)
}

/// The components a module declares a float vector constant with.
fn get_vector_constant(library: &ShaderLibrary, name: &str, constant: &str) -> Vec<f32> {
  let module: naga::Module = parse(library, name);

  match module.global_expressions[get_declared(&module, name, constant).init] {
    naga::Expression::Compose { ref components, .. } => components
      .iter()
      .map(|component| to_float(&module, name, *component))
      .collect(),
    ref other => panic!("'{name}' declares {constant} as {other:?}"),
  }
}

fn parse(library: &ShaderLibrary, name: &str) -> naga::Module {
  let source: String = library.compose(name).expect("composes");

  naga::front::wgsl::parse_str(&source).expect("parses")
}

fn get_declared<'m>(module: &'m naga::Module, name: &str, constant: &str) -> &'m naga::Constant {
  module
    .constants
    .iter()
    .map(|(_, declared)| declared)
    .find(|declared| declared.name.as_deref() == Some(constant))
    .unwrap_or_else(|| panic!("'{name}' declares no {constant}"))
}

fn to_float(module: &naga::Module, name: &str, expression: naga::Handle<naga::Expression>) -> f32 {
  match module.global_expressions[expression] {
    naga::Expression::Literal(naga::Literal::F32(value)) => value,
    ref other => panic!("'{name}' holds {other:?} where a float stands"),
  }
}
