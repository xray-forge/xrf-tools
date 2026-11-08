use xrf_renderer_core::{FrameGraph, GraphDeclaration, GraphTexture, UniformBinding};

use crate::contract::feature::tracing::render_reflection_settings::RenderReflectionSettings;
use crate::feature::effects::rain::cover::cover_textures::CoverTextures;
use crate::feature::image::resolve::before_water_readers::BeforeWaterReaders;
use crate::feature::lighting::combine::combine_inputs::CombineInputs;
use crate::feature::lighting::direct_light_recorded::DirectLightRecorded;
use crate::feature::lighting::lights::draw::light_buffers::LightBuffers;
use crate::feature::lighting::lights::draw::light_draw_inputs::LightDrawInputs;
use crate::feature::lighting::lights::lights_feature::LightsFeature;
use crate::feature::lighting::sun::direct_light_inputs::DirectLightInputs;
use crate::feature::scene::drawable::casters::drawable_kind_ref::DrawableKindRef;
use crate::feature::scene::drawable::draw::drawable_kind_readied::DrawableKindReadied;
use crate::feature::scene::drawable::draw::drawable_kind_source::DrawableKindSource;
use crate::feature::scene::drawable::drawable_kind_traced::DrawableKindTraced;
use crate::feature::scene::statics::cull::static_occlusion_uniform::StaticOcclusionUniform;
use crate::feature::tracing::indirect_trace::indirect_traced::IndirectTraced;
use crate::feature::tracing::ray_guides::ray_guide_inputs::RayGuideInputs;
use crate::feature::tracing::ray_guides::ray_guides::RayGuides;
use crate::feature::tracing::reflections::reflection_source::ReflectionSource;
use crate::feature::tracing::reflections::reflections_recorded::ReflectionsRecorded;
use crate::feature::tracing::surroundings::surroundings_cube::SurroundingsCube;
use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::tracing::view_trace::view_trace_feature::ViewTraceFeature;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::feature::weather::sky::sky_views::SkyViews;
use crate::feature::weather::water::water_inputs::WaterInputs;
use crate::feature::weather::wet::puddles::level_surface::LevelSurface;
use crate::feature::weather::wet::wet_targets::WetTargets;
use crate::feature::weather::wet::wet_uniform::WetUniform;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::view::record::lighting_handles::LightingHandles;
use crate::scene::level::view::record::lighting_recorded::LightingRecorded;
use crate::scene::level::view::record::readied_frame::ReadiedFrame;
use crate::scene::level::view::record::shadows_recorded::ShadowsRecorded;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::scene_view::SceneView;

