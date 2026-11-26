/// A gamepad's axis, numbered as SDL's `SDL_GamepadAxis` numbers it: the left stick's x and y, the right stick's, then
/// the left and right triggers.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlatformGamepadAxis(pub u8);
