use std::io::Write;
use std::time::Duration;

use xrf_animation_envelope::{AnimationEnvelope, AnimationKey, AnimationPlayback, AnimationShape};
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;

use crate::{PpeColor, PpeColorMap, PpeFile, PpeScreen};

/// One interpolating key, as `st_Key::Load_2` lays it out: value, time, shape, then seven quantised parameters.
fn new_key_bytes(value: f32, time: f32) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&value.to_le_bytes());
  bytes.extend_from_slice(&time.to_le_bytes());
  bytes.push(0);
  bytes.extend_from_slice(&[0u8; 7 * 2]);

  bytes
}

/// An envelope: the two behaviours, the key count, then the keys.
fn new_envelope_bytes(keys: &[Vec<u8>]) -> Vec<u8> {
  let mut bytes: Vec<u8> = vec![1, 1];

  bytes.extend_from_slice(&(keys.len() as u16).to_le_bytes());

  for key in keys {
    bytes.extend_from_slice(key);
  }

  bytes
}

/// A colour parameter: the unread base, then one envelope per channel.
fn new_color_bytes(keys: &[Vec<u8>]) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&0.5f32.to_le_bytes());
  bytes.extend(new_envelope_bytes(keys));
  bytes.extend(new_envelope_bytes(&[]));
  bytes.extend(new_envelope_bytes(&[]));

  bytes
}

/// A whole effect, flat, exactly as `BasicPostProcessAnimator::Load` reads one.
fn new_effect_bytes(version: u32) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&version.to_le_bytes());

  // Base colour carries the only keyed channel; the other two colours are empty throughout.
  bytes.extend(new_color_bytes(&[new_key_bytes(0.0, 0.0), new_key_bytes(1.0, 3.0)]));
  bytes.extend(new_color_bytes(&[]));
  bytes.extend(new_color_bytes(&[]));

  // Gray value is keyed and runs longer than the colour; the six remaining scalars are empty.
  bytes.extend(new_envelope_bytes(&[new_key_bytes(0.0, 0.5), new_key_bytes(0.8, 5.5)]));

  for _ in 1..7 {
    bytes.extend(new_envelope_bytes(&[]));
  }

  if version >= PpeFile::COLOR_MAP_VERSION {
    bytes.extend(new_envelope_bytes(&[new_key_bytes(1.0, 0.0)]));
    bytes.extend_from_slice(b"grad\\grad_psi\0");
  }

  bytes
}

/// An envelope rising in a straight line through `(time, value)` keys.
fn line(keys: &[(f32, f32)]) -> AnimationEnvelope {
  AnimationEnvelope {
    keys: keys
      .iter()
      .map(|&(time, value)| AnimationKey {
        value,
        time,
        shape: AnimationShape::Linear,
        ..AnimationKey::default()
      })
      .collect(),
    ..AnimationEnvelope::default()
  }
}

#[test]
fn an_effect_reads_as_the_version_and_the_parameters_it_is() -> XrfResult {
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(2))?;

  assert_eq!(effect.get_version(), 2);
  assert_eq!(effect.base_color.red.keys.len(), 2);
  assert!(effect.base_color.green.keys.is_empty());
  assert_eq!(effect.gray_value.keys.len(), 2);
  assert!(effect.noise_fps.keys.is_empty());

  Ok(())
}

#[test]
fn a_version_one_effect_carries_no_color_map_at_all() -> XrfResult {
  // Absent, rather than present and empty: the engine never reads the parameter for such a file.
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(1))?;

  assert_eq!(effect.get_version(), 1);
  assert_eq!(effect.color_map, None);

  Ok(())
}

#[test]
fn a_version_two_effect_carries_the_grading_and_the_texture_it_names() -> XrfResult {
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(2))?;
  let color_map: PpeColorMap = effect.color_map.expect("a version 2 effect carries a color map");

  assert_eq!(color_map.texture, "grad\\grad_psi");
  assert_eq!(color_map.influence.keys.len(), 1);
  assert!(color_map.is_used());

  Ok(())
}

