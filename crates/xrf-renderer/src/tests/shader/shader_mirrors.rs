//! What WGSL declares by hand of what Rust also declares, held to Rust: constants, flags and the sizes they set.

use xrf_renderer_core::{ShaderAddressSpace, ShaderLayoutVerifier};

use crate::contract::feature::lighting::render_contact_shadow_settings::RENDER_MAX_CONTACT_SHADOW_LIGHTS;
use crate::contract::view::output::render_debug_view::RenderDebugView;
use crate::feature::effects::flares::draw::flare_pass::FlarePass;
use crate::feature::effects::rain::draw::rain_pass::RainPass;
use crate::feature::effects::rain::rain_uniform::RainUniform;
use crate::feature::image::camera::camera_effects_uniform::CameraEffectsUniform;
use crate::feature::image::exposure::exposure_feature::ExposureFeature;
use crate::feature::image::exposure::exposure_pass::ExposurePass;
use crate::feature::image::exposure::histogram::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::feature::image::present::display::display_mode::DisplayMode;
use crate::feature::image::present::present_uniform::PresentUniform;
use crate::feature::image::resolve::fsr::fsr_pass::FsrPass;
use crate::feature::lighting::lights::lights_pass::LightsPass;
use crate::feature::scene::calibration::calibration_uniform::CalibrationUniform;
use crate::feature::scene::grass::cards::grass_cards::GrassCards;
use crate::feature::scene::grass::grass_pass::GrassPass;
use crate::feature::scene::pyramid::depth_pyramid_pass::DepthPyramidPass;
use crate::feature::scene::statics::cull::static_cull_pass::StaticCullPass;
use crate::feature::tracing::scene_trace::requests::trace_request::TraceRequest;
use crate::feature::tracing::scene_trace::requests::water_rays::WaterRays;
use crate::feature::tracing::surroundings::surroundings_pass::SurroundingsPass;
use crate::feature::tracing::view_trace::sway::trace_sway_pass::TraceSwayPass;
use crate::feature::weather::sky::irradiance::sky_irradiance_pass::SkyIrradiancePass;
use crate::feature::weather::sky::prefilter::sky_prefilter_pass::SkyPrefilterPass;
use crate::feature::weather::volumetric::froxels::froxel_pass::FroxelPass;
use crate::feature::weather::water::enhanced::enhanced_water_scroll::EnhancedWaterScroll;
use crate::feature::weather::water::water_uniform::WaterUniform;
use crate::feature::weather::wet::puddles::puddle_sites_pass::PuddleSitesPass;
use crate::feature::weather::wet::puddles::puddle_sites_uniform::PuddleSitesUniform;
use crate::feature::weather::wet::wet_uniform::WetUniform;
use crate::frame::targets::depth_pyramid::DepthPyramid;
use crate::frame::targets::view_targets::ViewTargets;
use crate::lighting::material_slice::material_slice;
use crate::pass::view::camera_uniform::CameraUniform;
use crate::pass::view::light_view::LightView;
use crate::pass::view::noise_turn::NoiseTurn;
use crate::pass::window::backdrop_uniform::BackdropUniform;
use crate::scene::level::lights::atlas::level_light_shadows::LevelLightShadows;
use crate::scene::level::particles::particle_surface_record::ParticleSurfaceRecord;
use crate::scene::static_scene::draw::slot::static_row::StaticRow;
use crate::scene::static_scene::draw::slot::static_slot::StaticSlot;
use crate::scene::static_scene::draw::static_class::StaticClass;
use crate::scene::static_scene::geometry::static_geometry::StaticGeometry;
use crate::scene::static_scene::geometry::static_impostor::StaticImpostor;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::surface::build::static_surface_builder::StaticSurfaceBuilder;
use crate::scene::static_scene::surface::static_surface::StaticSurface;
use crate::scene::static_scene::surface::static_surface_pass::StaticSurfacePass;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::shader::fixtures::{
  get_constant, get_float_constant, get_signed_constant, get_vector_constant, parse,
};

