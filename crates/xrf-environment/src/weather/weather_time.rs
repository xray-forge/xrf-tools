use serde::Serialize;
use xrf_engine_target::XrayEngine;

/// A time of day as a keyframe's section names it, in whole seconds since midnight: the engine's `exec_time`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct WeatherTime(u32);

impl WeatherTime {
  /// Seconds a day lasts, `DAY_LENGTH`.
  pub const DAY: u32 = 24 * 60 * 60;

  /// Reads a section name as `CEnvDescriptor::load` does, `sscanf(name, "%d:%d:%d")` with every field in range.
  pub fn parse(name: &str, engine: XrayEngine) -> Option<Self> {
    let fields: Vec<i64> = Self::scan_fields(name);

    if engine == XrayEngine::Vanilla && fields.len() != 3 {
      return None;
    }

    let field = |index: usize| fields.get(index).copied().unwrap_or(0);
    let (hours, minutes, seconds) = (field(0), field(1), field(2));

    if !(0..24).contains(&hours) || !(0..60).contains(&minutes) || !(0..60).contains(&seconds) {
      return None;
    }

    u32::try_from(hours * 3600 + minutes * 60 + seconds).ok().map(Self)
  }

  /// A time from seconds since midnight, wrapped into the day.
  pub fn from_seconds(seconds: u32) -> Self {
    Self(seconds % Self::DAY)
  }

  /// Seconds since midnight.
  pub fn get_seconds(self) -> u32 {
    self.0
  }

  /// The time of day seconds fall at, any number of days either way, in seconds since midnight.
  pub fn of_day(seconds: f32) -> f32 {
    seconds.rem_euclid(Self::DAY as f32)
  }

  /// The integers `sscanf` converts from `%d:%d:%d`, stopping where the next does not follow a colon.
  fn scan_fields(name: &str) -> Vec<i64> {
    let mut fields: Vec<i64> = Vec::with_capacity(3);
    let mut rest: &str = name;

    while fields.len() < 3 {
      let text: &str = rest.trim_start();
      let bytes: &[u8] = text.as_bytes();
      let sign: usize = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
      let digits: usize = bytes[sign..].iter().take_while(|it| it.is_ascii_digit()).count();

      let Some(field) = (digits > 0)
        .then(|| text[..sign + digits].parse::<i64>().ok())
        .flatten()
      else {
        break;
      };

      fields.push(field);
      rest = &text[sign + digits..];

      match rest.strip_prefix(':') {
        Some(next) => rest = next,
        None => break,
      }
    }

    fields
  }
}

#[cfg(test)]
mod tests {
  use xrf_engine_target::XrayEngine;

  use super::WeatherTime;

  #[test]
  fn reads_a_keyframe_name_into_seconds() {
    assert_eq!(
      WeatherTime::parse("00:00:00", XrayEngine::Vanilla).map(WeatherTime::get_seconds),
      Some(0)
    );
    assert_eq!(
      WeatherTime::parse("13:30:15", XrayEngine::Vanilla).map(WeatherTime::get_seconds),
      Some(48615)
    );
  }

  #[test]
  fn refuses_what_the_engine_asserts_on() {
    assert_eq!(WeatherTime::parse("24:00:00", XrayEngine::Vanilla), None);
    assert_eq!(WeatherTime::parse("12:60:00", XrayEngine::Extended), None);
    assert_eq!(WeatherTime::parse("noon", XrayEngine::Vanilla), None);
  }

  // Monolith dropped OpenXRay's check that all three fields were read, and starts each at zero.
  #[test]
  fn reads_a_short_name_as_monolith_does() {
    assert_eq!(WeatherTime::parse("12:00", XrayEngine::Vanilla), None);
    assert_eq!(
      WeatherTime::parse("12:00", XrayEngine::Extended).map(WeatherTime::get_seconds),
      Some(43200)
    );
    assert_eq!(
      WeatherTime::parse("noon", XrayEngine::Extended).map(WeatherTime::get_seconds),
      Some(0)
    );
  }
}
