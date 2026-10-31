use xrf_renderer_core::{GraphDeclaration, GraphTexture, UniformBinding};

use crate::frame::view_targets::ViewTargets;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::wet_uniform::WetUniform;
use crate::scene::level::cover_textures::CoverTextures;
use crate::scene::level::feature::before_water_readers::BeforeWaterReaders;
use crate::scene::level::feature::bitmask_tracing::BitmaskTracing;
use crate::scene::level::feature::combine_inputs::CombineInputs;
use crate::scene::level::feature::light_buffers::LightBuffers;
use crate::scene::level::feature::light_draw_inputs::LightDrawInputs;
use crate::scene::level::feature::lights_feature::LightsFeature;
use crate::scene::level::feature::ray_guides::RayGuides;
use crate::scene::level::feature::reflection_source::ReflectionSource;
use crate::scene::level::feature::traced_scene::TracedScene;
use crate::scene::level::feature::water_inputs::WaterInputs;
use crate::scene::level::feature::wet_targets::WetTargets;
use crate::scene::level::grass_card_handles::GrassCardHandles;
use crate::scene::level::grass_planting::GrassPlanting;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;
use crate::scene::level::grass_source::GrassSource;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_surface::LevelSurface;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::lighting_recorded::LightingRecorded;
use crate::scene::level::scene_view::SceneView;
use crate::scene::level::shadows_recorded::ShadowsRecorded;
use crate::scene::level::sky_views::SkyViews;
use crate::scene::level::surroundings_cube::SurroundingsCube;
use crate::scene::level::view_record::ViewRecord;

