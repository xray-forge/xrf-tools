use std::sync::Arc;
use std::time::Instant;

use xrf_renderer_core::GraphDeclaration;

use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::contract::report::scene::render_meter_reading::RenderMeterReading;
use crate::feature::effects::flares::flares_feature::FlaresFeature;
use crate::feature::effects::flares::sun_sprite::SunSprite;
use crate::feature::effects::thunder::thunder_feature::ThunderFeature;
use crate::feature::image::meter::meter_feature::MeterFeature;
use crate::feature::view::view_units::view_units;
use crate::feature::weather::fog::fog_feature::FogFeature;
use crate::feature::weather::sun_shafts::sun_shafts_feature::SunShaftsFeature;
use crate::feature::weather::volumetric::shaft_inputs::ShaftInputs;
use crate::feature::weather::volumetric::volumetric_feature::VolumetricFeature;
use crate::feature::weather::volumetric::volumetric_shafts::VolumetricShafts;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::weather::render_level_weather::RenderLevelWeather;
use crate::lighting::weather::render_lighting::RenderLighting;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;

/// The units laid over a view's lit frame before it is resolved, each alone in that part of it: the light volumes and
/// the sun shafts, the thunder's strike, the lens flares, the scattering fog, and the spot meter reading the frame
/// they leave. The composited surfaces, the particles, the rain and the engine's bloom record between them.
pub struct EffectStage {
  volumetric: VolumetricFeature,
  sun_shafts: SunShaftsFeature,
  thunder: ThunderFeature,
  flares: FlaresFeature,
  fog: FogFeature,
  meter: MeterFeature,
}

impl EffectStage {
  /// The stage, nothing of it drawn yet.
  pub fn new() -> Self {
    Self {
      volumetric: VolumetricFeature::default(),
      sun_shafts: SunShaftsFeature::default(),
      thunder: ThunderFeature::default(),
      flares: FlaresFeature::new(),
      fog: FogFeature::default(),
      meter: MeterFeature::default(),
    }
  }

  /// Asks for the textures of the lens flares the `weather` shows now and fades to.
  pub fn request_textures(
    &self,
    lighting: &RenderLighting,
    weather: Option<&Arc<RenderLevelWeather>>,
    weather_textures: &mut WeatherTextureCache,
    assets: &Arc<dyn RenderAssetSource>,
  ) {
    self.flares.request(lighting, weather, weather_textures, assets);
  }

  /// Takes this frame's units at the view's clock `now` and the scene's `time`: the flares, the strike, the shafts and
  /// the light volumes, the fog over the drawn frame and the meter. Answers the sun's sprite as the sky draws it, none
  /// where no sprite is drawn.
  pub fn prepare(&mut self, frame: &ViewPrepare<'_>, now: Instant, time: f32) -> Option<SunSprite> {
    let (device, options, lighting) = (frame.device, frame.options, frame.lighting);
    let sun_sprite: Option<SunSprite> = self.flares.prepare(frame, now);

    self.thunder.prepare(frame);
    self.sun_shafts.prepare(options, lighting);
    self.fog.prepare(options, lighting, Some(frame.drawn), time);
    self.volumetric.prepare(device, options);
    self.meter.prepare(device, options);

    sun_sprite
  }

  /// Writes into `applied` what it draws this frame: the enhanced height fog.
  pub fn describe(&self, applied: &mut RenderAppliedReport) {
    applied.height_fog = self.fog.get_applied();
  }

  /// The lens flare it shows, none while it shows none.
  pub fn get_shown_flare(&self) -> Option<&str> {
    self.flares.get_shown()
  }

  /// Asks for the spot meter's reading recorded with the frame just submitted.
  pub fn request_readbacks(&self) {
    self.meter.request();
  }

  /// Drops the reading a frame that was never submitted copied out.
  pub fn abandon_frame(&self) {
    self.meter.abandon();
  }

  /// What the spot meter last read; none while the view does not meter.
  pub fn take_meter_reading(&mut self) -> Option<RenderMeterReading> {
    self.meter.take_reading()
  }

  /// Adds the shafts: the shadowed lights' volumes, or the enhanced froxels in place of them, then the sun's shafts
  /// composited from them over the scene.
  pub fn add_shafts<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    ShaftInputs {
      lights,
      atlas,
      reflected,
    }: ShaftInputs<'a>,
  ) {
    let shafts: VolumetricShafts =
      self
        .volumetric
        .add_passes(GraphDeclaration::new(graph, bindings, runtime), record, lights, atlas);

    self.sun_shafts.add_passes(
      GraphDeclaration::new(graph, bindings, runtime),
      record,
      shafts,
      reflected,
    );
  }

  /// Adds the thunder's strike over the scene, while a bolt strikes.
  pub fn add_thunder<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
  ) {
    self
      .thunder
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record);
  }

  /// Adds what is laid over the frame after its transparent layers: the lens flares and the scattering fog, then the
  /// spot meter reading what they leave.
  pub fn add_over_lit<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
  ) {
    self
      .flares
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record);
    self
      .fog
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record);
    self
      .meter
      .add_passes(GraphDeclaration::new(graph, bindings, runtime), record);
  }
}

view_units!(EffectStage {
  volumetric,
  sun_shafts,
  thunder,
  flares,
  fog,
  meter,
});