#[test]
fn an_effect_is_written_back_byte_for_byte() -> XrfResult {
  for version in PpeFile::SUPPORTED_VERSIONS {
    let original: Vec<u8> = new_effect_bytes(version);
    let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

    let mut writer: ChunkWriter = ChunkWriter::new();

    effect.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, original, "version {version} did not round trip");
  }

  Ok(())
}

#[test]
fn an_effect_is_written_in_the_version_its_color_map_decides() -> XrfResult {
  // The engine's own save always writes 2, which would append a grading a version 1 author never stored.
  let mut effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(2))?;

  effect.color_map = None;

  let mut writer: ChunkWriter = ChunkWriter::new();

  effect.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, new_effect_bytes(1));
  assert_ne!(effect.get_version(), PpeFile::CURRENT_VERSION);
  assert_eq!(PpeFile::default().get_version(), PpeFile::FIRST_VERSION);

  Ok(())
}

#[test]
fn a_version_nothing_here_reads_is_refused_by_name() {
  let mut bytes: Vec<u8> = 7u32.to_le_bytes().to_vec();

  bytes.extend(new_effect_bytes(2).into_iter().skip(4));

  let error: String = PpeFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("an unknown version is refused")
    .to_string();

  assert!(error.contains('7'), "'{error}' names the version it refused");
}

#[test]
fn a_payload_the_parameters_do_not_account_for_is_refused() -> XrfResult {
  let mut bytes: Vec<u8> = new_effect_bytes(2);

  bytes.extend_from_slice(&[0, 0, 0, 0]);

  assert!(PpeFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

#[test]
fn each_envelope_is_named_as_an_author_knows_its_channel() -> XrfResult {
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(2))?;
  let named: Vec<(String, &AnimationEnvelope)> = effect.get_named_envelopes();
  let names: Vec<&str> = named.iter().map(|(name, _)| name.as_str()).collect();

  // Nine colour channels, seven scalars and the grading's influence, in the order `get_envelopes` walks them.
  assert_eq!(names.len(), effect.get_envelopes().count());
  assert_eq!(
    &names[..4],
    ["base color red", "base color green", "base color blue", "add color red"]
  );
  assert_eq!(names[9], "gray value");
  assert_eq!(names.last(), Some(&"colour map influence"));
  assert_eq!(named[9].1, &effect.gray_value);

  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(1))?;

  assert_eq!(
    effect.get_named_envelopes().last().map(|(name, _)| name.as_str()),
    Some("noise fps")
  );

  Ok(())
}

#[test]
fn an_effect_runs_as_long_as_its_longest_parameter() -> XrfResult {
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(2))?;

  // The base colour spans 3 s and the gray value 5, so the effect is 5 - not 5.5, which is where its last key sits.
  assert_eq!(effect.get_length_seconds(), 5.0);
  assert_eq!(effect.get_keys_count(), 5);
  assert!(effect.is_keyed());

  Ok(())
}

#[test]
fn an_effect_keyed_on_nothing_runs_for_no_time() {
  let effect: PpeFile = PpeFile::default();

  assert_eq!(effect.get_length_seconds(), 0.0);
  assert_eq!(effect.get_keys_count(), 0);
  assert!(!effect.is_keyed());
}

#[test]
fn a_colour_counts_the_keys_of_all_three_channels_rather_than_the_reds() -> XrfResult {
  // `CPostProcessColor::get_keys_count` answers with the red channel alone, which is an editor convenience.
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(2))?;

  assert_eq!(effect.base_color.get_keys_count(), 2);
  assert_eq!(effect.base_color.get_length_seconds(), 3.0);

  Ok(())
}

#[test]
fn an_effect_reads_the_same_off_a_reader_as_off_its_bytes() -> XrfResult {
  let bytes: Vec<u8> = new_effect_bytes(2);
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_all(&bytes)?;

  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(bytes.clone())?;

  assert_eq!(
    PpeFile::read::<XRayByteOrder, _>(&mut reader)?,
    PpeFile::read_from_bytes::<XRayByteOrder>(bytes)?
  );

  Ok(())
}

#[test]
fn an_effect_keyed_on_nothing_leaves_the_screen_as_the_identity_has_it() {
  assert_eq!(PpeFile::default().evaluate(1.5), PpeScreen::IDENTITY);
}

