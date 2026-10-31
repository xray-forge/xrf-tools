use std::ptr;
use std::sync::{Arc, Weak};

use glam::{Vec2, Vec3, Vec4};
use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, StorageArray, StorageValue};

use crate::contract::render_rain_settings::RenderRainSettings;

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_rain::RenderRain;
use crate::lighting::ambient_gust::AmbientGust;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::pass::level_passes::LevelPasses;
use crate::pass::rain_look_parameters::RainLookParameters;
use crate::pass::rain_parameters::RainParameters;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::over_scene::add_over_scene;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::rain_rings::RainRings;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::scene::texture::weather_textures::WeatherTextures;

/// The falling rain, while the weather rains and the view shows it: the splash's model built for the level's weather,
/// the uniform at the rain cover's window, the streak's and the splash's textures, and the drops drawn.
#[derive(Default)]
pub struct FallingRainFeature {
  /// The splash's model, with the level's weather it was built for, held weakly so its address is never another's.
  splash: Option<(Weak<RenderLevelWeather>, WeatherModelBuffers)>,
  uniform: RainUniform,
  /// The streak's and the splash's textures, as the cache holds them when the frame is prepared.
  views: Option<[wgpu::TextureView; 2]>,
  /// The streaks drawn and the near ones among them, which splash and ring, and the splash's indices; none while it
  /// does not rain.
  draw: Option<(u32, u32, u32)>,
  /// The rain's clock at the last frame it fell, which this frame's step is taken from.
  last_time: Option<f32>,
  /// Where the enhanced drops' square stood from the eye last frame, eased towards the camera's way.
  aim: Option<Vec2>,
  /// The rain's clock, run on at its pace each frame.
  clock: f32,
  /// The pace and the lean it fell at last frame, eased towards the weather's.
  eased: Option<(f32, Vec3)>,
  /// The rings the enhanced drops leave where they land, made once they first fall.
  rings: Option<RainRings>,
}

