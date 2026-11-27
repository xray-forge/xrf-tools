/// A key by its physical place on the keyboard, numbered as SDL and the USB HID usage tables number it, which X-Ray 16
/// reads input through and OpenXRay's key codes are: `4` is the key where a US layout prints A.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlatformScancode(pub u16);

impl PlatformScancode {
  /// The key left of 1, which opens the console.
  pub const GRAVE: Self = Self(53);

  /// The first letter's and the first digit's numbers, A and 1.
  const LETTER_A: u16 = 4;
  const DIGIT_1: u16 = 30;
  const F1: u16 = 58;
  const KEYPAD_1: u16 = 89;

  /// The key SDL names `name` (`W`, `Return`, `Left Shift`, `` ` ``, as `SDL_GetScancodeName` names the standard
  /// keyboard's), ignoring case, or `Grave` for the key SDL names only by its character; none for a name it does not
  /// know. Read from a table of its own, as SDL numbers them, so it needs no SDL loaded: a load thread reads the tree's
  /// controls while the main thread still starts SDL.
  pub fn from_name(name: &str) -> Option<Self> {
    let name: String = name.trim().to_ascii_lowercase();

    Self::of_character(&name)
      .or_else(|| Self::of_numbered(&name))
      .or_else(|| Self::of_word(&name))
      .map(Self)
  }

  /// A letter or a digit of the main block.
  fn of_character(name: &str) -> Option<u16> {
    let [character] = name.as_bytes() else {
      return None;
    };

    match character {
      b'a'..=b'z' => Some(Self::LETTER_A + u16::from(character - b'a')),
      b'1'..=b'9' => Some(Self::DIGIT_1 + u16::from(character - b'1')),
      b'0' => Some(Self::DIGIT_1 + 9),
      _ => None,
    }
  }

  /// A function key, `F1` to `F12`, or a keypad digit, `Keypad 0` to `Keypad 9`.
  fn of_numbered(name: &str) -> Option<u16> {
    if let Some(number) = name.strip_prefix('f').and_then(|number| number.parse::<u16>().ok()) {
      return (1..=12).contains(&number).then(|| Self::F1 + number - 1);
    }

    match name.strip_prefix("keypad ")?.parse::<u16>().ok()? {
      0 => Some(Self::KEYPAD_1 + 9),
      digit @ 1..=9 => Some(Self::KEYPAD_1 + digit - 1),
      _ => None,
    }
  }

  /// A key SDL names by a word or a sign.
  fn of_word(name: &str) -> Option<u16> {
    let code: u16 = match name {
      "return" => 40,
      "escape" => 41,
      "backspace" => 42,
      "tab" => 43,
      "space" => 44,
      "-" => 45,
      "=" => 46,
      "[" => 47,
      "]" => 48,
      "\\" => 49,
      ";" => 51,
      "'" => 52,
      "`" | "grave" => 53,
      "," => 54,
      "." => 55,
      "/" => 56,
      "capslock" => 57,
      "printscreen" => 70,
      "scrolllock" => 71,
      "pause" => 72,
      "insert" => 73,
      "home" => 74,
      "pageup" => 75,
      "delete" => 76,
      "end" => 77,
      "pagedown" => 78,
      "right" => 79,
      "left" => 80,
      "down" => 81,
      "up" => 82,
      "numlock" => 83,
      "keypad /" => 84,
      "keypad *" => 85,
      "keypad -" => 86,
      "keypad +" => 87,
      "keypad enter" => 88,
      "keypad ." => 99,
      "application" => 101,
      "left ctrl" => 224,
      "left shift" => 225,
      "left alt" => 226,
      "left gui" => 227,
      "right ctrl" => 228,
      "right shift" => 229,
      "right alt" => 230,
      "right gui" => 231,
      _ => return None,
    };

    Some(code)
  }
}
