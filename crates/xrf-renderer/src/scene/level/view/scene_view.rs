use std::sync::Arc;
use std::time::Instant;

use xrf_renderer_core::{GraphDeclaration, GraphTexture};

use crate::contract::view::render_view_options::RenderViewOptions;
use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::effects::rain::cover::rain_covers::RainCovers;
use crate::feature::image::exposure::reading::exposure_decisions::ExposureDecisions;
use crate::feature::image::present::display::display_mode::DisplayMode;
use crate::feature::scene::drawable::draw::drawable_kind_readied::DrawableKindReadied;
use crate::feature::scene::wind::wind_feature::WindFeature;
use crate::feature::view::scene_features::SceneFeatures;
use crate::feature::view::view_frame_end::ViewFrameEnd;
use crate::feature::view::view_stages::ViewStages;
use crate::feature::weather::wet::wet_targets::WetTargets;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::readback::pick_pass::PickPass;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::frame::targets::view_target_uses::ViewTargetUses;
use crate::frame::targets::view_targets::ViewTargets;
use crate::frame::timing::frame_clock::FrameClock;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::weather::render_level_weather::RenderLevelWeather;
use crate::lighting::weather::render_lighting::RenderLighting;
use crate::pass::frame_lighting::lighting_frame::LightingFrame;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::level_prepare::LevelPrepare;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::lights::asks::light_asks::LightAsks;
use crate::scene::level::page::helpers::view::helper_marks_pass::HelperMarksPass;
use crate::scene::level::shadow::shadow_sway::ShadowSway;
use crate::scene::level::view::record::lighting_recorded::LightingRecorded;
use crate::scene::level::view::record::readied_frame::ReadiedFrame;
use crate::scene::level::view::record::shadows_recorded::ShadowsRecorded;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::scene_asks::SceneAsks;
use crate::scene::level::view::scene_output::SceneOutput;
use crate::scene::level::view::view_frame::ViewFrame;
use crate::scene::level::view::view_pick::ViewPick;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

mod scene_view_controls;
mod scene_view_overlays;
mod scene_view_periodic;
mod scene_view_picks;
mod scene_view_post;
mod scene_view_prepare;
mod scene_view_record;
mod scene_view_reports;

/// A scene as one viewport draws it: the values its units share this frame (`ViewFrame`), its targets, its units shared
/// across its frame's parts (`SceneFeatures`) and its stages (`ViewStages`), whose units draw in one part alone; each
/// unit keeps what it keeps between frames. The scene (`LevelScene`) is shared by every view of it and passed in; this
/// prepares the view's frame from it and records it into the frame's graph.
pub struct SceneView {
  /// The values every unit reads this frame.
  shared: ViewFrame,
  /// The G-buffer, depth, light and scene it draws into, once its size is known.
  targets: Option<ViewTargets>,
  /// Its units shared across its frame's parts.
  features: SceneFeatures,
  /// Its units drawing in one part of its frame alone, by part.
  stages: ViewStages,
  /// The time its frames are drawn at: its scene's, or a step a frame of its own.
  clock: FrameClock,
}