impl FallingRainFeature {
  /// Takes this frame's rain where `rainfall` falls as the rain's `settings` say, the camera looking along `look`: the
  /// splash's model built for the level's weather, the uniform at the rain cover's window and shape (`cover`), and
  /// the textures; answers the uniform, which the wet surfaces follow, none where the weather has no rain to draw.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    ((rainfall, settings), gust, (time, look)): (
      (Option<RenderRainfall>, &RenderRainSettings),
      AmbientGust,
      (f32, Vec3),
    ),
    (cover, (texels, reach)): (Vec4, (f32, f32)),
    weather_textures: WeatherTextures<'_>,
  ) -> Option<RainUniform> {
    self.draw = None;
    self.views = None;

    // Held within a tenth of a second and never back, so a pause or a jump does not swing the drops' square.
    let step: f32 = self
      .last_time
      .replace(time)
      .map_or(0.0, |last| time - last)
      .clamp(0.0, 0.1);
    let aim: Vec2 = RainUniform::ease_aim(self.aim, look, step);

    self.aim = Some(aim);

    let rainfall: RenderRainfall = rainfall?;
    let (weather, rain): (&Arc<RenderLevelWeather>, &RenderRain) =
      weather.and_then(|weather| weather.rain.as_ref().map(|rain: &RenderRain| (weather, rain)))?;

    if self
      .splash
      .as_ref()
      .is_none_or(|(built, _)| !ptr::eq(built.as_ptr(), Arc::as_ptr(weather)))
    {
      self.splash = Some((
        Arc::downgrade(weather),
        WeatherModelBuffers::new(device, rain.drop.as_ref()),
      ));
    }

    let (_, splash) = self.splash.as_ref()?;
    let target: RainUniform = RainUniform::new(
      (&rainfall, settings.drops),
      (lighting.wind, gust.strength),
      (cover, Vec4::new(texels, reach, 0.0, 0.0)),
      aim,
      splash.index_count,
    )
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
    self.clock += step * pace;

    let uniform: RainUniform = target.clocked((self.clock, pace), axis);
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
    self.draw = Some((uniform.count, uniform.near_count, splash.index_count));

    Some(uniform)
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
    self.uniform.is_enhanced != 0 && self.draw.is_some_and(|(_, _, splash_indices)| splash_indices > 0)
  }

  /// Follows the rain cover to where it stands once it has moved, as the drops read it.
  pub fn follow_cover(&mut self, cover: Vec4, (texels, reach): (f32, f32)) {
    self.uniform.window = cover;
    self.uniform.cover_shape = Vec4::new(texels, reach, 0.0, 0.0);
  }

  /// Follows how the frame is shown (`DisplayMode::get_display`), which the streaks and the engine's splashes enter the
  /// scene through.
  pub fn follow_display(&mut self, display: Vec4) {
    self.uniform.display = display;
  }

  /// Follows the level water's and the level surface's maps (`water`, `surface`, both of `shape`), which the enhanced
  /// drops land on where `is_drawn`.
  pub fn follow_surfaces(&mut self, (water, surface, shape): (Vec4, Vec4, Vec4), is_drawn: bool) {
    let is_landing: bool = is_drawn && self.uniform.is_enhanced != 0;

    self.uniform.water = if is_landing { water } else { Vec4::ZERO };
    self.uniform.surface = if is_landing { surface } else { Vec4::ZERO };
    self.uniform.water_shape = if is_landing { shape } else { Vec4::ZERO };
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
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, lit, covers): (
      ViewTargetHandles,
      &LightingHandles<'a>,
      (Option<GraphTexture>, GraphTexture, GraphTexture),
    ),
  ) {
    let Some((counts, parameters)) =
      self.get_parameters(GraphDeclaration::new(graph, bindings, runtime), passes, covers, None)
    else {
      return;
    };
    // The enhanced splashes bend the ground and the streaks undo the water's distortion under them, which the window
    // pass would bend the drops drawn after the resolve by; neither reads a blurred image.
    let look: Option<RainLookParameters<'a>> = (self.uniform.is_enhanced != 0).then(|| {
      let nothing: GraphTexture = bindings.import_view(&mut *graph, "rain nothing", passes.rain.get_nothing());

      self.get_look(
        GraphDeclaration::new(graph, bindings, runtime),
        passes,
        (handles, lit),
        nothing,
      )
    });
    let mut pass = add_over_scene(&mut *graph, handles, "rain").parameters(&parameters);

    if let Some(look) = &look {
      pass = pass
        .parameters(look)
        .color(GraphColorAttachment::new(handles.distortion, wgpu::LoadOp::Load));
    }

    pass.record(move |context| passes.rain.record(context, view, (&parameters, look.as_ref()), counts));
  }

  /// Adds the enhanced streaks over a copy of `shown`, the image the temporal resolve shows, `size` texels across, so
  /// no history holds them; answers the copy, which the window pass shows in its place, none without enhanced rain.
  pub fn add_over_resolved<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, lit, covers): (
      ViewTargetHandles,
      &LightingHandles<'a>,
      (Option<GraphTexture>, GraphTexture, GraphTexture),
    ),
    (shown, size): (GraphTexture, (u32, u32)),
  ) -> Option<GraphTexture> {
    if self.uniform.is_enhanced == 0 {
      return None;
    }

    let ((streaks, _, _), parameters) = self.get_parameters(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      covers,
      Some(size),
    )?;
    let layer: GraphTexture = passes.rain.add_layer(&mut *graph, shown, size);
    // `phase_ssfx_rain`: what the drops bend, the image as shown, blurred small.
    let refracted: GraphTexture = passes.rain.add_blur(&mut *graph, shown, size);
    let look: RainLookParameters<'a> = self.get_look(
      GraphDeclaration::new(graph, bindings, runtime),
      passes,
      (handles, lit),
      refracted,
    );

    graph
      .add_raster_pass("rain")
      .color(GraphColorAttachment::new(layer, wgpu::LoadOp::Load))
      .parameters(&parameters)
      .parameters(&look)
      .record(move |context| passes.rain.record_over(context, view, (&parameters, &look), streaks));

    Some(layer)
  }

  /// Adds the rings the enhanced drops leave where they land on `covers`, while `is_read` says something reads them;
  /// answers them, none without them.
  pub fn add_rings<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    covers: (Option<GraphTexture>, GraphTexture, GraphTexture),
    is_read: bool,
  ) -> Option<GraphTexture> {
    let rings: &RainRings = self
      .rings
      .as_ref()
      .filter(|_| is_read && self.uniform.get_rings() > 0.0)?;
    let ((_, near, _), parameters) =
      self.get_parameters(GraphDeclaration::new(graph, bindings, runtime), passes, covers, None)?;
    let levels: [GraphTexture; RainRings::LEVELS as usize] =
      std::array::from_fn(|level| bindings.import_view(&mut *graph, "rain rings level", &rings.level_views[level]));

    passes
      .rain
      .add_rings(&mut *graph, view, parameters, (levels, rings.size, near));

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
    (cover, surface_water, surface_lowest): (Option<GraphTexture>, GraphTexture, GraphTexture),
    over: Option<(u32, u32)>,
  ) -> Option<((u32, u32, u32), RainParameters<'a>)> {
    let (Some(counts), Some(cover), Some([streak, splash]), Some((_, model))) =
      (self.draw, cover, &self.views, &self.splash)
    else {
      return None;
    };
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
        rain_sampler: passes.rain.get_sampler(),
        splash_vertices: StorageArray::new(bindings.import_buffer(&mut *graph, "splash vertices", &model.vertices)),
        splash_indices: StorageArray::new(bindings.import_buffer(&mut *graph, "splash indices", &model.indices)),
      },
    ))
  }

  /// What the enhanced rain reads beside it, the image it bends being `refracted`.
  fn get_look<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    (handles, lit): (ViewTargetHandles, &LightingHandles<'a>),
    refracted: GraphTexture,
  ) -> RainLookParameters<'a> {
    RainLookParameters {
      refracted,
      refracted_sampler: passes.rain.get_sampler(),
      drop_map: bindings.import_view(&mut *graph, "rain drop map", &passes.rain.get_maps().drop),
      splash_map: bindings.import_view(&mut *graph, "rain splash map", &passes.rain.get_maps().splash),
      map_sampler: passes.rain.get_map_sampler(),
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
    self
      .splash
      .as_ref()
      .map_or(0, |(_, model)| model.vertices.size() + model.indices.size())
      + self.rings.as_ref().map_or(0, RainRings::get_memory)
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}
