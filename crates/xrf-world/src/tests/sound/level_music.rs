//! A level's music, `CLevelSoundManager::Update`: which track is picked by the game's hour, how the engines pick, and
//! OpenXRay's halves where the stereo file is missing.

use std::collections::HashSet;
use std::time::Duration;

use xrf_engine_target::XrayEngine;

use crate::contract::sound::world_music_command::WorldMusicCommand;
use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::contract::sound::world_sound_music_report::WorldSoundMusicReport;
use crate::host::sound::world_music_track::WorldMusicTrack;
use crate::sound::level_music::LevelMusic;
use crate::sound::sound_emitters::SoundEmitters;
use crate::tests::sound::{found, missing, seconds};

const HOUR: u32 = 3_600_000;

fn track(name: &str, hours: [u32; 2]) -> WorldMusicTrack {
  WorldMusicTrack {
    name: name.to_owned(),
    stereo: found(name, 10),
    halves: None,
    hours: [hours[0] * HOUR, hours[1] * HOUR],
    volume: 0.8,
    pause: [1000, 1001],
  }
}

/// The tracks picked over `ticks` ticks of 100 ms at `game_time`, in order.
fn play(music: &mut LevelMusic, game_time: u32, ticks: u64) -> Vec<String> {
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let mut picked: Vec<String> = Vec::new();

  for tick in 1..=ticks {
    let now: Duration = Duration::from_millis(tick * 100);

    music.update(&mut emitters, game_time, now);
    emitters.retire(now);

    if let Some(track) = music.report(&emitters, now).track
      && picked.last() != Some(&track)
    {
      picked.push(track);
    }
  }

  picked
}

#[test]
fn plays_a_track_2d_at_its_volume_within_its_hours() {
  let mut music: LevelMusic = LevelMusic::new(
    vec![track("day", [6, 20]), track("night", [21, 5])],
    XrayEngine::Vanilla,
  );
  let mut emitters: SoundEmitters = SoundEmitters::default();

  music.update(&mut emitters, 12 * HOUR, seconds(1));

  let playing: Vec<_> = emitters.iter_playing(seconds(1)).collect();

  assert_eq!(playing.len(), 1);
  assert_eq!(&*playing[0].sound, "day");
  assert_eq!(playing[0].kind, WorldSoundKind::Music);
  assert!(playing[0].is_2d && !playing[0].is_looped);
  assert_eq!(playing[0].volume, 0.8);

  // Hours crossing midnight hold either side of it.
  assert_eq!(
    play(
      &mut LevelMusic::new(vec![track("night", [21, 5])], XrayEngine::Vanilla),
      2 * HOUR,
      5
    ),
    ["night"]
  );
}

// `in`: a track from hour nought to any other is eligible at any hour.
#[test]
fn takes_a_track_from_midnight_as_always_eligible() {
  let mut music: LevelMusic = LevelMusic::new(vec![track("dawn", [0, 4])], XrayEngine::Vanilla);

  assert_eq!(play(&mut music, 15 * HOUR, 5), ["dawn"]);
}

// With none eligible the music checks again ten seconds later.
#[test]
fn waits_ten_seconds_when_no_track_is_eligible() {
  let mut music: LevelMusic = LevelMusic::new(vec![track("night", [21, 23])], XrayEngine::Vanilla);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  music.update(&mut emitters, 12 * HOUR, seconds(1));

  assert_eq!(music.report(&emitters, seconds(1)).wait, seconds(10));
}

// Monolith shuffles once and plays every eligible track before any again; OpenXRay draws each pick.
#[test]
fn plays_every_track_once_before_repeating_on_monolith() {
  let tracks: Vec<WorldMusicTrack> = (0..5).map(|index| track(&format!("t{index}"), [1, 23])).collect();
  let picked: Vec<String> = play(&mut LevelMusic::new(tracks, XrayEngine::Extended), 12 * HOUR, 600);
  let first: HashSet<&String> = picked.iter().take(5).collect();

  assert!(picked.len() >= 5, "{picked:?}");
  assert_eq!(first.len(), 5, "{picked:?}");
}

