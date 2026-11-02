use glam::Vec4;
use xrf_engine_target::XrayEngine;
use xrf_renderer_core::{
  GraphColorAttachment, GraphDeclaration, GraphRuntime, GraphTexture, GraphTextureDescriptor, UniformBinding,
};

use crate::contract::feature::effects::render_rain_settings::RenderRainSettings;
use crate::feature::effects::rain::cover::rain_cover_shape::RainCoverShape;
use crate::feature::effects::rain::rain_uniform::RainUniform;
use crate::feature::effects::rain::rings::drop_rings::DropRings;
use crate::feature::view::overhead_windows::OverheadWindows;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::weather::water::enhanced::enhanced_water_maps::EnhancedWaterMaps;
use crate::feature::weather::wet::draw::wet_apply_parameters::WetApplyParameters;
use crate::feature::weather::wet::draw::wet_draw::WetDraw;
use crate::feature::weather::wet::draw::wet_pass::WetPass;
use crate::feature::weather::wet::draw::wet_patch_parameters::WetPatchParameters;
use crate::feature::weather::wet::wet_stage::WetStage;
use crate::feature::weather::wet::wet_targets::WetTargets;
use crate::feature::weather::wet::wet_uniform::WetUniform;
use crate::scene::level::view::record::shadows_recorded::ShadowsRecorded;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::level::wetness::level_wetness::LevelWetness;
use crate::scene::level::wetness::rain_wetness::RainWetness;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The wet surfaces: the G-buffer wetted by the level's wetness and its puddles' fill (`LevelWetness`, the scene's)
/// while the rain falls and, enhanced, while the level is still wet after it stops.
#[derive(Default)]
pub struct WetFeature {
  uniform: WetUniform,
  /// The splash volume and the flow, as the weather textures hold them, while the surfaces are wetted.
  views: Option<[wgpu::TextureView; 2]>,
}

impl WetFeature {
  /// Takes this frame's wetting from the level's wetness (`level`) at the scene's clock `time`: with the falling rain's
  /// uniform where it falls, over the overhead maps where they stand (`windows`), its cover's noise turned by
  /// `noise_frame`; dry for an orthographic view, which has no eye for the overhead maps to stand about.
  pub fn prepare(
    &mut self,
    &ViewPrepare {
      view,
      options,
      lighting,
      weather,
      weather_textures,
      ..
    }: &ViewPrepare<'_>,
    level: &LevelWetness,
    falling: Option<&RainUniform>,
    time: f32,
    windows: OverheadWindows,
    noise_frame: u32,
  ) {
    self.views = None;

    let has_eye: bool = !view.is_orthographic();

    let settings: RenderRainSettings = options.features.rain;
    let is_enhanced: bool = settings.is_enhanced();
    let drying: &RainWetness = level.get_wetness();
    let wetness: f32 = drying.get();
    let (fill, storm, soak): (f32, f32, f32) = level.get_puddles();
    // The traced reflections want the wet gloss as far as the rain reaches, whatever the rain's mode.
    let is_far: bool = options.features.reflections.is_drawn();

    // Puddles are the enhanced rain's alone; the surfaces dry on every engine.
    if !has_eye
      || !options.mode.is_lit
      || (falling.is_none() && wetness.max(if is_enhanced { fill } else { 0.0 }) <= 0.0)
    {
      return;
    }

    let Some(wet) = weather.and_then(|weather| weather.wet.as_ref()) else {
      return;
    };

    let RainCoverShape { texels, reach } = windows.cover_shape;

    self.uniform = WetUniform {
      density: falling.map_or(0.0, |_| lighting.rain.map_or(0.0, |rain| rain.density.clamp(0.0, 1.0))),
      time: falling.map_or(time, |uniform| uniform.time),
      is_extended: (lighting.engine == XrayEngine::Extended) as u32 as f32,
      is_enhanced: f32::from(u8::from(is_enhanced)),
      cover: windows.cover,
      cover_state: Vec4::new(
        texels,
        reach,
        WetUniform::CREEP_MOST * soak,
        WetUniform::get_cover_noise(noise_frame),
      ),
      surface: windows.surface,
      surface_shape: windows.surface_shape,
      water: windows.water,
      puddles: Vec4::new(wetness, settings.puddles, settings.reflectivity, settings.ripples),
      puddle_state: Vec4::new(settings.distance.max(0.0), fill, storm, soak),
      reach: Vec4::new(
        f32::from(u8::from(is_far)),
        options.features.reflections.reach.max(0.0),
        0.0,
        0.0,
      ),
      gust: WetUniform::get_gust(lighting.wind.direction, lighting.wind.velocity),
      relief: Vec4::new(f32::from(u8::from(settings.is_relief_flooded)), 0.0, 0.0, 0.0),
      drying: Vec4::new(drying.get_peak(), drying.get_left(), 0.0, 0.0),
      rings: Vec4::ZERO,
    };
    self.views = Some([
      weather_textures
        .get_view(Some(&wet.splash), WeatherTextureKind::Volume)
        .clone(),
      weather_textures
        .get_view(Some(&wet.flow), WeatherTextureKind::Flat)
        .clone(),
    ]);
  }

  /// Follows the enhanced drops' rings this frame.
  pub fn follow_rings(&mut self, rings: DropRings) {
    self.uniform.rings = Vec4::new(rings.tile, rings.reach, rings.strength, 0.0);
  }

  /// Follows the overhead maps to where they stand once they have moved, as the wet surfaces read them.
  pub fn follow(&mut self, windows: OverheadWindows) {
    let RainCoverShape { texels, reach } = windows.cover_shape;

    self.uniform.cover = windows.cover;
    self.uniform.cover_state.x = texels;
    self.uniform.cover_state.y = reach;
    self.uniform.surface = windows.surface;
    self.uniform.surface_shape = windows.surface_shape;
    self.uniform.water = windows.water;
  }

  /// Whether the surfaces are wetted this frame.
  pub fn is_wet(&self) -> bool {
    self.views.is_some()
  }

  /// Whether the enhanced wetting draws this frame, its puddles seated on the level's surface.
  pub fn is_enhanced(&self) -> bool {
    self.is_wet() && self.uniform.is_enhanced > 0.5
  }

  /// Whether puddles stand this frame, which no tuft is planted in.
  pub fn has_puddles(&self) -> bool {
    self.is_enhanced() && self.uniform.puddle_state.y > 0.0
  }

  /// How far the rain cover reaches for the traced reflections' wet gloss, where it reaches past the engine's.
  pub fn get_reach(&self) -> Option<f32> {
    (self.is_wet() && self.uniform.reach.x > 0.5).then_some(self.uniform.reach.y)
  }

  /// Pushes what its passes and the level surface's read, while the surfaces are wetted.
  pub fn push_uniform(&self, runtime: &mut GraphRuntime) -> Option<UniformBinding<WetUniform>> {
    self.is_wet().then(|| runtime.push_uniform(&self.uniform))
  }

  /// Adds the wetting of the G-buffer before any light is drawn over it, through the rain's cover and the level's
  /// surface (its lowest heights, its puddle sites and its water) as the shadows phase drew them, the puddles rippled
  /// by the enhanced drops' `rings`; answers what it writes for the reflections and combine.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes, view, handles, ..
    }: ViewRecord<'a>,
    ShadowsRecorded {
      covers,
      wet,
      surface: [surface_lowest, puddles, surface_water],
    }: ShadowsRecorded,
    rings: GraphTexture,
  ) -> WetTargets {
    let (Some(covers), Some(wet), Some([splash, flow])) = (covers, wet, &self.views) else {
      let none: &'a wgpu::TextureView = passes.tracing.reflections.get_empty();

      return WetTargets {
        surface: bindings.import_view(&mut *graph, "wet surface none", none),
        reach: bindings.import_view(&mut *graph, "wet reach none", none),
      };
    };
    let is_enhanced: bool = self.uniform.is_enhanced > 0.5;
    let maps: &EnhancedWaterMaps = passes.weather.water.get_maps();
    let (width, height) = handles.size;
    let wet_reach: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "wet reach",
      width,
      height,
      WetPass::REACH,
    ));
    let wet_surface: GraphTexture = if is_enhanced {
      graph.create_texture(GraphTextureDescriptor::new_2d(
        "wet surface",
        width,
        height,
        WetPass::SURFACE,
      ))
    } else {
      bindings.import_view(&mut *graph, "wet surface none", passes.tracing.reflections.get_empty())
    };
    let patch: WetPatchParameters = WetPatchParameters {
      depth_target: handles.depth,
      albedo_target: handles.albedo,
      normal_target: handles.normal,
      material_target: handles.material,
      relief_target: handles.relief,
      cover: covers.depth,
      cover_pyramid: covers.pyramid,
      surface_lowest,
      puddles,
      surface_water,
      splash: bindings.import_view(&mut *graph, "wet splash", splash),
      flow: bindings.import_view(&mut *graph, "wet flow", flow),
      ripples: bindings.import_view(&mut *graph, "wet ripples", &maps.ripples),
      puddle_noise: bindings.import_view(&mut *graph, "puddle noise", &maps.perlin),
      puddle_normal: bindings.import_view(&mut *graph, "puddle normal", &maps.normal),
      rain_rings: rings,
      wet_sampler: passes.weather.wet.get_sampler(),
      wet,
    };
    let apply: WetApplyParameters = WetApplyParameters {
      depth_target: handles.depth,
      material_target: handles.material,
      patched: handles.light,
      wet_surface,
      wet,
    };

    // The patches go into the light, which the wet look over the normals and the albedo then reads; the enhanced
    // patch's wet surface beside them.
    let builder = graph
      .add_raster_pass("wet patch")
      .parameters(&patch)
      .color(GraphColorAttachment::new(handles.light, wgpu::LoadOp::Load));
    let builder = if is_enhanced {
      builder.color(GraphColorAttachment::new(
        wet_surface,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
    } else {
      builder
    };

    builder
      .color(GraphColorAttachment::new(
        wet_reach,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .record(move |context| {
        let draw: WetDraw = WetDraw {
          stage: WetStage::Patch,
          is_enhanced,
        };

        passes.weather.wet.record(context, draw, view, &patch)
      });

    graph
      .add_raster_pass("wet apply")
      .parameters(&apply)
      .color(GraphColorAttachment::new(handles.normal, wgpu::LoadOp::Load))
      .color(GraphColorAttachment::new(handles.albedo, wgpu::LoadOp::Load))
      .record(move |context| {
        let draw: WetDraw = WetDraw {
          stage: WetStage::Apply,
          is_enhanced,
        };

        passes.weather.wet.record(context, draw, view, &apply)
      });

    WetTargets {
      surface: wet_surface,
      reach: wet_reach,
    }
  }
}

impl ViewFeature for WetFeature {
  /// Its targets are the frame's own; the textures are the weather's.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {
    self.views = None;
  }
}
