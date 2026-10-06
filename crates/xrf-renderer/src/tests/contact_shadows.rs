use glam::Vec4;

use crate::contract::render_contact_shadow_mode::RenderContactShadowMode;
use crate::contract::render_contact_shadow_settings::{RENDER_MAX_CONTACT_SHADOW_LIGHTS, RenderContactShadowSettings};
use crate::contract::render_shadow_settings::RenderShadowSettings;
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

// The lights' half asks only the contact shadows' own settings; the sun's asks its cascades too.
#[test]
fn marches_towards_the_lights_with_or_without_the_sun_shadows() {
  let enhanced: RenderContactShadowSettings = RenderContactShadowSettings {
    mode: RenderContactShadowMode::Enhanced,
    ..RenderContactShadowSettings::default()
  };
  let unshadowed: RenderShadowSettings = RenderShadowSettings {
    is_enabled: false,
    contact: enhanced,
    ..RenderShadowSettings::default()
  };

  assert_eq!(RenderContactShadowSettings::default().get_light_count(), 0);
  assert_eq!(enhanced.get_light_count(), 4);
  assert_eq!(
    RenderContactShadowSettings { lights: 99, ..enhanced }.get_light_count(),
    RENDER_MAX_CONTACT_SHADOW_LIGHTS
  );
  assert_eq!(
    RenderContactShadowSettings {
      intensity: 0.0,
      ..enhanced
    }
    .get_light_count(),
    0
  );
  assert!(!unshadowed.is_sun_contact_drawn());
  assert_eq!(unshadowed.contact.get_light_count(), 4);
  assert!(
    RenderShadowSettings {
      contact: RenderContactShadowSettings { lights: 0, ..enhanced },
      ..RenderShadowSettings::default()
    }
    .is_sun_contact_drawn()
  );
  assert_eq!(ContactShadowUniform::new(&enhanced, Vec4::Z, 1600, 0).lights, 4);
  assert_eq!(ContactShadowUniform::default().lights, 0);
}