// OpenXRay plays a track's halves either side where its stereo file is missing; Monolith plays nothing of it.
#[test]
fn plays_a_missing_tracks_halves_on_openxray_only() {
  let halved = || WorldMusicTrack {
    stereo: missing("split"),
    halves: Some([found("split_l", 10), found("split_r", 10)]),
    ..track("split", [1, 23])
  };
  let mut emitters: SoundEmitters = SoundEmitters::default();

  LevelMusic::new(vec![halved()], XrayEngine::Vanilla).update(&mut emitters, 12 * HOUR, seconds(1));

  let positions: Vec<[f32; 3]> = emitters.iter_playing(seconds(1)).map(|it| it.position).collect();

  assert_eq!(positions, [[-0.5, 0.0, 0.3], [0.5, 0.0, 0.3]]);

  let mut emitters: SoundEmitters = SoundEmitters::default();

  LevelMusic::new(vec![halved()], XrayEngine::Extended).update(&mut emitters, 12 * HOUR, seconds(1));

  assert_eq!(emitters.iter_playing(seconds(1)).count(), 0);
}

// A track played by hand plays at once whatever the hour, and again each time it ends with no pause, until the
// schedule is asked back, which picks at once.
#[test]
fn plays_a_track_by_hand_over_and_over_until_the_schedule_is_back() {
  let mut music: LevelMusic = LevelMusic::new(
    vec![track("day", [6, 20]), track("night", [21, 5])],
    XrayEngine::Vanilla,
  );
  let mut emitters: SoundEmitters = SoundEmitters::default();

  music.update(&mut emitters, 12 * HOUR, seconds(1));

  assert!(!music.command(
    &mut emitters,
    &WorldMusicCommand::Play {
      track: String::from("dusk")
    },
    seconds(2)
  ));
  assert!(music.command(
    &mut emitters,
    &WorldMusicCommand::Play {
      track: String::from("night")
    },
    seconds(2)
  ));

  let report: WorldSoundMusicReport = music.report(&emitters, seconds(2));

  assert_eq!(report.track.as_deref(), Some("night"));
  assert_eq!(report.chosen.as_deref(), Some("night"));
  assert_eq!(report.tracks, ["day", "night"]);
  assert_eq!(report.length, seconds(10));
  assert_eq!(emitters.iter_playing(seconds(2)).count(), 1);

  // Its ten seconds over, it starts again on the next update.
  for tick in 21..=125 {
    let now: Duration = Duration::from_millis(tick * 100);

    music.update(&mut emitters, 12 * HOUR, now);
    emitters.retire(now);
  }

  let again: WorldSoundMusicReport = music.report(&emitters, Duration::from_millis(12_500));

  assert_eq!(again.track.as_deref(), Some("night"));
  assert!(again.position < seconds(1), "{again:?}");

  assert!(music.command(&mut emitters, &WorldMusicCommand::Schedule, seconds(13)));
  music.update(&mut emitters, 12 * HOUR, Duration::from_millis(13_100));

  let scheduled: WorldSoundMusicReport = music.report(&emitters, Duration::from_millis(13_100));

  assert_eq!(scheduled.track.as_deref(), Some("day"));
  assert_eq!(scheduled.chosen, None);
}

// A seek plays the track playing from the point asked, which the report then stands at; none plays between tracks.
#[test]
fn plays_the_track_playing_from_a_point_asked() {
  let mut music: LevelMusic = LevelMusic::new(vec![track("day", [1, 23])], XrayEngine::Extended);
  let mut emitters: SoundEmitters = SoundEmitters::default();

  assert!(!music.command(&mut emitters, &WorldMusicCommand::Seek { position: 4000 }, seconds(1)));

  music.update(&mut emitters, 12 * HOUR, seconds(1));

  let first: u64 = emitters.iter_playing(seconds(1)).next().unwrap().id;

  assert!(music.command(&mut emitters, &WorldMusicCommand::Seek { position: 7000 }, seconds(2)));

  let seeked: Vec<_> = emitters.iter_playing(seconds(2)).collect();

  assert_eq!(seeked.len(), 1);
  assert_ne!(seeked[0].id, first);
  assert_eq!(seeked[0].offset, seconds(7));
  assert_eq!(music.report(&emitters, seconds(3)).position, seconds(8));
  // Three seconds were left of it.
  assert!(emitters.is_playing(seeked[0].id, Duration::from_millis(4900)));
  assert!(!emitters.is_playing(seeked[0].id, seconds(5)));
}

// Monolith plays music `sm_Intro`, which feeds no reverb.
#[test]
fn sends_no_reverb_for_monolith_music() {
  let mut emitters: SoundEmitters = SoundEmitters::default();

  LevelMusic::new(vec![track("day", [1, 23])], XrayEngine::Extended).update(&mut emitters, 12 * HOUR, seconds(1));

  assert!(emitters.iter_playing(seconds(1)).all(|it| !it.has_reverb));
}
