use xrf_math::EPS_L;

use crate::lighting::render_clouds::RenderClouds;
use crate::lighting::render_fog::RenderFog;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::lighting::render_sky::RenderSky;
use crate::lighting::render_tree_wind::RenderTreeWind;
use crate::lighting::render_wind::RenderWind;

/// What a fade shows at one step: every value blended from what was shown to what the weather shows now, and the skies
/// walked from one pair to the other in thirds, so the sky's two slots never show a cube they did not show a moment
/// before.
pub fn to_faded_lighting(from: &RenderLighting, to: &RenderLighting, progress: f32) -> RenderLighting {
  let t: f32 = progress.clamp(0.0, 1.0);

  if t >= 1.0 {
    return to.clone();
  }

  let lerp = |a: f32, b: f32| a + (b - a) * t;

  RenderLighting {
    sun_direction: from.sun_direction.lerp(to.sun_direction, t).normalize_or_zero(),
    sun_color: from.sun_color.lerp(to.sun_color, t),
    hemisphere_color: from.hemisphere_color.lerp(to.hemisphere_color, t),
    ambient_color: from.ambient_color.lerp(to.ambient_color, t),
    sky_irradiance: from.sky_irradiance.lerp(to.sky_irradiance, t),
    fog: match (from.fog, to.fog) {
      (Some(a), Some(b)) => Some(RenderFog {
        color: a.color.lerp(b.color, t),
        density: lerp(a.density, b.density),
        distance: lerp(a.distance, b.distance),
        far_plane: lerp(a.far_plane, b.far_plane),
      }),
      (_, fog) => fog,
    },
    sky: fade_sky(&from.sky, &to.sky, t),
    trees: match (from.trees, to.trees) {
      (Some(a), Some(b)) => Some(RenderTreeWind {
        amplitude: lerp(a.amplitude, b.amplitude),
        speed: lerp(a.speed, b.speed),
        rotation: lerp(a.rotation, b.rotation),
        wave: a.wave.lerp(b.wave, t),
      }),
      (_, trees) => trees,
    },
    water_intensity: lerp(from.water_intensity, to.water_intensity),
    sun_shafts: lerp(from.sun_shafts, to.sun_shafts),
    rain: fade_rain(from.rain, to.rain, t),
    wind: RenderWind {
      direction: lerp(from.wind.direction, to.wind.direction),
      velocity: lerp(from.wind.velocity, to.wind.velocity),
    },
    engine: to.engine,
    thunderbolt: to.thunderbolt.clone(),
    ambients: to.ambients.clone(),
  }
}

/// Rain that starts or stops fades from or to nothing, as though it had been falling at no density; under `EPS_L` it
/// does not rain at all.
fn fade_rain(from: Option<RenderRainfall>, to: Option<RenderRainfall>, t: f32) -> Option<RenderRainfall> {
  let either: RenderRainfall = to.or(from)?;
  let lerp = |a: f32, b: f32| a + (b - a) * t;
  let density: f32 = lerp(from.map_or(0.0, |it| it.density), to.map_or(0.0, |it| it.density));
  let (from, to) = (from.unwrap_or(either), to.unwrap_or(either));

  (density >= EPS_L).then(|| RenderRainfall {
    color: from.color.lerp(to.color, t),
    density,
  })
}

/// Two of what a sky slot holds, and how far from the first to the second the shader blends.
struct Slots<'a> {
  pair: &'a [Option<String>; 2],
  blend: f32,
}

fn fade_sky(from: &RenderSky, to: &RenderSky, t: f32) -> RenderSky {
  // One blend drives every slot in the shader, so every slot walks, or none does.
  let is_walked: bool =
    from.textures != to.textures || from.environments != to.environments || from.clouds.textures != to.clouds.textures;
  let walked = |a: &[Option<String>; 2], b: &[Option<String>; 2]| {
    walk(
      Slots {
        pair: a,
        blend: from.blend,
      },
      Slots {
        pair: b,
        blend: to.blend,
      },
      t,
      is_walked,
    )
  };
  let (textures, blend) = walked(&from.textures, &to.textures);
  let (environments, _) = walked(&from.environments, &to.environments);
  // The clouds walk with the skies, whose blend they share in the shader.
  let (clouds, _) = walked(&from.clouds.textures, &to.clouds.textures);

  RenderSky {
    textures,
    environments,
    blend,
    color: from.color.lerp(to.color, t),
    rotation: from.rotation + (to.rotation - from.rotation) * t,
    clouds: RenderClouds {
      textures: clouds,
      color: from.clouds.color.lerp(to.clouds.color, t),
      rotation: from.clouds.rotation + (to.clouds.rotation - from.clouds.rotation) * t,
    },
    sun: to.sun.clone(),
  }
}

/// Two slots walked from one pair to another: the first third leaves the old pair on its heavier half, the second
/// blends that half to the new pair's heavier one, the last brings the new pair to its own blend. Pairs that stay the
/// same only have their blend moved.
fn walk(from: Slots<'_>, to: Slots<'_>, t: f32, is_walked: bool) -> ([Option<String>; 2], f32) {
  let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;

  if !is_walked {
    return (to.pair.clone(), mix(from.blend, to.blend, t));
  }

  let phase: f32 = t * 3.0;
  let from_heavier: usize = usize::from(from.blend >= 0.5);
  let to_heavier: usize = usize::from(to.blend >= 0.5);

  if phase < 1.0 {
    (from.pair.clone(), mix(from.blend, from_heavier as f32, phase))
  } else if phase < 2.0 {
    (
      [from.pair[from_heavier].clone(), to.pair[to_heavier].clone()],
      phase - 1.0,
    )
  } else {
    (to.pair.clone(), mix(to_heavier as f32, to.blend, phase - 2.0))
  }
}