impl SceneView {
  /// Declares the direct light: the lights binned into the view's clusters, the sun with its contact shadows drawn into
  /// the light target, every light drawn over it, and the binning's overflow read back. Answers how much of the sun
  /// reaches each pixel.
  fn add_direct_light<'a>(
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
  ) -> GraphTexture {
    let lights: &'a LightsFeature = &self.features.lights;
    let buffers: LightBuffers =
      lights.add_binning_passes(GraphDeclaration::new(graph, bindings, runtime), record.passes);
    let direct: DirectLightRecorded = self.stages.lighting.add_direct(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      DirectLightInputs {
        traced,
        wet,
        filter_reach: self.features.shadows.get_values().filter_reach,
      },
    );

    lights.add_draw_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      LightDrawInputs {
        atlas: scene.lights.get_shadow_atlas(),
        buffers,
        wet,
        contact: direct.contact,
        contact_lights: direct.contact_lights,
        // The shadowed lights traced where DLSS ray reconstruction denoises them, their atlas faces otherwise.
        traced: traced.filter(|_| self.shared.resolve.is_ray_reconstructed),
        noise_frame: self.shared.resolve.noise_frame,
      },
    );
    lights.add_overflow_pass(&mut *graph, buffers);

    direct.sun_seen
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

  /// The scene's group, in the phases every drawable kind draws by: the distortion cleared, the kinds readied, the
  /// static scene culled and drawn into the G-buffer, the kinds culled against its depth and their casters stood, those
  /// the reflections walk over drawn before the reflections keep their depth and the rest after, the wall marks and the
  /// sky's motion. Answers the kinds as the frame readied them.
  pub(super) fn record_scene<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
  ) -> Vec<DrawableKindReadied<'a>> {
    let ViewRecord {
      passes,
      view,
      textures,
      handles,
      statics,
      ..
    } = record;
    let is_drawn: bool = !self.shared.is_wireframe;

    ViewTargets::add_distortion_clear(&mut *graph, handles);

    let mut kinds: Vec<DrawableKindReadied<'a>> = DrawableKindSource::list(&self.features.grass, scene, passes)
      .into_iter()
      .filter_map(|kind| kind.add_ready(GraphDeclaration::new(graph, bindings, runtime)))
      .collect();

    self
      .features
      .statics
      .add_gbuffer_passes(GraphDeclaration::new(graph, bindings, runtime), record);
    if is_drawn {
      self
        .stages
        .scene
        .add_passes(GraphDeclaration::new(graph, bindings, runtime), record);
    }

    // The kinds are culled once the static scene's depth is reduced, so what stands behind it draws nothing; then their
    // casters stand, which the sun's cascades, the lamps' faces and the trace draw.
    if !kinds.is_empty() {
      let occlusion: UniformBinding<StaticOcclusionUniform> =
        runtime.push_uniform(&self.features.statics.get_frame_occlusion());

      for kind in &kinds {
        kind.add_cull(&mut *graph, statics.pyramid, occlusion);
      }
    }

    for kind in &mut kinds {
      kind.add_casters(GraphDeclaration::new(graph, bindings, runtime));
    }

    self.features.statics.add_stats_pass(&mut *graph, &statics);

    // The depth the reflections walk their rays over, kept once the kinds they walk over are drawn, before the rest.
    let reflections: RenderReflectionSettings = self.features.reflections.get_settings();
    let add_draws = |graph: &mut FrameGraph<'a>, is_reflected: bool| {
      for kind in kinds
        .iter()
        .filter(|kind| is_drawn && kind.is_reflected(&reflections) == is_reflected)
      {
        kind.add_draw(graph, handles, view, textures);
      }
    };

    add_draws(&mut *graph, true);
    self.features.reflections.add_pyramid_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      handles.depth,
    );
    add_draws(&mut *graph, false);

    if is_drawn {
      self.features.statics.add_wallmark_passes(&mut *graph, record);
      self.features.sky.add_motion_pass(&mut *graph, passes, view, handles);
    }

    kinds
  }

  /// The shadows' group: the sun's cascades with what the drawable `kinds` cast, the rain's cover while it rains or the
  /// surfaces are wet, the level's surface the enhanced wetting places its puddles by, and the lamps' faces where this
  /// view draws the scene's first (as `frame` notes).
  pub(super) fn record_shadows<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    kinds: &[DrawableKindReadied<'a>],
    frame: &ReadiedFrame,
  ) -> ShadowsRecorded {
    let casters: Vec<DrawableKindRef<'a>> = kinds
      .iter()
      .filter_map(|kind| kind.get_shadow_casters(GraphDeclaration::new(graph, bindings, runtime), record.textures))
      .collect();

    self
      .features
      .shadows
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record, &casters);

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

    if frame.is_scene_recorder {
      scene
        .lights
        .add_shadow_passes(GraphDeclaration::new(graph, bindings, runtime), record, &casters);
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
    kinds: &[DrawableKindReadied<'a>],
    wet: WetTargets,
    rings: GraphTexture,
  ) -> LightingRecorded<'a> {
    let ViewRecord { handles, statics, .. } = record;
    // Each kind's part of the scene's trace, where the view traces it, recorded whether or not the kind readied: the
    // trace's shaders bind it regardless.
    let mut kinds_traced: Vec<DrawableKindTraced> = Vec::new();

    if let Some(trace) = ViewTraceFeature::get_scene_trace(scene, record.passes) {
      for kind in DrawableKindSource::list(&self.features.grass, scene, record.passes) {
        kinds_traced.push(kind.add_trace(
          GraphDeclaration::new(graph, bindings, runtime),
          record,
          kinds,
          trace.get_empty(),
        ));
      }
    }

    // The scene's trace, which RTAO, the reflections and the water trace their rays through.
    let traced: Option<TracedScene<'_>> = self.features.view_trace.add_scene_trace(
      scene,
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      &self.features.lights,
      &kinds_traced,
    );
    let sun_seen: GraphTexture = self.add_direct_light(
      scene,
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      traced,
      wet,
    );

    // The occlusion and the indirect light recorded beside the lights, so neither group bounds the frame's encoding.
    graph.begin_group("occlusion");

    let mut indirect_light: Option<GraphTexture> =
      self
        .stages
        .lighting
        .add_occlusion(GraphDeclaration::new(graph, bindings, runtime), record, traced);

    self
      .features
      .statics
      .add_overdraw_pass(&mut *graph, record, handles.light);
    let surroundings: &SurroundingsCube = &self.features.surroundings;
    let sky: Option<SkyParameters<'_>> = self.features.sky.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      surroundings.is_active().then_some(surroundings),
    );
    let mut reflections: Option<ReflectionsRecorded> = None;
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
      // The traced indirect light, where it is traced, in place of the bitmask's.
      let indirect: Option<IndirectTraced> =
        self
          .stages
          .lighting
          .add_indirect_trace(GraphDeclaration::new(graph, bindings, runtime), record, sky, traced);

      indirect_light = indirect.map(|indirect| indirect.light).or(indirect_light);

      reflections = self.features.reflections.add_passes(
        GraphDeclaration::new(graph, bindings, runtime),
        record,
        wet,
        source,
        &sky,
        traced,
      );

      // The sky as drawn: its textures smoothed where they band, while the debanding is enhanced.
      let drawn: SkyParameters<'_> =
        self
          .features
          .sky
          .add_drawn_sky(GraphDeclaration::new(graph, bindings, runtime), record.passes, sky);

      self.stages.lighting.add_combine(
        GraphDeclaration::new(graph, bindings, runtime),
        record,
        drawn,
        CombineInputs {
          indirect_light,
          reflections: reflections.map(|reflections| reflections.radiance),
          wet,
          sun_seen,
        },
      );
      ray_guides = self.features.resolve.add_guide_passes(
        &mut *graph,
        record,
        RayGuideInputs {
          wet,
          reflections,
          indirect,
        },
      );
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
      sky,
      ray_guides,
      before_water,
      opaque,
      depth_with_water,
      source,
    }
  }
}
