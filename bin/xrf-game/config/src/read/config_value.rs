use std::time::Duration;

use xrf_ltx::Condlist;
use xrf_math::glam::Vec3;

/// A kind of value a config key holds, read from its text as X-Ray reads it, or why it is not one.
pub trait ConfigValue: Sized {
  fn parse(text: &str) -> Result<Self, String>;
}

impl ConfigValue for f32 {
  fn parse(text: &str) -> Result<Self, String> {
    text
      .trim()
      .parse::<Self>()
      .map_err(|_| format!("'{text}' is no number"))
  }
}

impl ConfigValue for u32 {
  /// A whole number not below zero; a fraction is cut as `atoi` cuts it, a negative is refused.
  fn parse(text: &str) -> Result<Self, String> {
    let number: f32 = f32::parse(text)?;

    if !(0.0..=Self::MAX as f32).contains(&number) {
      return Err(format!("'{text}' is no whole number from 0"));
    }

    Ok(number as Self)
  }
}

impl ConfigValue for bool {
  /// A flag as `CInifile::r_bool` reads one: `on`, `yes`, `true` or `1`, and their opposites, in any case.
  fn parse(text: &str) -> Result<Self, String> {
    match text.trim().to_ascii_lowercase().as_str() {
      "on" | "yes" | "true" | "1" => Ok(true),
      "off" | "no" | "false" | "0" => Ok(false),
      _ => Err(format!("'{text}' is no flag: write true or false")),
    }
  }
}

impl ConfigValue for String {
  /// The text trimmed; empty text is none.
  fn parse(text: &str) -> Result<Self, String> {
    let text: &str = text.trim();

    if text.is_empty() {
      return Err(Self::from("it is empty"));
    }

    Ok(Self::from(text))
  }
}

impl ConfigValue for Vec3 {
  /// Three numbers, comma-separated, as `r_fvector3` reads them.
  fn parse(text: &str) -> Result<Self, String> {
    let numbers: Vec<f32> = text.split(',').map(f32::parse).collect::<Result<_, _>>()?;

    match numbers.as_slice() {
      [x, y, z] => Ok(Self::new(*x, *y, *z)),
      _ => Err(format!("'{text}' is not three numbers")),
    }
  }
}

impl ConfigValue for Duration {
  /// A time of day as `hours:minutes:seconds`, the time since midnight, as `CALifeTimeManager` reads `start_time`
  /// (`%d:%d:%d`): hours below 24, minutes and seconds below 60.
  fn parse(text: &str) -> Result<Self, String> {
    let parts: Vec<u32> = text
      .trim()
      .split(':')
      .map(|part| part.trim().parse::<u32>())
      .collect::<Result<_, _>>()
      .map_err(|_| format!("'{text}' is no time: write hours:minutes:seconds"))?;

    match parts.as_slice() {
      [hours, minutes, seconds] if *hours < 24 && *minutes < 60 && *seconds < 60 => Ok(Self::from_secs(
        u64::from(*hours) * 3_600 + u64::from(*minutes) * 60 + u64::from(*seconds),
      )),
      _ => Err(format!("'{text}' is no time of day: write hours:minutes:seconds")),
    }
  }
}

impl ConfigValue for Condlist {
  fn parse(text: &str) -> Result<Self, String> {
    // The condlist's own parser, which an inherent method's name reaches before this trait's.
    Self::parse(text).map_err(|error| format!("'{text}' is no condlist: {error}"))
  }
}
