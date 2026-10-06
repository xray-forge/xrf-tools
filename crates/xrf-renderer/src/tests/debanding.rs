//! The sky debanding the present runs.

use glam::Vec4;

use crate::contract::render_debanding_mode::RenderDebandingMode;
use crate::contract::render_debanding_quality::RenderDebandingQuality;
use crate::contract::render_debanding_settings::RenderDebandingSettings;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::present_uniform::PresentUniform;

fn enhanced() -> RenderDebandingSettings {
  RenderDebandingSettings {
    mode: RenderDebandingMode::Enhanced,
    ..RenderDebandingSettings::default()
  }
}

fn options_with(debanding: RenderDebandingSettings) -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.mode.is_lit = true;
  options.show.is_sky_visible = true;
  options.features.debanding = debanding;

  options
}

fn present() -> PresentUniform {
  PresentUniform::new(
    RenderDebugView::Final,
    (false, false, None),
    false,
    0.0,
    RenderRect::default(),
    &RenderImageCorrections::default(),
    (None, false),
  )
}

#[test]
fn ships_the_sky_as_drawn_with_two_passes_out_to_48_pixels() {
  let defaults: RenderDebandingSettings = RenderDebandingSettings::default();

  assert_eq!(defaults.mode, RenderDebandingMode::Engine);
  assert_eq!(defaults.quality, RenderDebandingQuality::Medium);
  assert_eq!(defaults.radius, 48.0);
  assert!(!defaults.is_drawn());
  assert!(enhanced().is_drawn());
  assert!(
    !RenderDebandingSettings {
      radius: 0.0,
      ..enhanced()
    }
    .is_drawn()
  );
  assert_eq!(
    RenderDebandingQuality::ALL.map(RenderDebandingQuality::get_passes),
    [1, 2, 3, 4]
  );
}

// Nothing is debanded while the engine's sky is asked for, no sky is shown, unlit or in wireframe.
#[test]
fn debands_only_a_sky_shown_lit_and_solid() {
  assert_eq!(
    PresentUniform::get_debanding(&options_with(enhanced())),
    Some(RenderDebandingQuality::Medium)
  );
  assert_eq!(
    PresentUniform::get_debanding(&options_with(RenderDebandingSettings::default())),
    None
  );

  for change in [
    |options: &mut RenderViewOptions| options.show.is_sky_visible = false,
    |options: &mut RenderViewOptions| options.mode.is_lit = false,
    |options: &mut RenderViewOptions| options.mode.is_wireframe = true,
  ] {
    let mut options: RenderViewOptions = options_with(enhanced());

    change(&mut options);

    assert_eq!(PresentUniform::get_debanding(&options), None);
  }
}

#[test]
fn hands_the_present_its_passes_radius_and_clock() {
  assert_eq!(present().deband, Vec4::ZERO);
  assert_eq!(
    present().with_debanding(3, 32.0, 1.5).deband,
    Vec4::new(3.0, 32.0, 1.5, 0.0)
  );
}
