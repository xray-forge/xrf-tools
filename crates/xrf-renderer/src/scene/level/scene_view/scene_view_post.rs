use xrf_renderer_core::{GraphDeclaration, GraphTexture};

use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::sky_parameters::SkyParameters;
use crate::scene::level::feature::composited_draw::CompositedDraw;
use crate::scene::level::feature::image_chain::ImageChain;
use crate::scene::level::feature::rain_covers::RainCovers;
use crate::scene::level::feature::resolve_feature::ResolveFeature;
use crate::scene::level::feature::shaft_inputs::ShaftInputs;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::lighting_recorded::LightingRecorded;
use crate::scene::level::scene_output::SceneOutput;
use crate::scene::level::scene_view::SceneView;
use crate::scene::level::shadows_recorded::ShadowsRecorded;
use crate::scene::level::view_record::ViewRecord;

impl SceneView {
  /// The post group: the composited surfaces and the particles, the reflections' source for the next frame, the light
  /// volumes and the sun shafts, the rain and the thunder, the frame after its transparent layers for ray
  /// reconstruction, the flares, the fog, the engine's bloom, the distortion, the smoothing, the resolve, the enhanced
  /// bloom from the resolved image and the selection, the streaks over it and the exposure; the finished frame, the
  /// image shown and the enhanced bloom written into `output`.
  pub(super) fn record_post<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    lit_frame: LightingRecorded<'a>,
    shadowed: ShadowsRecorded,
    output: &mut SceneOutput,
  ) {
    let ViewRecord {
      passes,
      view,
      handles,
      statics,
      lit,
      ..
    } = record;
    let LightingRecorded {
      mut ray_guides,
      before_water,
      opaque,
      depth_with_water,
      source,
      sky,
      ..
    } = lit_frame;
    let is_drawn: bool = !self.shared.is_wireframe;
    let composited_draw: Option<(CompositedDraw, SkyParameters)> = sky
      .filter(|_| is_drawn && self.features.sky.get_views().is_some())
      .map(|sky| {
        (
          CompositedDraw::new(record, self.features.statics.list_draw_args(&statics)),
          sky,
        )
      });

    if let Some((draw, sky)) = &composited_draw {
      self.features.composited.add_passes(&mut *graph, passes, draw, *sky);
    }

    if is_drawn && self.features.particles.is_drawing() {
      self.features.particles.add_passes(
        GraphDeclaration::new(graph, bindings, runtime),
        record,
        scene.particles.get_surfaces(),
      );
    }

    if let Some((draw, sky)) = &composited_draw {
      self.features.composited.add_after_particles(
        GraphDeclaration::new(graph, bindings, runtime),
        passes,
        draw,
        *sky,
        lit,
      );
    }

    // What a reflection reads a frame later: the scene with the water and the particles drawn, before the sun shafts,
    // the rain and the flares, which belong to this camera and not to what it reflects. The depth it stood on is the
    // target itself, which the reflections keep at the frame's end.
    if let Some(source) = source {
      add_texture_copy(&mut *graph, "reflection source scene", handles.scene, source.scene);
    }

    // The volumetrics, the flares and the bloom, then the resolve, recorded beside the forward passes.
    graph.begin_group("effects");

    self.stages.effects.add_shafts(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      ShaftInputs {
        lights: &self.features.lights,
        atlas: scene.lights.get_shadow_atlas(),
        reflected: source.map(|source| source.scene),
      },
    );
    self.features.rain.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      RainCovers::of(&shadowed),
    );
    self
      .stages
      .effects
      .add_thunder(GraphDeclaration::new(graph, bindings, runtime), record);

    ResolveFeature::add_transparency_guide(&mut *graph, handles, &mut ray_guides, before_water);

    self
      .stages
      .effects
      .add_over_lit(GraphDeclaration::new(graph, bindings, runtime), record);

    self
      .features
      .bloom
      .add_engine_passes(GraphDeclaration::new(graph, bindings, runtime), passes, handles);

    graph.begin_group("resolve");

    self
      .features
      .present
      .add_distortion_pass(GraphDeclaration::new(graph, bindings, runtime), passes, view, handles);

    let smoothed: GraphTexture =
      self
        .stages
        .image
        .add_smoothing(GraphDeclaration::new(graph, bindings, runtime), passes, handles);
    // The water's pixels carried by its surface rather than by what lies under it.
    let surfaced: ViewTargetHandles = ViewTargetHandles {
      depth: depth_with_water.unwrap_or(handles.depth),
      ..handles
    };
    let mut chain: ImageChain = self.features.resolve.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      ViewRecord {
        handles: surfaced,
        ..record
      },
      smoothed,
      opaque,
      ray_guides,
    );

    chain = self
      .stages
      .image
      .add_camera(GraphDeclaration::new(graph, bindings, runtime), passes, surfaced, chain);

    // The enhanced bloom from the resolved image the present shows, before the streaks laid over it.
    let enhanced_bloom: Option<GraphTexture> =
      self
        .features
        .bloom
        .add_enhanced_passes(GraphDeclaration::new(graph, bindings, runtime), record, chain.shown);

    // A selected composited surface, glass, marked where the present outlines the selection, once the enhanced
    // bloom has read the marks.
    if let (Some((draw, _)), Some(_)) = (&composited_draw, self.features.overlays.get_selection_color()) {
      self.features.composited.add_selection_passes(&mut *graph, passes, draw);
    }

    // The enhanced streaks over what the resolve shows, so no history carries them.
    chain = chain.lay(self.features.rain.add_over_resolved(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      RainCovers::of(&shadowed),
      chain,
    ));

    self
      .features
      .exposure
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record);

    output.scene = chain.resolved;
    output.shown = chain.shown;
    output.indirect_light = lit_frame.indirect_light.unwrap_or(record.unlit);
    output.reflections = lit_frame.reflections.unwrap_or(record.untraced);
    output.reflection_ends = lit_frame.reflection_ends.unwrap_or(record.no_ends);
    output.bloom = enhanced_bloom.unwrap_or(handles.bloom[0]);
  }
}
