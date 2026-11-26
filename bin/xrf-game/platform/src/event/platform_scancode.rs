/// A key by its physical place on the keyboard, numbered as SDL and the USB HID usage tables number it, which X-Ray 16
/// reads input through and OpenXRay's key codes are: `4` is the key where a US layout prints A.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlatformScancode(pub u16);
