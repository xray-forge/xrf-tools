use std::sync::Arc;

use xrf_audio::{AudioBus, AudioOutputChoice, AudioTransportPlayback};

use crate::{AudioEngineState, AudioSettings, AudioVolumes, PreviewTransports};

/// A 16-bit mono PCM WAV of `frames` silent frames at 48 kHz.
fn silent_wav(frames: u32) -> Arc<[u8]> {
  let data_length: u32 = frames * 2;
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(b"RIFF");
  bytes.extend_from_slice(&(36 + data_length).to_le_bytes());
  bytes.extend_from_slice(b"WAVEfmt ");
  bytes.extend_from_slice(&16u32.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&48_000u32.to_le_bytes());
  bytes.extend_from_slice(&96_000u32.to_le_bytes());
  bytes.extend_from_slice(&2u16.to_le_bytes());
  bytes.extend_from_slice(&16u16.to_le_bytes());
  bytes.extend_from_slice(b"data");
  bytes.extend_from_slice(&data_length.to_le_bytes());
  bytes.resize(bytes.len() + data_length as usize, 0);

  bytes.into()
}

/// Settings stored that do not read whole leave the store to its defaults rather than a half-read mix.
#[test]
fn refuses_stored_settings_that_do_not_read_whole() {
  assert!(serde_json::from_str::<AudioSettings>(r#"{"volumes":{"music":0.5}}"#).is_err());
  assert_eq!(AudioSettings::default().output, AudioOutputChoice::SystemDefault);
  assert!(AudioSettings::default().is_muted_in_background);
}

#[test]
fn holds_every_volume_in_range() {
  let settings: AudioSettings = AudioSettings {
    volumes: AudioVolumes {
      master: 2.0,
      music: -1.0,
      ambience: f32::NAN,
      world: 0.25,
      previews: 1.0,
    },
    ..AudioSettings::default()
  }
  .clamp();

  assert_eq!(settings.volumes.master, 1.0);
  assert_eq!(settings.volumes.get_bus(AudioBus::Music), 0.0);
  assert_eq!(settings.volumes.get_bus(AudioBus::Ambience), 1.0);
  assert_eq!(settings.volumes.get_bus(AudioBus::World), 0.25);
}

#[test]
fn round_trips_settings_through_their_wire_shape() {
  let settings: AudioSettings = AudioSettings {
    output: AudioOutputChoice::Device {
      id: String::from("speakers"),
    },
    volumes: AudioVolumes {
      previews: 0.5,
      ..AudioVolumes::default()
    },
    is_muted_in_background: false,
  };
  let written: String = serde_json::to_string(&settings).expect("settings write");

  assert!(
    written.contains(r#""output":{"kind":"device","id":"speakers"}"#),
    "{written}"
  );
  assert_eq!(
    serde_json::from_str::<AudioSettings>(&written).expect("settings read"),
    settings
  );
}

#[test]
fn opens_steers_and_closes_a_preview() {
  let transports: PreviewTransports = PreviewTransports::start(AudioEngineState::new(AudioSettings::default()));
  let id = transports.open(silent_wav(48_000), "main").expect("the preview opens");

  transports
    .seek(id, std::time::Duration::from_millis(250))
    .expect("the preview seeks");
  assert_eq!(
    transports.describe(id).expect("described").playback,
    AudioTransportPlayback::Paused
  );
  assert_eq!(
    transports.describe(id).expect("described").position,
    std::time::Duration::from_millis(250)
  );
  assert_eq!(transports.read_peaks(id, 4).expect("peaks read").peaks, vec![0.0; 4]);

  transports.set_looped(id, true).expect("the preview loops");
  assert!(transports.describe(id).expect("described").is_looped);

  transports.close(id);
  assert!(transports.describe(id).is_err());
  assert!(transports.play(id).is_err());

  transports.shut_down();
}

#[test]
fn refuses_to_preview_what_is_not_a_sound() {
  let transports: PreviewTransports = PreviewTransports::start(AudioEngineState::new(AudioSettings::default()));

  assert!(transports.open(Arc::from(&b"no sound"[..]), "main").is_err());

  transports.shut_down();
}

/// A window's page that loads again or goes closes its own previews, and another window's play on.
#[test]
fn closes_only_the_previews_of_the_window_that_went() {
  let transports: PreviewTransports = PreviewTransports::start(AudioEngineState::new(AudioSettings::default()));
  let main = transports.open(silent_wav(4_800), "main").expect("the preview opens");
  let other = transports.open(silent_wav(4_800), "other").expect("the preview opens");

  transports.close_window("main");

  assert!(transports.describe(main).is_err());
  assert!(transports.describe(other).is_ok());

  transports.shut_down();
  assert!(transports.describe(other).is_err());
}

/// The application mutes in the background only once no window has the focus, whatever order windows report it in,
/// and never before any window reported it.
#[test]
fn mutes_in_the_background_once_no_window_has_the_focus() {
  let engine: AudioEngineState = AudioEngineState::new(AudioSettings::default());

  assert!(!engine.is_muted());

  engine.set_window_focused("main", true);
  engine.set_window_focused("other", true);
  engine.set_window_focused("main", false);
  assert!(!engine.is_muted());

  engine.set_window_focused("other", false);
  assert!(engine.is_muted());

  engine.apply(AudioSettings {
    is_muted_in_background: false,
    ..AudioSettings::default()
  });
  assert!(!engine.is_muted());
}
