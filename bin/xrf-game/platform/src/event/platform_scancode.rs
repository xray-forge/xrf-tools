use sdl3::keyboard::Scancode;

/// A key by its physical place on the keyboard, numbered as SDL and the USB HID usage tables number it, which X-Ray 16
/// reads input through and OpenXRay's key codes are: `4` is the key where a US layout prints A.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlatformScancode(pub u16);

impl PlatformScancode {
  /// The key SDL names `name` (`W`, `Grave`, `Return`, `Left Shift`), ignoring case; none for a name SDL does not know.
  pub fn from_name(name: &str) -> Option<Self> {
    Scancode::from_name(name).and_then(|scancode| u16::try_from(scancode as i32).ok().map(Self))
  }
}