/// The inverse square's lamps give as much light within their reach as the engine's falloff does: the two integrated
/// over the ball of the reach, `x` its share, `(1 - x²)` against `share · (1 - x⁴)² / x²`, each by the shell's `x²`.
#[test]
fn the_inverse_square_lamps_give_the_engines_light_within_their_reach() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let share: f32 = get_float_constant(&library, "common/local_lights", "LIGHT_ENERGY_SHARE");
  let steps: u32 = 4096;
  let (mut engine, mut inverse_square): (f32, f32) = (0.0, 0.0);

  for step in 0..steps {
    let x: f32 = (step as f32 + 0.5) / steps as f32;

    engine += (1.0 - x * x) * x * x;
    inverse_square += share * (1.0 - x.powi(4)).powi(2);
  }

  assert!(
    (inverse_square / engine - 1.0).abs() < 1e-3,
    "{inverse_square} against {engine}"
  );
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

/// The enhanced water turns, tiles and scales its map reads as Rust works out where each has scrolled to, so each
/// read's offset lands where the shader's own coordinates would have put it.
#[test]
fn the_enhanced_water_reads_its_maps_as_rust_scrolls_them() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let module: &str = "static/water_enhanced";

  assert_eq!(
    get_vector_constant(&library, module, "WAVE_TURN_COS"),
    EnhancedWaterScroll::WAVE_TURN_COS
  );
  assert_eq!(
    get_vector_constant(&library, module, "WAVE_TURN_SIN"),
    EnhancedWaterScroll::WAVE_TURN_SIN
  );
  assert_eq!(
    get_vector_constant(&library, module, "WAVE_TILES"),
    EnhancedWaterScroll::WAVE_TILES
  );
  assert_eq!(
    get_float_constant(&library, module, "GUST_SCALE"),
    EnhancedWaterScroll::GUST_SCALE
  );
  assert_eq!(
    get_float_constant(&library, module, "RIPPLE_MAP_SCALE"),
    EnhancedWaterScroll::RIPPLE_MAP_SCALE
  );
  assert_eq!(
    get_vector_constant(&library, module, "RIPPLE_LAYER_SCALES"),
    EnhancedWaterScroll::RIPPLE_LAYER_SCALES
  );
}

/// The water's and the rain's clocks wrap where every reader of them repeats a whole number of times, so a wrap moves
/// nothing they draw: the engine water's wave and its normal layers' circle; the wet surfaces' splashes at either
/// engine's pace, their flow at each slope's step and either engine's pace, and their rain rings' layers.
#[test]
fn the_water_and_rain_clocks_wrap_where_their_readers_repeat() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let whole = |turns: f64| (turns - turns.round()).abs() < 1e-3;
  let wave_turns: f64 = WaterUniform::PERIOD / std::f64::consts::TAU;
  let scroll_rate: f32 = get_float_constant(&library, "static/water", "SCROLL_RATE");

  assert!(whole(wave_turns));
  assert!(whole(wave_turns * f64::from(scroll_rate)), "{scroll_rate}");

  let (wet, period): (&str, f64) = ("frame/wet_patch", RainUniform::CLOCK_PERIOD);
  let steps: f32 = get_float_constant(&library, wet, "FLOW_SLIDE_STEPS");
  let ripple: f32 = get_float_constant(&library, wet, "RIPPLE_PACE");

  for pace in get_vector_constant(&library, wet, "SPLASH_PACE") {
    assert!(whole(period * f64::from(pace)), "splash {pace}");
  }

  for pace in get_vector_constant(&library, wet, "FLOW_SLIDE_PACE") {
    for step in 1..=steps as u32 {
      assert!(
        whole(period * f64::from(pace) * f64::from(step) / f64::from(steps)),
        "flow {pace} at {step}"
      );
    }
  }

  for speed in get_vector_constant(&library, wet, "RIPPLE_LAYER_SPEEDS") {
    assert!(whole(period * f64::from(ripple) * f64::from(speed)), "ripple {speed}");
  }
}

