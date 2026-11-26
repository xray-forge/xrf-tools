/// A gamepad's button, numbered as SDL's `SDL_GamepadButton` numbers it by place: `0` is the bottom face button,
/// whatever the pad prints on it.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlatformGamepadButton(pub u8);
