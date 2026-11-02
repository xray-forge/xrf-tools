#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WeaponSoundValue<'a> {
  name: &'a str,
  has_parameters: bool,
}

impl<'a> WeaponSoundValue<'a> {
  /// Engine sentinel that disables a weapon sound field.
  pub(crate) const NO_SOUND: &'static str = "$no_sound";

  pub(crate) fn parse(value: &'a str) -> Self {
    match value.split_once(',') {
      Some((name, _)) => Self {
        name: name.trim(),
        has_parameters: true,
      },
      None => Self {
        name: value.trim(),
        has_parameters: false,
      },
    }
  }

  pub(crate) fn name(self) -> &'a str {
    self.name
  }

  pub(crate) fn has_parameters(self) -> bool {
    self.has_parameters
  }

  /// Whether the field plays nothing, which the engine reads `$no_sound` as.
  pub(crate) fn is_disabled(self) -> bool {
    self.name == Self::NO_SOUND
  }
}

#[cfg(test)]
mod tests {
  use super::WeaponSoundValue;

  #[test]
  fn parses_the_first_comma_separated_token() {
    let value: WeaponSoundValue<'_> = WeaponSoundValue::parse(" weapons\\ak74\\shot, 1.0, 0.1 ");

    assert_eq!(value.name(), "weapons\\ak74\\shot");
    assert!(value.has_parameters());
  }
}