impl SceneView {
  /// A view of a scene, nothing of it drawn yet.
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, view_layout: &wgpu::BindGroupLayout) -> Self {
    Self {
      shared: ViewFrame::default(),
      targets: None,
      features: SceneFeatures::new(device, queue, view_layout),
      stages: ViewStages::new(),
      clock: FrameClock::default(),
    }
  }

  /// Asks for the textures its weather draws with: the rain's, the bolts', the flares' and the sky's.
  pub fn request_textures(
    &mut self,
    scene: &LevelScene,
    weather_textures: &mut WeatherTextureCache,
    lighting: &RenderLighting,
    weather: Option<&Arc<RenderLevelWeather>>,
    options: &RenderViewOptions,
  ) {
    let assets: Arc<dyn RenderAssetSource> = Arc::clone(&scene.source) as Arc<dyn RenderAssetSource>;

    if let Some(rain) = weather.and_then(|weather| weather.rain.as_ref())
      && lighting.rain.is_some()
      && options.mode.is_lit
    {
      weather_textures.request(&rain.streak, WeatherTextureKind::Flat, &assets);

      if let Some(drop) = &rain.drop {
        weather_textures.request(&drop.texture, WeatherTextureKind::Flat, &assets);
      }

      if let Some(wet) = weather.and_then(|weather| weather.wet.as_ref()) {
        weather_textures.request(&wet.splash, WeatherTextureKind::Volume, &assets);
        weather_textures.request(&wet.flow, WeatherTextureKind::Flat, &assets);
      }
    }

    // Every bolt's textures are held while the level's weather has bolts, so a strike has them.
    if let Some(thunder) = weather.and_then(|weather| weather.thunder.as_ref())
      && options.mode.is_lit
    {
      let references = thunder.models.iter().map(|model| model.mesh.texture.as_str()).chain(
        thunder
          .bolts
          .values()
          .flat_map(|bolt| [bolt.top.texture.as_str(), bolt.center.texture.as_str()]),
      );

      for reference in references.filter(|reference| !reference.is_empty()) {
        weather_textures.request(reference, WeatherTextureKind::Flat, &assets);
      }
    }

    if options.mode.is_lit && options.show.is_sky_visible {
      self
        .stages
        .effects
        .request_textures(lighting, weather, weather_textures, &assets);
      weather_textures.request_sky(&lighting.sky, options.show.is_clouded, &assets);
    } else {
      // The irradiance cubes light the hemisphere whether or not the sky is drawn.
      for reference in lighting.sky.environments.iter().flatten() {
        weather_textures.request(reference, WeatherTextureKind::Cube, &assets);
      }
    }
  }

  /// Sizes the targets to the viewport and readies every unit's frame, in the order each reads what an earlier one
  /// decided: the resolve's decisions, the weather and the sky, the scene, the view's own units, then the lighting.
  /// Answers what it asks of the scene, which settles every view's asks before each finishes its frame
  /// (`finish_prepare`).
  pub fn prepare(&mut self, scene: &LevelScene, encoder: &mut FrameEncoder, frame: &ViewPrepare<'_>) -> SceneAsks {
    profiling::scope!("prepare view");

    let (width, height) = frame.drawn;

    if !self.targets.as_ref().is_some_and(|it| it.is_sized(width, height)) {
      self.targets = Some(ViewTargets::new(frame.device, width, height));
      self.features.statics.fit(frame.device, (width, height));
    }

    let level: LevelPrepare = LevelPrepare::of(scene, frame.view);
    let now: Instant = self.clock.tick(scene.get_now());

    self.decide(frame, now);

    let lighting_frame: LightingFrame = self.prepare_weather(scene, frame, &level, now);

    self
      .features
      .overlays
      .follow_skeletons(scene, frame.device, &mut *encoder);

    let lights: LightAsks = self.prepare_scene(scene, frame, &lighting_frame, &level);

    self.prepare_view(scene, encoder, frame, &level);
    self.prepare_lighting(frame, &lighting_frame, &level);

    SceneAsks { lights }
  }

  /// Finishes readying its frame once its scene settled every view's asks (`LevelScene::settle`): writes its lights as
  /// it asked for them, the faces now settled, and fills its effects' quads from the step the scene joined; then, every
  /// unit knowing what it draws, keeps the engine's bloom for a distorting frame's blur and notes what the window pass
  /// reads, with `options` and over `output` as its frame was prepared.
  pub fn finish_prepare(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    options: &RenderViewOptions,
    output: RenderRect,
    asks: &SceneAsks,
  ) {
    profiling::scope!("finish view");

    scene
      .lights
      .write(&mut *encoder, &mut self.features.lights, &asks.lights);
    scene
      .particles
      .fill(&self.shared.camera, options, &mut self.features.particles);
    self.features.particles.upload(device, &mut *encoder);

    let distortion: ScreenTiles = self.get_distortion_tiles(options);

    self.features.bloom.keep_for_distortion(!distortion.is_empty());
    self.prepare_present(options, output, self.shared.resolve.noise_frame, distortion);
  }

  /// What every other unit reads of the frame's exposure and resolve, prepared before any of them, which the view's frame
  /// holds: the exposure, whose scale the resolve carries its history by and whose metering the present shows the frame
  /// by, then the resolve.
  fn decide(&mut self, frame: &ViewPrepare<'_>, now: Instant) {
    let options: &RenderViewOptions = frame.options;
    let exposure: ExposureDecisions = self.features.exposure.prepare(frame.queue, options, now);

    self.shared.exposure = exposure;
    self.shared.display = DisplayMode::of(options).get_display(&options.features.grading, &exposure.metering);
    self.shared.resolve = self.features.resolve.prepare(
      frame,
      now,
      self.targets.as_ref().map(|targets| (targets.width, targets.height)),
      exposure.pre_exposure,
    );
  }

  /// Ends a frame its graph recorded: every unit's history it wrote becomes the one the next frame keeps.
  pub fn end_frame(&mut self) {
    let mut end: ViewFrameEnd<'_> = ViewFrameEnd {
      depth: self.targets.as_mut().map(|targets| &mut targets.depth),
    };

    for feature in self.features.list_mut().into_iter().chain(self.stages.list_mut()) {
      feature.end_frame(&mut end);
    }
  }

  /// Readies a frame for its graph, none before the view's targets are made: a pick's camera narrowed to the texel
  /// picked and the readback it is copied into while one is free, and the depth the next frame's occlusion tests.
  pub fn begin_frame(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    pick: Option<ViewPick<'_>>,
  ) -> Option<ReadiedFrame> {
    if self.targets.is_none() || !self.features.statics.is_ready() {
      return None;
    }

    let pick_slot: Option<usize> =
      pick.and_then(|pick| self.features.pick.ready(device, encoder, pick.camera, pick.view_layout));

    self.features.statics.begin_frame();

    Some(ReadiedFrame {
      pick_slot,
      is_pick_through: pick.is_some_and(|pick| pick.is_through),
      is_scene_recorder: false,
    })
  }

  /// Declares the frame `begin_frame` readied into the frame's graph, in the encode groups the frame is recorded in, on
  /// as many threads where it is not timed: the scene (culled and drawn into the G-buffer, the drawable kinds
  /// readied and drawn), its shadows (the cascades, the rain's cover, the level's surface and the lamps' faces), its lighting with
  /// the reflections and the water, and what blends over it and resolves it; the pick last. Which passes run is decided
  /// by the units; each reads the view, which recording leaves as it is. The present pass reads what it leaves.
  pub fn record<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    textures: &'a TextureCache,
    frame: &ReadiedFrame,
  ) -> Option<SceneOutput> {
    profiling::scope!("declare");

    let targets: &'a ViewTargets = self.targets.as_ref()?;
    let uses: ViewTargetUses = ViewTargetUses {
      is_occluded: self.stages.lighting.is_occluded(),
      is_hazed: self.features.sky.is_hazing(),
      is_engine_bloomed: self.features.bloom.is_engine_built(),
      is_specular_split: self.shared.lighting.is_light_anchored(),
    };
    let handles: ViewTargetHandles = ViewTargetHandles::import(
      &mut *graph,
      &mut *bindings,
      targets,
      uses,
      passes.lighting.vbao.get_dark(),
    );
    let statics: StaticSceneHandles = self.features.statics.import(
      GraphDeclaration::new(graph, bindings, runtime),
      &scene.statics,
      self.features.wind.get_uniform(),
    )?;
    let record: ViewRecord<'a> = ViewRecord {
      passes,
      view,
      textures: textures.get_bind_group(),
      handles,
      statics,
      cull: self.features.statics.get_cull(),
      lit: self.import_lighting(GraphDeclaration::new(graph, bindings, runtime), passes),
      unlit: bindings.import_view(&mut *graph, "indirect light none", passes.lighting.vbao.get_dark()),
      untraced: bindings.import_view(
        &mut *graph,
        "reflections none",
        passes.tracing.reflections.get_untraced(),
      ),
      no_ends: bindings.import_view(
        &mut *graph,
        "reflection ends none",
        passes.tracing.reflections.get_no_ends(),
      ),
    };

    graph.begin_group("scene");

    let kinds: Vec<DrawableKindReadied<'a>> =
      self.record_scene(scene, GraphDeclaration::new(graph, bindings, runtime), record);

    graph.begin_group("shadows");

    let shadowed: ShadowsRecorded = self.record_shadows(
      scene,
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      &kinds,
      frame,
    );

    graph.begin_group("lighting");

    // The rings the enhanced drops leave where they land, while the puddles or the enhanced water ripple by them.
    let is_rippled: bool = self.features.wet.has_puddles() || self.features.water.is_enhanced();
    let rings: GraphTexture = is_rippled
      .then(|| {
        self.features.rain.add_rings(
          GraphDeclaration::new(graph, bindings, runtime),
          record,
          RainCovers::of(&shadowed),
        )
      })
      .flatten()
      .unwrap_or_else(|| bindings.import_view(&mut *graph, "rain rings none", passes.effects.rain.get_nothing()));
    let wet: WetTargets =
      self
        .features
        .wet
        .add_passes(GraphDeclaration::new(graph, bindings, runtime), record, shadowed, rings);
    let mut output: SceneOutput = SceneOutput {
      targets: handles,
      scene: handles.scene,
      shown: handles.scene,
      present: runtime.push_uniform(self.features.present.get_uniform()),
      lighting: record.lit.lighting,
      indirect_light: record.unlit,
      reflections: record.untraced,
      reflection_ends: record.no_ends,
      bloom: handles.bloom[0],
      marks: bindings.import_view(&mut *graph, "helper marks none", passes.scene.gbuffer.get_unmarked()),
    };

    let lit: LightingRecorded<'a> = self.record_lighting(
      scene,
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      &kinds,
      wet,
      rings,
    );

    graph.begin_group("post");

    self.record_post(
      scene,
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      lit,
      shadowed,
      &mut output,
    );

    if let Some(marks) = self.features.helpers.add_marks_pass(
      &mut *graph,
      HelperMarksPass {
        gbuffer: &passes.scene.gbuffer,
        view,
        textures: record.textures,
        scene: &statics,
        handles,
      },
    ) {
      output.marks = marks;
    }

    if let Some(slot) = frame.pick_slot {
      self.features.pick.add_passes(
        &mut *graph,
        &mut *bindings,
        PickPass {
          gbuffer: &passes.scene.gbuffer,
          pipelines: passes.pipelines,
          textures: record.textures,
          scene: &statics,
          helpers: self.features.helpers.get_draws(),
          slot,
          is_through: frame.is_pick_through,
        },
      );
    }

    Some(output)
  }

  /// The sway the shadows see this frame: the `wind`, the trees' reach, and where they stand.
  fn to_sway<'s>(scene: &'s StaticScene, wind: &WindFeature) -> ShadowSway<'s> {
    let (amplitude, time): (f32, f32) = wind.get_sway();

    ShadowSway {
      amplitude,
      reach: scene.get_sway_reach(),
      time,
      places: scene.list_swaying(),
    }
  }
}
