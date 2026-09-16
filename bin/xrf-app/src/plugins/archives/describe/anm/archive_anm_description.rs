use serde::Serialize;
use xrf_db::{AnmFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::anm::archive_anm_channel::ArchiveAnmChannel;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;

/// Everything the viewer says about one object motion.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAnmDescription {
  /// The name the editor saved it under, absent for the 1,051 of 1,221 shipped animations carrying none.
  pub name: Option<String>,
  pub version: u16,
  pub frame_start: i32,
  pub frame_end: i32,
  /// Frames the range spans, counting both ends, which is the engine's `Length`.
  pub frames: u32,
  pub fps: f32,
  /// Seconds the engine plays it for, which is what a camera effect's lifetime is taken from.
  pub duration_seconds: f32,
  /// Keys across every channel.
  pub keys: usize,
  /// Seconds the last key of any channel sits at, absent when nothing is keyed.
  pub keyed_seconds: Option<f32>,
  /// One entry per channel, in the order the format stores them.
  pub channels: Vec<ArchiveAnmChannel>,
}

impl ArchiveAnmDescription {
  /// Reads the object motion an entry holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not an object motion this reader can
  /// walk - a version it does not implement, or a payload not accounting for six envelopes.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    Ok(Self::of(&AnmFile::read_from_bytes::<XRayByteOrder>(
      source.read_bytes(name)?,
    )?))
  }

  /// What an animation already in hand says about itself.
  pub fn of(file: &AnmFile) -> Self {
    let channels: Vec<ArchiveAnmChannel> = ArchiveAnmChannel::of_all(&file.channels);

    Self {
      name: Some(file.name.clone()).filter(|name| !name.is_empty()),
      version: file.version,
      frame_start: file.frame_start,
      frame_end: file.frame_end,
      frames: file.get_frame_count(),
      fps: file.fps,
      duration_seconds: file.get_duration_seconds(),
      keys: file.get_keys_count(),
      keyed_seconds: channels
        .iter()
        .filter_map(|channel| channel.last_seconds)
        .reduce(f32::max),
      channels,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_db::{AnmEnvelope, AnmFile, AnmKey};

  use super::ArchiveAnmDescription;

  fn key(value: f32, time: f32) -> AnmKey {
    AnmKey {
      value,
      time,
      shape: 0,
      interpolation: None,
    }
  }

  fn animation(frame_start: i32, frame_end: i32, keys: Vec<AnmKey>) -> AnmFile {
    AnmFile {
      name: String::new(),
      frame_start,
      frame_end,
      fps: 30.0,
      version: 5,
      channels: (0..AnmFile::CHANNEL_COUNT)
        .map(|index| AnmEnvelope {
          behavior: (1, 1),
          keys: if index == 0 { keys.clone() } else { Vec::new() },
        })
        .collect(),
    }
  }

  #[test]
  fn an_animation_is_as_long_as_the_engine_would_play_it() {
    let described: ArchiveAnmDescription = ArchiveAnmDescription::of(&animation(0, 59, vec![key(0.0, 0.0)]));

    assert_eq!(described.frames, 60);
    assert_eq!(described.duration_seconds, 2.0);
    assert_eq!(described.channels.len(), AnmFile::CHANNEL_COUNT);
  }

  #[test]
  fn an_animation_saved_under_no_name_carries_none_rather_than_an_empty_one() {
    assert_eq!(ArchiveAnmDescription::of(&animation(0, 59, Vec::new())).name, None);

    let mut named: AnmFile = animation(0, 59, Vec::new());

    named.name = String::from("camera_shake");

    assert_eq!(
      ArchiveAnmDescription::of(&named).name,
      Some(String::from("camera_shake"))
    );
  }

  #[test]
  fn the_keyed_reach_is_the_furthest_key_of_any_channel() {
    // `camera_effects\earthquake_00.anm`: keys running to 9.967 s inside a range declaring 4.
    let described: ArchiveAnmDescription =
      ArchiveAnmDescription::of(&animation(0, 119, vec![key(0.0, 0.0), key(1.0, 9.967)]));

    assert_eq!(described.duration_seconds, 4.0);
    assert_eq!(described.keyed_seconds, Some(9.967));
    assert_eq!(described.keys, 2);
  }

  #[test]
  fn an_animation_keyed_on_nothing_reaches_nowhere() {
    let described: ArchiveAnmDescription = ArchiveAnmDescription::of(&animation(0, 59, Vec::new()));

    assert_eq!(described.keys, 0);
    assert_eq!(described.keyed_seconds, None);
  }
}
