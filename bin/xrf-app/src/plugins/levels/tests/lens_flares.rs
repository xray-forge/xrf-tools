//! A `suns.ltx` section as the renderer draws it, as `CLensFlareDescriptor` reads it.

use xrf_engine_target::XrayEngine;
use xrf_environment::{EnvironmentValue, LensFlare, LensFlareKey};

use crate::plugins::levels::lens_flares::to_render_lens_flare;

fn new_flare(values: Vec<(LensFlareKey, EnvironmentValue)>) -> LensFlare {
  let mut flare: LensFlare = LensFlare::new("partly_halo", "environment\\suns.ltx");

  flare.values.extend(values);

  flare
}

fn text(value: &str) -> EnvironmentValue {
  EnvironmentValue::Text(value.to_owned())
}

fn list(items: &[&str]) -> EnvironmentValue {
  EnvironmentValue::List(items.iter().map(|it| (*it).to_owned()).collect())
}

fn sun_sprite(switch: bool, texture: &str) -> Vec<(LensFlareKey, EnvironmentValue)> {
  vec![
    (LensFlareKey::Sun, EnvironmentValue::Flag(switch)),
    (LensFlareKey::SunTexture, text(texture)),
    (LensFlareKey::SunRadius, EnvironmentValue::Number(0.3)),
    (LensFlareKey::SunIgnoreColor, EnvironmentValue::Flag(false)),
  ]
}

#[test]
fn reads_the_sprite_flares_and_gradient_a_section_draws() {
  let mut values = sun_sprite(true, "fx\\fx_sun_halo.tga");

  values.extend([
    (LensFlareKey::Flares, EnvironmentValue::Flag(true)),
    (LensFlareKey::FlareTextures, list(&["fx\\fx_flare1", "fx\\fx_flare2"])),
    (LensFlareKey::FlareRadius, list(&["0.08", "0.12"])),
    (LensFlareKey::FlareOpacity, list(&["0.34", "0.26"])),
    (LensFlareKey::FlarePosition, list(&["1.3", "-0.6"])),
    (LensFlareKey::Gradient, EnvironmentValue::Flag(true)),
    (LensFlareKey::GradientTexture, text("fx\\fx_gradient")),
    (LensFlareKey::GradientRadius, EnvironmentValue::Number(1.2)),
    (LensFlareKey::GradientOpacity, EnvironmentValue::Number(0.4)),
    (LensFlareKey::BlendRiseTime, EnvironmentValue::Number(30.0)),
    (LensFlareKey::BlendDownTime, EnvironmentValue::Number(20.0)),
  ]);

  let flare = to_render_lens_flare(&new_flare(values), XrayEngine::Extended);
  let sprite = flare.sprite.expect("the sprite is drawn");

  assert_eq!(sprite.texture, "fx\\fx_sun_halo.tga");
  assert_eq!(sprite.radius, 0.3);
  assert_eq!(flare.flares.len(), 2);
  assert_eq!(flare.flares[1].texture, "fx\\fx_flare2");
  assert_eq!(
    (
      flare.flares[1].radius,
      flare.flares[1].opacity,
      flare.flares[1].position
    ),
    (0.12, 0.26, -0.6)
  );
  assert_eq!(flare.gradient.map(|it| (it.radius, it.opacity)), Some((1.2, 0.4)));
  assert_eq!((flare.rise_time, flare.down_time), (30.0, 20.0));
}

// The engine reads `atof` of an item it does not find, a zero.
#[test]
fn reads_a_flare_short_of_its_lists_as_zero() {
  let flare = to_render_lens_flare(
    &new_flare(vec![
      (LensFlareKey::Flares, EnvironmentValue::Flag(true)),
      (LensFlareKey::FlareTextures, list(&["fx\\a", "fx\\b"])),
      (LensFlareKey::FlareRadius, list(&["0.1"])),
      (LensFlareKey::FlareOpacity, list(&["0.5"])),
      (LensFlareKey::FlarePosition, list(&["1"])),
    ]),
    XrayEngine::Extended,
  );

  assert_eq!(flare.flares[1].radius, 0.0);
  assert_eq!(flare.flares[1].opacity, 0.0);
  assert!(flare.sprite.is_none());
}

// OpenXRay reads Shadow of Chernobyl's `source` first; a `sun` written after it replaces it whole, its switch too.
#[test]
fn takes_the_sprite_from_source_unless_sun_is_written_over_it() {
  let mut sourced = vec![
    (LensFlareKey::Source, EnvironmentValue::Flag(true)),
    (LensFlareKey::SourceTexture, text("fx\\fx_sun")),
    (LensFlareKey::SourceRadius, EnvironmentValue::Number(0.15)),
    (LensFlareKey::SourceIgnoreColor, EnvironmentValue::Flag(true)),
  ];

  let only_source = to_render_lens_flare(&new_flare(sourced.clone()), XrayEngine::Vanilla);

  assert_eq!(
    only_source.sprite.map(|it| (it.texture, it.is_colorless)),
    Some(("fx\\fx_sun".to_owned(), true))
  );

  sourced.extend(sun_sprite(false, "fx\\fx_sun_halo"));

  assert!(
    to_render_lens_flare(&new_flare(sourced), XrayEngine::Vanilla)
      .sprite
      .is_none()
  );
}