#[test]
fn an_effect_adds_its_colours_to_the_identity_and_plays_its_values_as_keyed() {
  // `CPostprocessAnimator::Process` adds `pp_identity`'s colours, and multiplies a keyed noise fps by 100.
  let effect: PpeFile = PpeFile {
    base_color: PpeColor {
      red: line(&[(0.0, 0.0), (2.0, 0.2)]),
      ..PpeColor::default()
    },
    blur: line(&[(0.0, 0.0), (2.0, 1.0)]),
    noise_granularity: line(&[(0.0, 0.4)]),
    noise_fps: line(&[(0.0, 0.2)]),
    color_map: Some(PpeColorMap {
      influence: line(&[(0.0, 0.0), (2.0, 0.5)]),
      texture: String::from("grad\\grad_psi"),
    }),
    ..PpeFile::default()
  };
  let screen: PpeScreen = effect.evaluate(1.0);

  assert!((screen.base_color[0] - 0.6).abs() < 1e-6, "{screen:?}");
  assert_eq!(screen.base_color[1..], [0.5, 0.5]);
  assert_eq!(screen.gray_color, PpeScreen::IDENTITY.gray_color);
  assert_eq!(screen.blur, 0.5);
  assert_eq!(screen.noise_intensity, 0.0);
  assert_eq!(screen.noise_granularity, 0.4);
  assert!((screen.noise_fps - 20.0).abs() < 1e-4, "{screen:?}");
  assert_eq!(screen.color_map_influence, 0.25);
}

#[test]
fn a_noise_granularity_reaching_zero_is_found_where_the_engine_refuses_it() {
  // Falling from 1 to -1 over two seconds, the grain is zero at the middle of its span.
  let falling: PpeFile = PpeFile {
    noise_granularity: line(&[(0.0, 1.0), (2.0, -1.0)]),
    ..PpeFile::default()
  };

  assert_eq!(falling.find_refused_noise_granularity(), Some((1.0, 0.0)));
}

#[test]
fn a_positive_or_unkeyed_noise_granularity_is_never_refused() {
  let positive: PpeFile = PpeFile {
    noise_granularity: line(&[(0.0, 0.1), (2.0, 2.0)]),
    ..PpeFile::default()
  };

  assert_eq!(positive.find_refused_noise_granularity(), None);
  // Unkeyed, it keeps the identity's 1, though the envelope itself evaluates to 0.
  assert_eq!(PpeFile::default().find_refused_noise_granularity(), None);
}

#[test]
fn a_noise_granularity_below_zero_only_before_the_effect_starts_is_never_played() {
  // The effect plays from zero for the blur's three seconds, and the grain has risen to 1 a second before it starts.
  let early: PpeFile = PpeFile {
    blur: line(&[(0.0, 0.0), (3.0, 1.0)]),
    noise_granularity: line(&[(-2.0, -1.0), (-1.0, 1.0)]),
    ..PpeFile::default()
  };

  assert_eq!(early.find_refused_noise_granularity(), None);
}

#[test]
fn an_effect_plays_from_zero_and_a_cyclic_one_starts_again_every_length() -> XrfResult {
  // `m_start_time` is the moment the effect began, not its first key, and a cyclic effect moves it on by
  // `GetLength`, the longest key span: 5 s here, though the last key sits at 5.5 s.
  let effect: PpeFile = PpeFile::read_from_bytes::<XRayByteOrder>(new_effect_bytes(2))?;
  let played =
    |seconds: f32, playback: AnimationPlayback| effect.get_played_time(Duration::from_secs_f32(seconds), playback);

  assert_eq!(played(0.25, AnimationPlayback::Looped), 0.25);
  assert_eq!(played(5.0, AnimationPlayback::Looped), 5.0);
  assert!((played(6.5, AnimationPlayback::Looped) - 1.5).abs() < 1e-5);
  assert_eq!(played(6.5, AnimationPlayback::Once), 6.5);

  // An effect of no length has nothing to start again.
  assert_eq!(
    PpeFile::default().get_played_time(Duration::from_secs(9), AnimationPlayback::Looped),
    9.0
  );

  Ok(())
}
