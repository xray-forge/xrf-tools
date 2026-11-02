//! The sky debanding: the sky's textures smoothed at their source, which the combine draws the sky with.

use crate::contract::feature::image::display::render_debanding_mode::RenderDebandingMode;
use crate::contract::feature::image::display::render_debanding_settings::RenderDebandingSettings;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::weather::sky::deband::sky_deband::SkyDeband;

fn enhanced() -> RenderDebandingSettings {
  RenderDebandingSettings {
    mode: RenderDebandingMode::Enhanced,
  }
}

fn options_with(debanding: RenderDebandingSettings) -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.mode.is_lit = true;
  options.show.is_sky_visible = true;
  options.features.debanding = debanding;

  options
}

#[test]
fn ships_the_sky_as_drawn() {
  let defaults: RenderDebandingSettings = RenderDebandingSettings::default();

  assert_eq!(defaults.mode, RenderDebandingMode::Engine);
  assert!(!defaults.is_drawn());
  assert!(enhanced().is_drawn());
}

// Nothing is debanded while the engine's sky is asked for, no sky is shown, unlit or in wireframe.
#[test]
fn debands_only_a_sky_shown_lit_and_solid() {
  assert!(SkyDeband::is_asked(&options_with(enhanced())));
  assert!(!SkyDeband::is_asked(&options_with(RenderDebandingSettings::default())));

  for change in [
    |options: &mut RenderViewOptions| options.show.is_sky_visible = false,
    |options: &mut RenderViewOptions| options.mode.is_lit = false,
    |options: &mut RenderViewOptions| options.mode.is_wireframe = true,
  ] {
    let mut options: RenderViewOptions = options_with(enhanced());

    change(&mut options);

    assert!(!SkyDeband::is_asked(&options));
  }
}
