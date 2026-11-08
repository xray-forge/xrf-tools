use std::sync::Arc;

use glam::{Vec2, Vec3, Vec4};
use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, StorageArray, StorageValue};

use crate::contract::feature::effects::render_rain_settings::RenderRainSettings;
use crate::frame::jitter_guard::JitterGuard;

use crate::feature::effects::over_scene::add_over_scene;
use crate::feature::effects::rain::cover::rain_cover_shape::RainCoverShape;
use crate::feature::effects::rain::cover::rain_covers::RainCovers;
use crate::feature::effects::rain::draw::rain_counts::RainCounts;
use crate::feature::effects::rain::draw::rain_parameters::RainParameters;
use crate::feature::effects::rain::enhanced::rain_look_parameters::RainLookParameters;
use crate::feature::effects::rain::rain_frame::RainFrame;
use crate::feature::effects::rain::rain_uniform::RainUniform;
use crate::feature::effects::rain::rings::rain_rings::RainRings;
use crate::feature::effects::rain::rings::rain_rings_draw::RainRingsDraw;
use crate::feature::effects::weather_built::WeatherBuilt;
use crate::feature::effects::weather_model_buffers::WeatherModelBuffers;
use crate::feature::image::image_chain::ImageChain;
use crate::feature::view::overhead_windows::OverheadWindows;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::view::view_restart::ViewRestart;
use crate::host::weather::render_level_weather::RenderLevelWeather;
use crate::host::weather::render_rain::RenderRain;
use crate::lighting::weather::render_rainfall::RenderRainfall;
use crate::lighting::wind::ambient_gust::AmbientGust;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::level::view::view_time::ViewTime;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The falling rain, while the weather rains and the view shows it: the splash's model built for the level's weather,
/// the uniform at the rain cover's window, the streak's and the splash's textures, and the drops drawn.
#[derive(Default)]
pub struct FallingRainFeature {
  splash: Option<WeatherBuilt<WeatherModelBuffers>>,
  uniform: RainUniform,
  /// The streak's and the splash's textures, as the cache holds them when the frame is prepared.
  views: Option<[wgpu::TextureView; 2]>,
  /// How much of the rain is drawn; none while it does not rain.
  draw: Option<RainCounts>,
  /// Where the enhanced drops' square stood from the eye last frame, eased towards the camera's way.
  aim: Option<Vec2>,
  /// The rain's clock in seconds, run on at its pace each frame, in `f64` so a frame's step still adds to a day's.
  clock: f64,
  /// The pace and the lean it fell at last frame, eased towards the weather's.
  eased: Option<(f32, Vec3)>,
  /// The rings the enhanced drops leave where they land, made once they first fall.
  rings: Option<RainRings>,
}

impl FallingRainFeature {
  /// Takes this frame's rain where the weather rains on a lit view, as the rain's settings say, blown by `gust` at
  /// `time`: the splash's model built for the level's weather, the uniform at the rain cover's window and shape (in
  /// `windows`), and the textures; answers the uniform, which the wet surfaces follow, none where the weather has no
  /// rain to draw.
  pub fn prepare(
    &mut self,
    &ViewPrepare {
      device,
      view,
      options,
      lighting,
      weather,
      weather_textures,
      ..
    }: &ViewPrepare<'_>,
    gust: AmbientGust,
    time: ViewTime,
    windows: OverheadWindows,
  ) -> Option<RainUniform> {
    let rainfall: Option<RenderRainfall> = lighting.rain.filter(|_| options.mode.is_lit);
    let settings: &RenderRainSettings = &options.features.rain;
    let look: Vec3 = view.get_look();
    let (cover, RainCoverShape { texels, reach }) = (windows.cover, windows.cover_shape);

    self.draw = None;
    self.views = None;

    // Held within a tenth of a second and never back, so a pause or a jump does not swing the drops' square.
    let step: f32 = time.get_step().map_or(0.0, |step| step.clamp(0.0, 0.1) as f32);
    let aim: Vec2 = RainUniform::ease_aim(self.aim, look, step);

    self.aim = Some(aim);

    let rainfall: RenderRainfall = rainfall?;
    let (weather, rain): (&Arc<RenderLevelWeather>, &RenderRain) =
      weather.and_then(|weather| weather.rain.as_ref().map(|rain: &RenderRain| (weather, rain)))?;

    let splash: &WeatherModelBuffers = WeatherBuilt::follow(&mut self.splash, weather, || {
      WeatherModelBuffers::new(device, rain.drop.as_ref())
    });
    let target: RainUniform = RainUniform::new(RainFrame {
      rain: &rainfall,
      drops: settings.drops,
      wind: lighting.wind,
      strength: gust.strength,
      window: cover,
      cover_shape: Vec4::new(texels, reach, 0.0, 0.0),
      aim,
      splash_indices: splash.index_count,
    })
    .ringed(settings.ring_size)
    .squared(settings.drop_square);
    let (pace, axis): (f32, Vec3) = self
      .eased
      .map_or((target.pace, target.axis.truncate()), |(pace, axis)| {
        let following: f32 = RainUniform::get_following(step);

        (
          pace + (target.pace - pace) * following,
          axis.lerp(target.axis.truncate(), following).normalize_or(Vec3::NEG_Y),
        )
      });

    self.eased = Some((pace, axis));
    self.clock += f64::from(step * pace);

    let uniform: RainUniform = target.clocked(LevelTime::wrap(self.clock, RainUniform::CLOCK_PERIOD), pace, axis);
    let flat = |reference: Option<&str>| weather_textures.get_view(reference, WeatherTextureKind::Flat).clone();

    if uniform.is_enhanced != 0 {
      let size: u32 = RainRings::get_size(uniform.near);

      if self.rings.as_ref().is_none_or(|rings| rings.size != size) {
        self.rings = Some(RainRings::new(device, size));
      }
    }

    self.uniform = uniform;
    self.views = Some([
      flat(Some(&rain.streak)),
      flat(rain.drop.as_ref().map(|drop| drop.texture.as_str())),
    ]);
    self.draw = Some(RainCounts {
      streaks: uniform.count,
      near: uniform.near_count,
      splash_indices: splash.index_count,
    });

    Some(uniform)
  }

