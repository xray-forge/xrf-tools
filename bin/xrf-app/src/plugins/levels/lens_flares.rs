use xrf_engine_target::XrayEngine;
use xrf_environment::{LensFlare, LensFlareKey};
use xrf_renderer::{RenderFlare, RenderLensFlare, RenderSunSprite};

/// One lens flare as `CLensFlareDescriptor` reads it: OpenXRay takes the sprite from `source` where it is written and
/// from `sun` over it where that is written too; a flare short of a radius, opacity or position takes zero.
pub fn to_render_lens_flare(sun: &LensFlare, engine: XrayEngine) -> RenderLensFlare {
  let sprite_keys: [[LensFlareKey; 4]; 2] = [
    [
      LensFlareKey::Source,
      LensFlareKey::SourceTexture,
      LensFlareKey::SourceRadius,
      LensFlareKey::SourceIgnoreColor,
    ],
    [
      LensFlareKey::Sun,
      LensFlareKey::SunTexture,
      LensFlareKey::SunRadius,
      LensFlareKey::SunIgnoreColor,
    ],
  ];
  let is_sourced: bool = engine == XrayEngine::Vanilla && sun.has(LensFlareKey::Source);
  let read = |[switch, texture, radius, colorless]: [LensFlareKey; 4]| {
    sun.get_flag(switch, engine).then(|| RenderSunSprite {
      texture: sun.get_text(texture, engine).to_owned(),
      radius: sun.get_number(radius, engine),
      is_colorless: sun.get_flag(colorless, engine),
    })
  };
  // `sun` written after `source` replaces it whole, its switch too.
  let sprite: Option<RenderSunSprite> = if is_sourced && !sun.has(LensFlareKey::Sun) {
    read(sprite_keys[0])
  } else {
    read(sprite_keys[1])
  };
  let item = |key: LensFlareKey, index: usize| -> f32 {
    sun
      .get_list(key)
      .get(index)
      .and_then(|item| item.trim().parse::<f32>().ok())
      .unwrap_or(0.0)
  };
  let flares: Vec<RenderFlare> = if sun.get_flag(LensFlareKey::Flares, engine) {
    sun
      .get_list(LensFlareKey::FlareTextures)
      .iter()
      .enumerate()
      .map(|(index, texture)| RenderFlare {
        texture: texture.trim().to_owned(),
        radius: item(LensFlareKey::FlareRadius, index),
        opacity: item(LensFlareKey::FlareOpacity, index),
        position: item(LensFlareKey::FlarePosition, index),
      })
      .collect()
  } else {
    Vec::new()
  };

  RenderLensFlare {
    sprite: sprite.filter(|it| !it.texture.is_empty()),
    flares,
    gradient: sun
      .get_flag(LensFlareKey::Gradient, engine)
      .then(|| RenderFlare {
        texture: sun.get_text(LensFlareKey::GradientTexture, engine).to_owned(),
        radius: sun.get_number(LensFlareKey::GradientRadius, engine),
        opacity: sun.get_number(LensFlareKey::GradientOpacity, engine),
        position: 1.0,
      })
      .filter(|it| !it.texture.is_empty()),
    rise_time: sun.get_number(LensFlareKey::BlendRiseTime, engine),
    down_time: sun.get_number(LensFlareKey::BlendDownTime, engine),
  }
}