impl SceneView {
  /// Declares the lighting: the lights binned into the view's clusters, the contact shadows marched, the sun then every
  /// light drawn into the light target, the binning's overflow read back, and the ambient occlusion and the indirect
  /// light searched and filtered. Answers the indirect light filtered, none where it is not gathered, and how much of
  /// the sun reaches each pixel.
  fn add_lighting_passes<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    traced: Option<TracedScene<'a>>,
    wet: WetTargets,
  ) -> (Option<GraphTexture>, GraphTexture) {
    let ViewRecord {
      passes,
      view,
      handles: targets,
      ..
    } = record;
    let lights: &'a LightsFeature = &self.features.lights;
    let buffers: LightBuffers = lights.add_binning_passes(GraphDeclaration::new(graph, bindings, runtime), passes);

    // The contact shadows the sun multiplies its own by, and the uniform the lights march theirs by.
    let (contact_shadows, contact): (GraphTexture, UniformBinding<ContactShadowUniform>) = self
      .features
      .contact_shadows
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), passes, view, targets);
    // What the sun sees, traced where DLSS ray reconstruction denoises it; the cascades shadow it otherwise.
    let sun_traced: Option<GraphTexture> = self.features.sun_trace.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      view,
      targets,
      traced,
    );
    let sun_seen: GraphTexture = self.features.sun.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      wet,
      contact_shadows,
      sun_traced,
      self.features.shadows.get_values().filter_reach,
    );

    lights.add_draw_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      LightDrawInputs {
        atlas: scene.lights.get_shadow_atlas(),
        buffers,
        wet,
        contact,
        contact_lights: self.features.contact_shadows.get_light_count(),
        // The shadowed lights traced where DLSS ray reconstruction denoises them, their atlas faces otherwise.
        traced: traced.filter(|_| self.shared.resolve.is_ray_reconstructed),
        noise_frame: self.shared.resolve.noise_frame,
      },
    );
    lights.add_overflow_pass(&mut *graph, buffers);
    // The occlusion and the indirect light recorded beside the lights, so neither group bounds the frame's encoding.
    graph.begin_group("occlusion");

    // GTAO searches on its own; VBAO, the indirect light or both share one bitmask search.
    self
      .features
      .ambient_occlusion
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), passes, view, targets);

    let indirect_light: Option<GraphTexture> = self.features.bitmask.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      view,
      targets,
      BitmaskTracing {
        traced,
        is_light_traced: self.features.indirect_trace.is_traced(traced.as_ref()),
      },
    );

    (indirect_light, sun_seen)
  }

  /// The lighting every lit pass binds: the view's lighting uniform, the exposure, the material table and the sun's
  /// shadow maps with what they were drawn with.
  pub(super) fn import_lighting<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
  ) -> LightingHandles<'a> {
    LightingHandles {
      lighting: runtime.push_uniform(&self.shared.lighting),
      exposure: bindings.import_buffer(&mut *graph, "exposure", &self.features.exposure.state),
      material_lut: bindings.import_view(&mut *graph, "material table", &passes.lighting.table.view),
      brdf_table: bindings.import_view(&mut *graph, "brdf table", &passes.lighting.brdf_table.view),
      lut_sampler: &passes.lighting.table.sampler,
      shadow_maps: bindings.import_view(
        &mut *graph,
        "sun shadow maps",
        self
          .features
          .shadows
          .get_maps()
          .map_or(passes.lighting.sun.get_unshadowed(), |maps| &maps.view),
      ),
      shadows: runtime.push_uniform(self.features.shadows.get_values()),
    }
  }

  /// The scene's group: the distortion cleared, the grass planted, the static scene culled and drawn into the G-buffer,
  /// the grass culled against its depth and its cards stood, the reflections' depth kept before or after the grass
  /// draws, the wall marks and the sky's motion. Answers the grass's cards.
  pub(super) fn record_scene<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
  ) -> Option<GrassCardHandles> {
    let ViewRecord {
      passes,
      view,
      textures,
      handles,
      statics,
      ..
    } = record;
    let is_drawn: bool = !self.shared.is_wireframe;
    let grass: GrassSource = GrassSource {
      pass: &passes.scene.grass,
      level: scene.grass.get_level(),
    };

    ViewTargets::add_distortion_clear(&mut *graph, handles);

    let grass_planting: Option<GrassPlanting> = self
      .features
      .grass
      .add_planting(GraphDeclaration::new(graph, bindings, runtime), grass);

    self
      .features
      .statics
      .add_gbuffer_passes(GraphDeclaration::new(graph, bindings, runtime), record);
    if is_drawn {
      self
        .features
        .calibration
        .add_passes(GraphDeclaration::new(graph, bindings, runtime), passes, view, handles);
    }

    // The grass is culled once the static scene's depth is reduced, so a cell behind it plants nothing to draw.
    if let Some(grass_planting) = grass_planting {
      let occlusion: GrassOcclusionParameters = GrassOcclusionParameters {
        pyramid: statics.pyramid,
        occlusion: runtime.push_uniform(&self.features.statics.get_frame_occlusion()),
      };

      self
        .features
        .grass
        .add_cull(&mut *graph, grass, grass_planting, occlusion);
    }

    // The grass's cards, which cast its shadows into the sun's cascades or the trace, stood once its ring is planted.
    let grass_cards: Option<GrassCardHandles> = grass_planting.and_then(|grass_planting| {
      self
        .features
        .grass
        .add_cards(GraphDeclaration::new(graph, bindings, runtime), grass, grass_planting)
    });
    // The depth the reflections walk their rays over, kept before the grass draws, or after it where the grass is
    // reflected.
    let is_grass_reflected: bool = self.features.reflections.get_settings().is_grass_reflected;

    if !is_grass_reflected {
      self.features.reflections.add_pyramid_passes(
        GraphDeclaration::new(graph, bindings, runtime),
        passes,
        handles.depth,
      );
    }

    self.features.statics.add_stats_pass(&mut *graph, &statics);

    if let (true, Some(grass_planting)) = (is_drawn, grass_planting) {
      self
        .features
        .grass
        .add_draw(&mut *graph, grass, handles, grass_planting, view, textures);
    }

    if is_grass_reflected {
      self.features.reflections.add_pyramid_passes(
        GraphDeclaration::new(graph, bindings, runtime),
        passes,
        handles.depth,
      );
    }

    if is_drawn {
      self.features.statics.add_wallmark_passes(&mut *graph, record);
      self.features.sky.add_motion_pass(&mut *graph, passes, view, handles);
    }

    grass_cards
  }

  /// The shadows' group: the sun's cascades with the grass's cards, the rain's cover while it rains or the surfaces are
  /// wet, the level's surface the enhanced wetting places its puddles by, and the lamps' faces where this view draws
  /// the scene's first (as `frame` notes).
  pub(super) fn record_shadows<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    grass_cards: Option<GrassCardHandles>,
    frame: &LevelFrame,
  ) -> ShadowsRecorded {
    let ViewRecord { passes, textures, .. } = record;
    let grass_casters: Option<GrassShadowCasters> = self.features.grass.get_shadow_casters(
      GraphDeclaration::new(graph, bindings, runtime),
      GrassSource {
        pass: &passes.scene.grass,
        level: scene.grass.get_level(),
      },
      textures,
      grass_cards,
    );

    self
      .features
      .shadows
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record, grass_casters);

    // The rain's cover, which its passes draw and reduce and the rain and the wet surfaces read.
    let covers: Option<CoverTextures> = self
      .features
      .rain_cover
      .as_ref()
      .filter(|_| self.features.rain.is_raining() || self.features.wet.is_wet())
      .map(|rain_cover| rain_cover.add_passes(GraphDeclaration::new(graph, bindings, runtime), record));
    let wet: Option<UniformBinding<WetUniform>> = self.features.wet.push_uniform(runtime);
    // The level's surface seen from overhead, its lowest heights, its puddle sites and its water, which the enhanced
    // wetting places its puddles by.
    let surface: [GraphTexture; 3] = LevelSurface::add_surface_passes(
      self
        .features
        .level_surface
        .as_ref()
        .filter(|_| self.features.wet.is_enhanced()),
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      wet,
      self.features.water.get_uniform(),
    );

    if frame.is_scene_first {
      scene
        .lights
        .add_shadow_passes(GraphDeclaration::new(graph, bindings, runtime), record, grass_casters);
    }

    ShadowsRecorded { covers, wet, surface }
  }

  /// The lighting's group and the reflections': the scene's trace, the lights and the sun, the sky, the traced indirect
  /// light, the reflections and combine, the guides ray reconstruction reads, the frame copied before the water, and
  /// the water.
  pub(super) fn record_lighting<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    grass_cards: Option<GrassCardHandles>,
    wet: WetTargets,
    rings: GraphTexture,
  ) -> LightingRecorded<'a> {
    let ViewRecord { handles, statics, .. } = record;
    // The scene's trace, which RTAO, the reflections and the water trace their rays through.
    let traced: Option<TracedScene> = self.features.view_trace.add_scene_trace(
      scene,
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      grass_cards,
      &self.features.lights,
      &self.features.grass,
    );
    let (mut indirect_light, sun_seen): (Option<GraphTexture>, GraphTexture) = self.add_lighting_passes(
      scene,
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      traced,
      wet,
    );

    self
      .features
      .statics
      .add_overdraw_pass(&mut *graph, record, handles.light);
    let surroundings: &SurroundingsCube = &self.features.surroundings;
    let sky: Option<SkyParameters> = self.features.sky.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      surroundings.is_active().then_some(surroundings),
    );
    let mut reflections: Option<GraphTexture> = None;
    let mut reflection_ends: Option<GraphTexture> = None;
    let mut ray_guides: Option<RayGuides> = None;

    // The reflections and combine record in a group of their own, encoded beside the lights before them.
    graph.begin_group("reflections");

    // The last frame's scene and depth the reflections keep: read by this frame's hits, written again after the
    // particles draw.
    let source: Option<ReflectionSource> = self
      .features
      .reflections
      .import_source(GraphDeclaration::new(graph, bindings, runtime));

    if let Some(sky) = sky {
      let mut traced_rays: Option<GraphTexture> = None;
      // How far the traced indirect light's rays went, where it is traced.
      let mut indirect_hits: Option<GraphTexture> = None;

      if let Some((light, hits)) =
        self
          .features
          .indirect_trace
          .add_passes(GraphDeclaration::new(graph, bindings, runtime), record, sky, traced)
      {
        indirect_light = Some(light);
        indirect_hits = Some(hits);
      }

      if let Some((history, ends, traced)) = self.features.reflections.add_passes(
        GraphDeclaration::new(graph, bindings, runtime),
        record,
        wet,
        source,
        &sky,
        traced,
      ) {
        reflections = Some(history);
        reflection_ends = Some(ends);
        traced_rays = Some(traced);
      }

      self.features.combine.add_passes(
        GraphDeclaration::new(graph, bindings, runtime),
        record,
        sky,
        CombineInputs {
          indirect_light,
          reflections,
          wet,
          sun_seen,
        },
      );
      ray_guides = self
        .features
        .resolve
        .add_guide_passes(&mut *graph, record, wet, traced_rays, indirect_hits);
    }

    let water_views: Option<&SkyViews> = self.features.sky.get_views();
    let before_water: Option<GraphTexture> = self.features.resolve.add_before_water(
      &mut *graph,
      handles,
      BeforeWaterReaders {
        is_guided: ray_guides.is_some(),
        is_water_reading: water_views.is_some() && self.features.water.is_drawn() && self.features.water.is_enhanced(),
      },
    );
    let opaque: Option<GraphTexture> = self.features.resolve.add_reactive_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      handles.scene,
      before_water,
    );

    let depth_with_water: Option<GraphTexture> = self.features.water.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      WaterInputs {
        views: water_views,
        before_water,
        args: self.features.statics.list_draw_args(&statics),
        guides: ray_guides,
        rings,
        sun_seen,
        sky,
        traced,
      },
    );

    LightingRecorded {
      indirect_light,
      reflections,
      reflection_ends,
      sky,
      ray_guides,
      before_water,
      opaque,
      depth_with_water,
      source,
    }
  }
}
