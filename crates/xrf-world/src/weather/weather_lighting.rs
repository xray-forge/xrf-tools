use glam::{Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_environment::{WeatherDescriptor, WeatherMix, WeatherPlayedKeyframe};
use xrf_math::EPS_L;
use xrf_renderer::{
  RenderAmbients, RenderClouds, RenderFog, RenderLighting, RenderRainfall, RenderSky, RenderTreeWind, RenderWind,
};

/// A weather's mix as the scene is lit by it, in renderer space: the skies and clouds of both keyframes by their
/// references, the sky's irradiance stood in for until its cubes are up.
pub fn to_weather_lighting(
  mix: &WeatherMix,
  [a, b]: &[WeatherPlayedKeyframe; 2],
  engine: XrayEngine,
) -> RenderLighting {
  let [a, b]: [&WeatherDescriptor; 2] = [&a.descriptor, &b.descriptor];

  RenderLighting {
    sun_direction: to_renderer(mix.sun_direction),
    sun_color: Vec3::from(mix.sun_color),
    hemisphere_color: Vec3::new(mix.hemi_color[0], mix.hemi_color[1], mix.hemi_color[2]),
    ambient_color: Vec3::from(mix.ambient_color),
    sky_irradiance: RenderLighting::default().sky_irradiance,
    fog: Some(RenderFog {
      color: Vec3::from(mix.fog_color),
      density: mix.fog_density,
      distance: mix.fog_distance,
      far_plane: mix.far_plane,
    }),
    sky: RenderSky {
      textures: [to_named(&a.sky_texture), to_named(&b.sky_texture)],
      environments: [to_environment(a), to_environment(b)],
      blend: mix.weight,
      color: Vec3::from(mix.sky_color),
      rotation: mix.sky_rotation,
      clouds: RenderClouds {
        textures: [to_named(&a.clouds_texture), to_named(&b.clouds_texture)],
        color: Vec4::from(mix.clouds_color),
        rotation: mix.clouds_rotation,
      },
      sun: if mix.weight < 0.5 { a.sun.clone() } else { b.sun.clone() },
    },
    trees: Some(RenderTreeWind {
      amplitude: mix.tree_amplitude,
      speed: mix.tree_speed,
      rotation: mix.tree_rotation,
      wave: Vec3::from(mix.tree_wave),
    }),
    water_intensity: mix.water_intensity,
    sun_shafts: mix.sun_shafts_intensity,
    // Under `EPS_L` it does not rain at all.
    rain: (mix.rain_density >= EPS_L).then(|| RenderRainfall {
      color: Vec3::from(mix.rain_color),
      density: mix.rain_density,
    }),
    wind: RenderWind {
      direction: mix.wind_direction,
      velocity: mix.wind_velocity,
    },
    engine,
    thunderbolt: None,
    ambients: RenderAmbients {
      names: [a.ambient.clone(), b.ambient.clone()],
      weight: mix.weight,
    },
  }
}

fn to_named(reference: &str) -> Option<String> {
  Some(reference.to_owned()).filter(|it| !it.is_empty())
}

/// A keyframe's irradiance cube; one without a sky names only its suffix, which nothing answers to.
fn to_environment(keyframe: &WeatherDescriptor) -> Option<String> {
  to_named(&keyframe.sky_texture).and_then(|_| to_named(&keyframe.sky_texture_env))
}

/// An engine-space direction in renderer space, its `z` negated.
fn to_renderer([x, y, z]: [f32; 3]) -> Vec3 {
  Vec3::new(x, y, -z)
}
