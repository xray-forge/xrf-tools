use crate::event::platform_scancode::PlatformScancode;

// SDL's own names need its library loaded, which the tests do not do; the word the crate adds is its own.
#[test]
fn names_the_console_key_grave_in_any_case() {
  assert_eq!(PlatformScancode::from_name("Grave"), Some(PlatformScancode::GRAVE));
  assert_eq!(PlatformScancode::from_name(" grave "), Some(PlatformScancode::GRAVE));
}