  /// The rain's clock in seconds, as it last fell: what the wet surfaces' splashes, flow and ripples run by while it
  /// rains.
  pub fn get_clock(&self) -> f64 {
    self.clock
  }

  /// The tile the enhanced drops' rings wrap over this frame (`RainUniform::get_rings`), none while none are drawn.
  pub fn get_rings(&self) -> f32 {
    match self.draw {
      Some(_) => self.uniform.get_rings(),
      None => 0.0,
    }
  }

  /// Whether the rain falls this frame.
  pub fn is_raining(&self) -> bool {
    self.draw.is_some()
  }

  /// Whether the enhanced splashes write into the distortion target this frame, bending the ground under them.
  pub fn is_distorting(&self) -> bool {
    self.uniform.is_enhanced != 0 && self.draw.is_some_and(|counts| counts.splash_indices > 0)
  }

  /// Follows the rain cover to where it stands once it has moved, as the drops read it.
  pub fn follow_cover(&mut self, windows: OverheadWindows) {
    let RainCoverShape { texels, reach } = windows.cover_shape;

    self.uniform.window = windows.cover;
    self.uniform.cover_shape = Vec4::new(texels, reach, 0.0, 0.0);
  }

  /// Follows how the frame is shown (`DisplayMode::get_display`), which the streaks and the engine's splashes enter the
  /// scene through.
  pub fn follow_display(&mut self, display: Vec4) {
    self.uniform.display = display;
  }

  /// Follows the level water's and the level surface's maps where `windows` gives them, none where the surfaces are not
  /// drawn, which the enhanced drops land on.
  pub fn follow_surfaces(&mut self, windows: Option<OverheadWindows>) {
    let landing: Option<OverheadWindows> = windows.filter(|_| self.uniform.is_enhanced != 0);

    self.uniform.water = landing.map_or(Vec4::ZERO, |it| it.water);
    self.uniform.surface = landing.map_or(Vec4::ZERO, |it| it.surface);
    self.uniform.water_shape = landing.map_or(Vec4::ZERO, |it| it.surface_shape);
  }

