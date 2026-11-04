use std::sync::Arc;

use xrf_audio::{
  AudioBus, AudioEngineParts, AudioEngineSettings, AudioOutputChoice, AudioServer, AudioSpatialization,
  AudioTransportPlayback,
};

use crate::{AudioFocus, AudioSettings, AudioVolumes, PreviewEvents, PreviewTransports};

/// Previews on a server that plays to nothing: no device opens in a test.
fn transports() -> PreviewTransports {
  let AudioEngineParts { engine, .. } = AudioEngineParts::new(AudioEngineSettings::default());
  let events: PreviewEvents = PreviewEvents::default();
  let server: AudioServer = AudioServer::start_on(engine, None, events.clone()).expect("the server starts");

  PreviewTransports::new(server, &events)
}

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
    spatialization: AudioSpatialization::Speakers,
  };
  let written: String = serde_json::to_string(&settings).expect("settings write");

  assert!(
    written.contains(r#""output":{"kind":"device","id":"speakers"}"#),
    "{written}"
  );
  assert!(written.contains(r#""spatialization":"speakers""#), "{written}");
  assert_eq!(
    serde_json::from_str::<AudioSettings>(&written).expect("settings read"),
    settings
  );
}

// Settings kept before placed sound had a choice still read whole, as headphones, rather than falling to the defaults.
#[test]
fn reads_settings_kept_before_the_spatialization_choice() {
  let kept: &str = r#"{"output":{"kind":"systemDefault"},"volumes":{"master":0.5,"music":1,"ambience":1,"world":1,
    "previews":1},"isMutedInBackground":false}"#;
  let settings: AudioSettings = serde_json::from_str(kept).expect("settings read");

  assert_eq!(settings.volumes.master, 0.5);
  assert_eq!(settings.spatialization, AudioSpatialization::Headphones);
}

#[test]
fn opens_steers_and_closes_a_preview() {
  let transports: PreviewTransports = transports();
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
  let transports: PreviewTransports = transports();

  assert!(transports.open(Arc::from(&b"no sound"[..]), "main").is_err());

  transports.shut_down();
}

/// A window's page that loads again or goes closes its own previews, and another window's play on.
#[test]
fn closes_only_the_previews_of_the_window_that_went() {
  let transports: PreviewTransports = transports();
  let main = transports.open(silent_wav(4_800), "main").expect("the preview opens");
  let other = transports.open(silent_wav(4_800), "other").expect("the preview opens");

  transports.close_window("main");

  assert!(transports.describe(main).is_err());
  assert!(transports.describe(other).is_ok());

  transports.shut_down();
  assert!(transports.describe(other).is_err());
}

/// The application goes to the background only once no window has the focus, whatever order windows report it in,
/// and never before any window reported it.
#[test]
fn goes_to_the_background_once_no_window_has_the_focus() {
  let focus: AudioFocus = AudioFocus::default();

  assert!(!focus.is_in_background());

  assert!(!focus.set_window_focused("main", true));
  assert!(!focus.set_window_focused("other", true));
  assert!(!focus.set_window_focused("main", false));
  assert!(!focus.is_in_background());

  assert!(focus.set_window_focused("other", false));
  assert!(focus.is_in_background());

  assert!(focus.set_window_focused("other", true));
}

/// The background silences the master only where the settings say so; the buses keep their volumes.
#[test]
fn silences_the_master_in_the_background_where_the_settings_say_so() {
  let settings: AudioSettings = AudioSettings {
    volumes: AudioVolumes {
      master: 0.8,
      music: 0.5,
      ..AudioVolumes::default()
    },
    ..AudioSettings::default()
  };

  assert_eq!(settings.get_gains(false).master, 0.8);
  assert_eq!(settings.get_gains(true).master, 0.0);
  assert_eq!(settings.get_gains(true).get_bus(AudioBus::Music), 0.5);

  let audible: AudioSettings = AudioSettings {
    is_muted_in_background: false,
    ..settings
  };

  assert_eq!(audible.get_gains(true).master, 0.8);
}
