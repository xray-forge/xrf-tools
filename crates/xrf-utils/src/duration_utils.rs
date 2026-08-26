use std::time::Duration;

/// Convert a duration to whole milliseconds, saturating at `u64::MAX`.
///
/// The workspace wire format for a duration is a `u64` millisecond count, so this is the one place the
/// narrowing from `Duration` is defined.
#[inline]
pub fn duration_to_millis(duration: Duration) -> u64 {
  u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// Format a duration the way every surface reports elapsed time: `742 ms`, `3.2 s`, `1 m 12 s`.
///
/// Mirrors the UI's `formatDuration` in `lib/format/duration.ts` threshold for threshold, so the CLI and
/// the desktop app render the same run identically.
pub fn format_duration(duration: Duration) -> String {
  let seconds: f64 = duration.as_secs_f64();

  if seconds < 1.0 {
    format!("{} ms", (seconds * 1000.0).round() as u64)
  } else if seconds < 60.0 {
    format!("{seconds:.1} s")
  } else {
    let minutes: u64 = (seconds / 60.0) as u64;

    format!("{minutes} m {} s", (seconds - (minutes * 60) as f64).round() as u64)
  }
}

/// Serialize a `Duration` as whole milliseconds, the workspace wire format for durations.
///
/// Use as `#[serde(with = "xrf_utils::duration_ms")]`. The field keeps its own wire name: milliseconds
/// are the standard duration unit on the consuming side, so the unit stays implicit there.
pub mod duration_ms {
  use std::time::Duration;

  use serde::{Deserialize, Deserializer, Serializer};

  pub fn serialize<S: Serializer>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(super::duration_to_millis(*duration))
  }

  pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    Ok(Duration::from_millis(u64::deserialize(deserializer)?))
  }
}

/// Serialize an optional `Duration` as whole milliseconds, or `null` when it was never measured.
///
/// Use as `#[serde(with = "xrf_utils::optional_duration_ms")]`. `serde(with)` does not reach through
/// an `Option` on its own: without this, an `Option<Duration>` falls back to serde's own `Duration`
/// representation and emits a `{ secs, nanos }` object beside fields that emit a millisecond count.
/// A report that answers the same question two ways is the thing this exists to prevent.
pub mod optional_duration_ms {
  use std::time::Duration;

  use serde::{Deserialize, Deserializer, Serializer};

  pub fn serialize<S: Serializer>(duration: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error> {
    match duration {
      Some(duration) => serializer.serialize_some(&super::duration_to_millis(*duration)),
      None => serializer.serialize_none(),
    }
  }

  pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Duration>, D::Error> {
    Ok(Option::<u64>::deserialize(deserializer)?.map(Duration::from_millis))
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use serde::{Deserialize, Serialize};

  use crate::{duration_to_millis, format_duration};

  #[test]
  fn test_duration_to_millis() {
    assert_eq!(duration_to_millis(Duration::ZERO), 0);
    assert_eq!(duration_to_millis(Duration::from_millis(742)), 742);
    assert_eq!(duration_to_millis(Duration::from_micros(1500)), 1);
    assert_eq!(duration_to_millis(Duration::MAX), u64::MAX);
  }

  #[test]
  fn test_serializes_durations_as_millisecond_counts() {
    #[derive(Serialize)]
    struct Measured {
      #[serde(with = "crate::duration_ms")]
      duration: Duration,
      #[serde(with = "crate::optional_duration_ms")]
      optional: Option<Duration>,
    }

    assert_eq!(
      serde_json::to_value(Measured {
        duration: Duration::from_millis(742),
        optional: Some(Duration::from_millis(7)),
      })
      .unwrap(),
      serde_json::json!({ "duration": 742, "optional": 7 })
    );

    assert_eq!(
      serde_json::to_value(Measured {
        duration: Duration::ZERO,
        optional: None,
      })
      .unwrap(),
      serde_json::json!({ "duration": 0, "optional": serde_json::Value::Null })
    );
  }

  #[test]
  fn test_reads_back_a_millisecond_count() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Measured {
      #[serde(with = "crate::duration_ms")]
      duration: Duration,
      #[serde(with = "crate::optional_duration_ms")]
      optional: Option<Duration>,
    }

    assert_eq!(
      serde_json::from_str::<Measured>(r#"{"duration":742,"optional":null}"#).unwrap(),
      Measured {
        duration: Duration::from_millis(742),
        optional: None,
      }
    );
  }

  #[test]
  fn test_format_duration_sub_second_as_millis() {
    assert_eq!(format_duration(Duration::ZERO), "0 ms");
    assert_eq!(format_duration(Duration::from_millis(742)), "742 ms");
    assert_eq!(format_duration(Duration::from_micros(999_400)), "999 ms");
  }

  #[test]
  fn test_format_duration_sub_minute_as_seconds() {
    assert_eq!(format_duration(Duration::from_millis(1000)), "1.0 s");
    assert_eq!(format_duration(Duration::from_millis(3240)), "3.2 s");
    // 59.96 s rounds up within the seconds branch, exactly as the UI's formatDuration renders it.
    assert_eq!(format_duration(Duration::from_millis(59_960)), "60.0 s");
  }

  #[test]
  fn test_format_duration_minutes_and_seconds() {
    assert_eq!(format_duration(Duration::from_secs(60)), "1 m 0 s");
    assert_eq!(format_duration(Duration::from_secs(72)), "1 m 12 s");
    assert_eq!(format_duration(Duration::from_millis(241_833)), "4 m 2 s");
  }
}
