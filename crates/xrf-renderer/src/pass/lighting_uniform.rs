use glam::{Mat4, Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_renderer_core::ShaderStruct;

use crate::context::gpu_tracing::GpuTracing;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_fog_settings::RenderFogSettings;
use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_view_options::RenderViewOptions;
use crate::lighting::light_specular::to_light_specular;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::srgb::{anchored_light, authored_light};
use crate::pass::bitmask_search::BitmaskSearch;
use crate::pass::display_mode::DisplayMode;
use crate::pass::light_encoding::LightEncoding;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::picture_shares::PictureShares;
use crate::pass::reflection_trace::ReflectionTrace;

/// The lighting the shaders read as their `Lighting`: one viewport's, since the sun is given in its view space.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Lighting")]
pub struct LightingUniform {
  /// Towards the sun, in view space.
  pub to_sun: Vec4,
  /// The sun's colour, then its specular weight, at the pre-exposure.
  pub sun: Vec4,
  /// The ambient light, at the pre-exposure.
  pub ambient: Vec4,
  /// `env_color` by the lights' gain, then one where the hemisphere is the engine's own square, at the anchored levels.
  pub environment: Vec4,
  /// The irradiance's stand-in, then one once both cubes are up.
  pub sky_irradiance: Vec4,
  /// The sky's irradiance over π as harmonics, which the physical light lights the hemisphere by (`SkyHarmonics`), times
  /// the hemisphere's light scale: the first's `w` one once the sky was projected.
  pub sky_harmonics: [Vec4; 9],
  /// What the sky's light the reflections read is multiplied by, per channel: at the anchored levels the scale its
  /// harmonics take to the anchored hemisphere (`SkyHarmonics::get_level_scale`), so a rough reflection of the sky and
  /// the diffuse light the sky gives agree where both join as linear light; one elsewhere, and while the sky is too
  /// dark to have a shape. Then the share of the fog's colour the sky along the view gives (`RenderFogColour`) at the
  /// anchored levels while the harmonics hold the sky's shape; nought elsewhere, the fog the weather's colour.
  pub sky_reflected: Vec4,
  /// The fog's colour as the weather authors it (a display value: the passes take it along their view through
  /// `common/fog_colour`), then one where it fogs.
  pub fog_color: Vec4,
  /// `fog_params.x` and `.w`, then one where far geometry fades into the sky's haze above the fold, then one where the
  /// sky is drawn.
  pub fog: Vec4,
  /// `sky_color`, then the blend between the keyframes' skies.
  pub sky: Vec4,
  /// The sky's rotation, the clouds', the clouds' clock.
  pub sky_params: Vec4,
  /// The clouds' colour and cover; nothing where they are hidden.
  pub clouds: Vec4,
  /// One for Anomaly's shading, the rain's density, one where the enhanced glass draws glass in place of the
  /// composited surfaces, and one where the light is physical (`RenderLightModel`): the colours here lit as linear light,
  /// the irradiance cubes decoded where they are read.
  pub engine: Vec4,
  /// The scale the frame's light is written at per cd/m² (`PreExposure`), then the last frame's, by which a history is
  /// brought to this one's, then the cd/m² a light of one in the engine's units stands for (`LightUnits`), the weather's
  /// and the local lights'. The sun and the ambient here arrive at the weather's and the pre-exposure; the shaders give
  /// the light they make from the engine's values the same.
  pub pre_exposure: Vec4,
  /// How much light the occlusion's creases bounce back (VBAO's alone), then ones where the scene is
  /// lit, the exposure adapts and the occlusion darkens.
  pub params: Vec4,
  /// `L_ambient`, `L_hemi_color` and `L_sun_color` as a forward pass binds them: the weather's own, neither doubled
  /// nor scaled by the console, nor decoded under the physical light, since the forward passes light as the engine does
  /// and hand the scene a shown colour.
  pub forward_ambient: Vec4,
  pub forward_hemi: Vec4,
  pub forward_sun: Vec4,
  /// The sun's sprite colour times how far it has faded in, then half its side as a share of the distance it stands
  /// at; zero where none is drawn.
  pub sun_sprite: Vec4,
  /// The sun shafts' density, then the steps along a ray `accum_volumetric_sun` takes as the settings ask; nothing
  /// where they are not drawn.
  pub shafts: Vec4,
  /// One while flora is lit as foliage, how much sunlight it passes through from behind, and how much of the sun's
  /// colour that light keeps.
  pub flora: Vec4,
  /// One where the indirect light is gathered and added.
  pub indirect: Vec4,
  /// One where the reflections are traced and blended in, the frame's pixels a traced pixel stands for each way, their
  /// intensity, and how far the sky's highlights are expanded as their light (`sky_as_light`).
  pub reflections: Vec4,
  /// The enhanced fog's height, density and sun colour, then one where it is drawn; nothing where the engine's alone
  /// is.
  pub height_fog: Vec4,
  /// The fog's share past which the enhanced fog fades fogged surfaces into the sky behind them, then one where it
  /// does; nothing where they fade by the fog's square, as the engine fades them.
  pub sky_fade: Vec4,
  /// The roughness from which a traced reflection fades into the prefiltered sky, and past which a surface is not
  /// traced.
  pub reflection_cut: Vec4,
  /// The share of the frame's exposure the game's pictures (the sky, the clouds, the fog's colour) are shown at
  /// (`PictureShares`), then one where it is of the pre-exposure.
  pub pictures: Vec4,
  /// The gain the lights took (`LightUnits::light_gain`) where their diffuse sum is decoded once, as the engine's display
  /// shows it (`authored_diffuse` in `common/hmodel`), at the authored levels, nought elsewhere; then the sun's colour as
  /// the weather authors it, a display value by the look's sun scale, which the sun's shafts add to the frame's display
  /// values (`fs_sun_shafts_over`) at the authored levels and the forward passes take as the engine binds it
  /// (`sun_shown`) at the anchored, where the sun itself is anchored; nought elsewhere.
  pub summed_light: Vec4,
  /// How the present shows the frame (`DisplayMode::get_display`), which the game's pictures enter the scene through
  /// the inverse of (`picture_for_display` in `common/display_transform`).
  pub display: Vec4,
}

impl LightingUniform {
  pub fn new(lighting: &RenderLighting, view: Mat4, options: &RenderViewOptions, frame: &LightingFrame) -> Self {
    // The direction the light travels, turned to face the sun and into view space.
    let to_sun: Vec3 = view
      .transform_vector3(-lighting.get_sun_direction())
      .normalize_or_zero();
    let fog = lighting.fog.filter(|_| options.show.is_fogged);
    let (offset, scale) = fog.map_or((0.0, 0.0), |fog| fog.get_params());
    let flag = |is: bool| is as u32 as f32;
    let occlusion: &RenderAmbientOcclusionSettings = &options.features.ambient_occlusion;
    let height_fog: &RenderFogSettings = &options.features.fog;
    // Searched only where the view is lit and solid.
    let is_occluded: bool = occlusion.is_enabled && options.mode.is_lit && !options.mode.is_wireframe;
    // The weather's colours lit as light: display values, their hue decoded and their strength kept where the light is
    // physical, in its units at the pre-exposure. The sky's, the clouds' and the sprite's colour the pictures they
    // tint, which the shaders decode with them; the forward passes' as the engine binds them.
    let is_physical: bool = options.is_light_physical();
    let light_scale: f32 = frame.light_units.weather * frame.pre_exposure.current;
    let gain: f32 = frame.light_units.light_gain;
    let light = |colour: Vec3| authored_light(colour, is_physical) * gain * light_scale;
    let level: Option<RenderLightLevel> = options.get_physical_light_level();
    let is_anchored: bool = level == Some(RenderLightLevel::Anchored);
    let scales = &options.features.light_scales;
    let ambient: Vec3 = lighting.get_ambient(scales);
    let environment: Vec3 = lighting.get_environment(scales);

    // The sun as the world lights it at the physical levels, a white surface facing it showing its illuminance over π;
    // at the anchored levels anchored over the hemisphere a level surface facing up takes and the ambient, its specular
    // following it; else the weather's, its specular weighed as the engine weighs it, which the authored levels'
    // `hmodel` adds to the diffuse in display values as the engine's does (`hmodel_terms`).
    let sun: Vec4 = match frame.daylight {
      Some(daylight) => {
        let sun: Vec3 = daylight.sun / std::f32::consts::PI;

        (sun * frame.pre_exposure.current).extend(to_light_specular(sun, true) * frame.pre_exposure.current)
      }
      None if is_anchored => {
        // The hemisphere's zenith as the irradiance pass last looked it up, the weather's stand-in cube's before.
        let zenith: Vec3 = frame
          .sky_harmonics
          .get_authored_zenith()
          .unwrap_or_else(|| (environment * lighting.sky_irradiance).powf(2.0));
        let sun: Vec3 = anchored_light(lighting.get_sun_color(scales), zenith + ambient);

        (sun * light_scale).extend(to_light_specular(sun, true) * light_scale)
      }
      None => light(lighting.get_sun_color(scales)).extend(lighting.get_sun_specular(scales) * gain * light_scale),
    };

    Self {
      to_sun: to_sun.extend(0.0),
      sun,
      // The flat ambient the game lifts its nights by, kept but at the physical levels, which have the world's dark.
      ambient: match level {
        Some(RenderLightLevel::Physical) => Vec4::ZERO,
        Some(RenderLightLevel::Anchored) => (anchored_light(ambient, Vec3::ZERO) * light_scale).extend(0.0),
        Some(RenderLightLevel::Authored) | None => light(ambient).extend(0.0),
      },
      // The hemisphere's colour by the weather's lights' gain, which `srgb_light` keeps as it scales; then one where
      // the hemisphere is the engine's own square, which the anchored levels' harmonics anchor.
      environment: (environment * gain).extend(flag(is_anchored)),
      sky_irradiance: lighting.sky_irradiance.extend(flag(frame.is_irradiance_up)),
      sky_harmonics: frame
        .sky_harmonics
        .at_level(options.features.light_level, scales.hemi, ambient),
      sky_reflected: match level {
        Some(RenderLightLevel::Anchored) => {
          match frame
            .sky_harmonics
            .get_level_scale(RenderLightLevel::Anchored, scales.hemi, ambient)
          {
            Some(scale) => scale.extend(options.features.fog.colour.get_sky_share()),
            None => Vec4::new(1.0, 1.0, 1.0, 0.0),
          }
        }
        Some(RenderLightLevel::Authored | RenderLightLevel::Physical) | None => Vec4::new(1.0, 1.0, 1.0, 0.0),
      },
      fog_color: fog.map_or(Vec3::ZERO, |fog| fog.color).extend(flag(fog.is_some())),
      fog: Vec4::new(
        offset,
        scale,
        flag(options.show.is_sky_hazed),
        flag(options.show.is_sky_visible),
      ),
      sky: lighting.sky.color.extend(frame.sky_blend),
      sky_params: Vec4::new(
        lighting.sky.rotation,
        lighting.sky.clouds.rotation,
        frame.clouds_time,
        0.0,
      ),
      clouds: if options.show.is_clouded {
        lighting.sky.clouds.color
      } else {
        Vec4::ZERO
      },
      engine: Vec4::new(
        flag(lighting.engine == XrayEngine::Extended),
        lighting.rain.map_or(0.0, |rain| rain.density),
        flag(options.mode.is_lit && !options.mode.is_wireframe && options.features.glass.is_enhanced()),
        flag(is_physical),
      ),
      pre_exposure: Vec4::new(
        frame.pre_exposure.current,
        frame.pre_exposure.previous,
        frame.light_units.weather,
        frame.light_units.local,
      ),
      params: Vec4::new(
        if is_occluded && !occlusion.is_gtao() {
          occlusion.vbao.bounce.clamp(0.0, 1.0)
        } else {
          0.0
        },
        flag(options.mode.is_lit),
        flag(frame.is_adapting),
        flag(is_occluded),
      ),
      forward_ambient: lighting.ambient_color.extend(0.0),
      forward_hemi: lighting.hemisphere_color.extend(0.0),
      forward_sun: lighting.sun_color.extend(0.0),
      sun_sprite: frame.sun_sprite,
      shafts: if options.show.is_sun_shafted && options.mode.is_lit {
        Vec4::new(
          lighting.get_sun_shafts(&options.features.sun_shafts),
          options.features.sun_shafts.get_steps() as f32,
          0.0,
          0.0,
        )
      } else {
        Vec4::ZERO
      },
      flora: Vec4::new(
        flag(options.features.grass.foliage.is_enhanced()),
        options.features.grass.foliage.sss_intensity,
        options.features.grass.foliage.sss_color,
        0.0,
      ),
      indirect: Vec4::new(
        // Whether the search gathers light does not hang on whether its occlusion is traced.
        flag(BitmaskSearch::new(options, GpuTracing::Unavailable).is_some_and(|search| search.is_lit)),
        0.0,
        0.0,
        0.0,
      ),
      reflections: ReflectionTrace::new(options).map_or(Vec4::ZERO, |trace| {
        Vec4::new(1.0, trace.get_ratio() as f32, trace.intensity, trace.sky_gain)
      }),
      height_fog: if fog.is_some() && options.mode.is_lit && height_fog.is_enhanced() {
        Vec4::new(
          height_fog.height.max(0.01),
          height_fog.density,
          height_fog.sun_color,
          1.0,
        )
      } else {
        Vec4::ZERO
      },
      sky_fade: if fog.is_some() && options.mode.is_lit && height_fog.is_enhanced() && height_fog.sky_fade > 0.0 {
        Vec4::new(height_fog.sky_fade.min(0.99), 0.0, 0.0, 1.0)
      } else {
        Vec4::ZERO
      },
      reflection_cut: {
        let limit: f32 = options.features.reflections.trace_roughness.clamp(0.05, 1.0);

        Vec4::new(limit * (2.0 / 3.0), limit, 0.0, 0.0)
      },
      pictures: PictureShares::of(options, lighting, frame).to_vec4(),
      summed_light: {
        let sun: Vec3 = lighting.get_sun_color(scales);

        match level {
          Some(RenderLightLevel::Authored) => Vec4::new(gain, sun.x, sun.y, sun.z),
          Some(RenderLightLevel::Anchored) => Vec4::new(0.0, sun.x, sun.y, sun.z),
          Some(RenderLightLevel::Physical) | None => Vec4::ZERO,
        }
      },
      display: DisplayMode::of(options).get_display(&options.features.grading, &frame.metering),
    }
  }

  /// How the light is written, as `common/lighting` reads it: a light in the engine's units scaled to its luminance at
  /// the pre-exposure (`light_scale`), and whether it is the authored levels' summed light (`is_light_summed`).
  pub fn get_light_encoding(&self) -> LightEncoding {
    LightEncoding {
      scale: self.pre_exposure.z * self.pre_exposure.x,
      is_summed: self.summed_light.x > 0.0,
    }
  }

  /// Whether the light stands at the anchored levels, as `is_light_anchored` in `common/lighting` reads it: the sun and
  /// the lamps shaded by the one BRDF, their specular coloured in a target of its own.
  pub fn is_light_anchored(&self) -> bool {
    self.environment.w > 0.5
  }
}
