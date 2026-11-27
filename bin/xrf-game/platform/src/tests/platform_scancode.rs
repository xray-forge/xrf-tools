use crate::event::platform_scancode::PlatformScancode;

fn code(name: &str) -> Option<u16> {
  PlatformScancode::from_name(name).map(|scancode| scancode.0)
}

#[test]
fn names_the_console_key_grave_in_any_case() {
  assert_eq!(PlatformScancode::from_name("Grave"), Some(PlatformScancode::GRAVE));
  assert_eq!(PlatformScancode::from_name(" grave "), Some(PlatformScancode::GRAVE));
  assert_eq!(PlatformScancode::from_name("`"), Some(PlatformScancode::GRAVE));
}

#[test]
fn reads_sdls_names_of_the_standard_keyboard_as_sdl_numbers_them() {
  assert_eq!(code("A"), Some(4));
  assert_eq!(code("w"), Some(26));
  assert_eq!(code("1"), Some(30));
  assert_eq!(code("0"), Some(39));
  assert_eq!(code("F12"), Some(69));
  assert_eq!(code("Keypad 0"), Some(98));
  assert_eq!(code("Keypad 7"), Some(95));
  assert_eq!(code("Space"), Some(44));
  assert_eq!(code("Left Shift"), Some(225));
  assert_eq!(code("Right GUI"), Some(231));
  assert_eq!(code("F13"), None);
  assert_eq!(code("mouse1"), None);
}
