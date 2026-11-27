/// X-Ray's key names (`kW`, `kLSHIFT`, `kNUMPAD7`; `xrGame/xr_level_controller.cpp`'s `keyboards`, named for
/// DirectInput's `DIK_` codes) read as SDL's (`W`, `Left Shift`, `Keypad 7`), which the platform binds keys by.
pub struct GameKeyNames;

impl GameKeyNames {
  /// The SDL name of the key X-Ray names `name`, none for a name that is no key of X-Ray's keyboard (a mouse button,
  /// a pad's).
  pub fn to_sdl(name: &str) -> Option<String> {
    let key: &str = name.trim().strip_prefix('k')?;

    if let Some(renamed) = Self::rename(key) {
      return Some(String::from(renamed));
    }

    if let Some(digit) = key.strip_prefix("NUMPAD")
      && digit.len() == 1
      && digit.chars().all(|character| character.is_ascii_digit())
    {
      return Some(format!("Keypad {digit}"));
    }

    let is_letter_or_digit: bool = key.len() == 1 && key.chars().all(|character| character.is_ascii_alphanumeric());
    let is_function: bool = key
      .strip_prefix('F')
      .and_then(|number| number.parse::<u8>().ok())
      .is_some_and(|number| (1..=12).contains(&number));

    (is_letter_or_digit || is_function).then(|| String::from(key))
  }

  /// SDL's name of a key X-Ray names other than SDL with its `k` dropped.
  fn rename(key: &str) -> Option<&'static str> {
    let sdl: &str = match key {
      "ESCAPE" => "Escape",
      "MINUS" => "-",
      "EQUALS" => "=",
      "BACK" => "Backspace",
      "TAB" => "Tab",
      "LBRACKET" => "[",
      "RBRACKET" => "]",
      "RETURN" => "Return",
      "LCONTROL" => "Left Ctrl",
      "SEMICOLON" => ";",
      "APOSTROPHE" => "'",
      "GRAVE" => "Grave",
      "LSHIFT" => "Left Shift",
      "BACKSLASH" => "\\",
      "COMMA" => ",",
      "PERIOD" => ".",
      "SLASH" => "/",
      "RSHIFT" => "Right Shift",
      "NUMPADSTAR" => "Keypad *",
      "LMENU" => "Left Alt",
      "SPACE" => "Space",
      "CAPITAL" => "CapsLock",
      "NUMLOCK" => "Numlock",
      "SCROLL" => "ScrollLock",
      "NUMPADMINUS" => "Keypad -",
      "NUMPADPLUS" => "Keypad +",
      "NUMPADPERIOD" => "Keypad .",
      "NUMPADENTER" => "Keypad Enter",
      "RCONTROL" => "Right Ctrl",
      "NUMPADSLASH" => "Keypad /",
      "RMENU" => "Right Alt",
      "PAUSE" => "Pause",
      "HOME" => "Home",
      "UP" => "Up",
      "PGUP" => "PageUp",
      "LEFT" => "Left",
      "RIGHT" => "Right",
      "END" => "End",
      "DOWN" => "Down",
      "PGDN" => "PageDown",
      "INSERT" => "Insert",
      "DELETE" => "Delete",
      "APPS" => "Application",
      _ => return None,
    };

    Some(sdl)
  }
}
