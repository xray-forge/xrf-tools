use glam::Vec4;

use crate::contract::render_contact_shadow_mode::RenderContactShadowMode;
use crate::contract::render_contact_shadow_settings::RenderContactShadowSettings;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;

#[test]
fn draws_only_enhanced_with_every_strength_above_none() {
  let enhanced: RenderContactShadowSettings = RenderContactShadowSettings {
    mode: RenderContactShadowMode::Enhanced,
    ..RenderContactShadowSettings::default()
  };

  assert!(!RenderContactShadowSettings::default().is_drawn());
  assert!(enhanced.is_drawn());
  assert!(
    !RenderContactShadowSettings {
      length: 0.0,
      ..enhanced
    }
    .is_drawn()
  );
  assert!(
    !RenderContactShadowSettings {
      intensity: 0.0,
      ..enhanced
    }
    .is_drawn()
  );
  assert!(
    !RenderContactShadowSettings {
      thickness: 0.0,
      ..enhanced
    }
    .is_drawn()
  );
}

// The noise turns with the frame and comes back to its start; a ray spans an eighth of the drawn height at most.
#[test]
fn marches_by_the_settings_over_the_drawn_height() {
  let settings: RenderContactShadowSettings = RenderContactShadowSettings {
    mode: RenderContactShadowMode::Enhanced,
    intensity: 2.0,
    ..RenderContactShadowSettings::default()
  };
  let uniform = |frame: u32| ContactShadowUniform::new(&settings, Vec4::Z, 1600, frame);

  assert_eq!(uniform(0).reach, 200.0);
  assert_eq!(uniform(0).intensity, 1.0);
  assert_eq!(uniform(0).steps, settings.steps);
  assert_eq!(uniform(0).noise, uniform(64).noise);
  assert_ne!(uniform(0).noise, uniform(1).noise);
}