  /// Adds the rain over the scene, falling onto `covers`: the engine's streaks and splashes, or the enhanced splashes
  /// and the distortion under the enhanced streaks, which draw after the resolve (`add_over_resolved`).
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    covers: RainCovers,
  ) {
    let ViewRecord {
      passes, view, handles, ..
    } = record;
    let Some((counts, parameters)) =
      self.get_parameters(GraphDeclaration::new(graph, bindings, runtime), passes, covers, None)
    else {
      return;
    };
    // The enhanced splashes bend the ground and the streaks undo the water's distortion under them, which the window
    // pass would bend the drops drawn after the resolve by; neither reads a blurred image.
    let look: Option<RainLookParameters<'a>> = (self.uniform.is_enhanced != 0).then(|| {
      let nothing: GraphTexture = bindings.import_view(&mut *graph, "rain nothing", passes.effects.rain.get_nothing());

      self.get_look(GraphDeclaration::new(graph, bindings, runtime), record, nothing)
    });
    let mut pass = add_over_scene(&mut *graph, handles, "rain").parameters(&parameters);

    if let Some(look) = &look {
      pass = pass
        .parameters(look)
        .color(GraphColorAttachment::new(handles.distortion, wgpu::LoadOp::Load));
    }

    pass.record(move |context| {
      passes
        .effects
        .rain
        .record(context, view, &parameters, look.as_ref(), counts);
    });
  }

  /// Adds the enhanced streaks over a copy of the image the resolve shows (`chain`), so no history holds them; answers
  /// the copy, which the window pass shows in its place, none without enhanced rain.
  pub fn add_over_resolved<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    covers: RainCovers,
    ImageChain {
      shown,
      shown_size: size,
      ..
    }: ImageChain,
  ) -> Option<GraphTexture> {
    let ViewRecord { passes, view, .. } = record;

    if self.uniform.is_enhanced == 0 {
      return None;
    }

    let (RainCounts { streaks, .. }, parameters) = self.get_parameters(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      covers,
      Some(size),
    )?;
    let layer: GraphTexture = passes.effects.rain.add_layer(&mut *graph, shown, size);
    // `phase_ssfx_rain`: what the drops bend, the image as shown, blurred small.
    let refracted: GraphTexture = passes.effects.rain.add_blur(&mut *graph, shown, size);
    let look: RainLookParameters<'a> =
      self.get_look(GraphDeclaration::new(graph, bindings, runtime), record, refracted);

    // The streaks read the depth behind them and the light about them bilinearly over four drawn texels.
    graph
      .add_raster_pass("rain")
      .tag(JitterGuard::READS_DEPTH)
      .tag(JitterGuard::READS_LIGHT)
      .color(GraphColorAttachment::new(layer, wgpu::LoadOp::Load))
      .parameters(&parameters)
      .parameters(&look)
      .record(move |context| {
        passes
          .effects
          .rain
          .record_over(context, view, &parameters, &look, streaks);
      });

    Some(layer)
  }

  /// Adds the rings the enhanced drops leave where they land on `covers`; answers them, none without them.
  pub fn add_rings<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    ViewRecord { passes, view, .. }: ViewRecord<'a>,
    covers: RainCovers,
  ) -> Option<GraphTexture> {
    let rings: &RainRings = self.rings.as_ref().filter(|_| self.uniform.get_rings() > 0.0)?;
    let (RainCounts { near, .. }, parameters) =
      self.get_parameters(GraphDeclaration::new(graph, bindings, runtime), passes, covers, None)?;
    let levels: [GraphTexture; RainRings::LEVELS as usize] =
      std::array::from_fn(|level| bindings.import_view(&mut *graph, "rain rings level", &rings.level_views[level]));

    passes.effects.rain.add_rings(
      &mut *graph,
      view,
      parameters,
      RainRingsDraw {
        levels,
        size: rings.size,
        near,
      },
    );

    Some(bindings.import_view(&mut *graph, "rain rings", &rings.view))
  }

  /// The rain's parameters over `covers`, its uniform placed over an image of `over`'s size where given, and the
  /// streaks, the near ones and the splash's indices to draw; none without rain or a cover.
  fn get_parameters<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    RainCovers {
      cover,
      surface_water,
      surface_lowest,
    }: RainCovers,
    over: Option<(u32, u32)>,
  ) -> Option<(RainCounts, RainParameters<'a>)> {
    let (Some(counts), Some(cover), Some([streak, splash]), Some(model)) =
      (self.draw, cover, &self.views, &self.splash)
    else {
      return None;
    };
    let model: &WeatherModelBuffers = model.get();
    let uniform: RainUniform = over.map_or(self.uniform, |size| self.uniform.over(size));

    Some((
      counts,
      RainParameters {
        rain: runtime.push_uniform(&uniform),
        cover,
        surface_water,
        surface_lowest,
        streak_texture: bindings.import_view(&mut *graph, "rain streak", streak),
        splash_texture: bindings.import_view(&mut *graph, "rain splash", splash),
        rain_sampler: passes.effects.rain.get_sampler(),
        splash_vertices: StorageArray::new(bindings.import_buffer(&mut *graph, "splash vertices", &model.vertices)),
        splash_indices: StorageArray::new(bindings.import_buffer(&mut *graph, "splash indices", &model.indices)),
      },
    ))
  }

  /// What the enhanced rain reads beside it, the image it bends being `refracted`.
  fn get_look<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes, handles, lit, ..
    }: ViewRecord<'a>,
    refracted: GraphTexture,
  ) -> RainLookParameters<'a> {
    RainLookParameters {
      refracted,
      refracted_sampler: passes.effects.rain.get_sampler(),
      drop_map: bindings.import_view(&mut *graph, "rain drop map", &passes.effects.rain.get_maps().drop),
      splash_map: bindings.import_view(&mut *graph, "rain splash map", &passes.effects.rain.get_maps().splash),
      map_sampler: passes.effects.rain.get_map_sampler(),
      depth_target: handles.depth,
      light_target: handles.light,
      lighting: lit.lighting,
      exposure: StorageValue::new(lit.exposure),
    }
  }
}

impl ViewFeature for FallingRainFeature {
  /// The splash's model, kept for the level's weather, and the enhanced drops' rings.
  fn get_memory(&self) -> u64 {
    self.splash.as_ref().map_or(0, |model| model.get().get_memory())
      + self.rings.as_ref().map_or(0, RainRings::get_memory)
  }

  fn release(&mut self) {
    *self = Self::default();
  }

  /// Its motion restarted, its clock, its drops' square and its eased pace and lean start again from nothing, as at a
  /// view's first frame; a cut leaves them, the drops falling on around the camera wherever it stands.
  fn restart(&mut self, restart: ViewRestart) {
    if restart == ViewRestart::Motion {
      self.clock = 0.0;
      self.aim = None;
      self.eased = None;
    }
  }
}