/// The puddles' gusts drift over a repeat each of the gust noise's two readings fits a whole number of times in, the
/// second reading at half the drift.
#[test]
fn the_puddles_gusts_repeat_where_both_readings_do() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let tiles: Vec<f32> = get_vector_constant(&library, "frame/wet_patch", "GUST_TILES");
  let whole = |value: f64| (value - value.round()).abs() < 1e-9;

  assert!(whole(WetUniform::GUST_REPEAT / f64::from(tiles[0])));
  assert!(whole(WetUniform::GUST_REPEAT * 0.5 / f64::from(tiles[1])));
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

/// The physical light's exposure counts its histogram in the bins, workgroups and strides Rust sizes it by, and the
/// sky's level in the slots Rust leaves after the bins.
#[test]
fn the_exposure_histogram_is_sized_as_rust_sizes_it() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/exposure_histogram", "BINS"),
    ExposureHistogramUniform::BINS
  );
  assert_eq!(
    get_float_constant(&library, "frame/exposure_histogram", "BRIGHT_SHARE"),
    ExposureHistogramUniform::BRIGHT_SHARE
  );
  assert_eq!(
    get_constant(&library, "frame/exposure_histogram", "ENGINE_SUM"),
    ExposureHistogramUniform::BINS
  );
  assert_eq!(
    get_constant(&library, "frame/exposure_histogram", "ENGINE_COUNT") + 1,
    ExposureHistogramUniform::BINS + ExposureHistogramUniform::ENGINE_SLOTS
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

/// The rain rings' draw takes as many vertices a near drop as `vs_ring` lays its two falls' copies out over, so every
/// near drop rings.
#[test]
fn the_rain_rings_draw_every_near_drops_vertices() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "frame/rain", "RING_VERTICES"),
    RainPass::RING_VERTICES
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
/// The camera effects' stages reduce over the blocks and tiles Rust sizes their targets by: a motion block's shown
/// pixels, and a depth of field tile's half-size texels and the shown pixels they span.
#[test]
fn the_camera_effects_reduce_over_the_tiles_rust_sizes() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let module: &str = "frame/camera_effects";

  assert_eq!(
    get_signed_constant(&library, module, "MOTION_BLOCK"),
    CameraEffectsUniform::MOTION_BLOCK.cast_signed()
  );
  assert_eq!(
    get_signed_constant(&library, module, "FOCUS_TILE_TEXELS") * 2,
    CameraEffectsUniform::FOCUS_TILE.cast_signed()
  );
  assert_eq!(
    get_float_constant(&library, module, "FOCUS_TILE_PIXELS"),
    CameraEffectsUniform::FOCUS_TILE as f32
  );
}

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

/// The grass's cards start thinning where Rust stops treating every traced tuft as whole.
#[test]
fn the_grass_cards_thin_from_where_rust_stops_their_whole_reach() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_float_constant(&library, "grass/card_thinning", "CARD_THIN_FROM"),
    GrassCards::THIN_FROM
  );
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

/// The reflected rays, the surfaces' and the water's, climb a nearest depth's pyramid no higher than its last level.
#[test]
fn the_rays_climb_the_pyramid_as_high_as_rust_makes_it() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_signed_constant(&library, "common/reflection_trace", "RAY_TOP_LEVEL") + 1,
    DepthPyramid::RAY_LEVELS.cast_signed()
  );
}

/// The water's every-pixel ray records take the words Rust makes room for at each pixel.
#[test]
fn the_water_ray_records_fill_the_room_rust_makes() {
  let library: ShaderLibrary = ShaderLibrary::default();

  assert_eq!(
    get_constant(&library, "common/water_rays", "WATER_RAY_WORDS"),
    WaterRays::WORDS
  );
}

/// Each compute entry dispatched by groups of a size Rust counts in declares that size: one side for a line of work,
/// two for a square of pixels.
#[test]
fn the_compute_entries_run_the_workgroups_rust_dispatches() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let line = |size: u32| [size, 1, 1];
  let square = |size: u32| [size, size, 1];
  let entries: [(&str, &[&str], [u32; 3]); 19] = [
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
    (
      "frame/reflection_scene",
      &["cs_water_scene"],
      square(WaterRays::WORKGROUP),
    ),
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
