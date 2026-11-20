use std::sync::Arc;

use xrf_app_core::session::SessionSnapshot;
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxProject, LtxResolution};
use xrf_vfs::{XrayLogicalPath, XraySearch};
use xrf_world::{WorldLevelSounds, WorldMusicTrack};

use crate::level::{LevelConfigs, SelectedLevel};
use crate::objects::LevelObjectsDescription;
use crate::sound::{LevelSoundEmitters, LevelSoundFiles, LevelSoundFilesPart};

/// A level's weather-independent sounds for the world (`CLevelSoundManager::Load`, `CCustomZone::Load`): its emitters
/// (`LevelSoundEmitters`) and the music tracks its `game.ltx` section names by `music_tracks`.
pub(crate) struct LevelWorldSounds;

impl LevelWorldSounds {
  const GAME: &'static str = "game.ltx";

  /// Read the level's sounds; emitters are unnamed when the object listing cannot be read.
  ///
  /// # Errors
  ///
  /// Returns an error when the game's configs cannot be opened.
  pub(crate) fn read(level: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> XrfResult<WorldLevelSounds> {
    let objects: Option<LevelObjectsDescription> = LevelObjectsDescription::read(level, search).ok();
    let (emitters, music): (LevelSoundEmitters, XrfResult<Vec<WorldMusicTrack>>) =
      LevelSoundFilesPart::with_files(level, search, |files| {
        (
          LevelSoundEmitters::read(level, search, objects.as_ref(), files),
          Self::read_music(level, search, files),
        )
      });
    let music: Vec<WorldMusicTrack> = music?;

    Ok(WorldLevelSounds {
      engine: level.engine.engine,
      statics: emitters.statics,
      music,
      zones: emitters.zones,
    })
  }

  /// The level's music tracks in config order; none without `game.ltx` or a `music_tracks` section.
  fn read_music(
    level: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    files: &LevelSoundFiles<'_>,
  ) -> XrfResult<Vec<WorldMusicTrack>> {
    let name: String = level.source.get_name().unwrap_or_default();
    let project: LtxProject = LevelConfigs::of(level)
      .open(search)
      .map_err(XrfError::new_asset_error)?;
    let path: XrayLogicalPath = project.config_path(Self::GAME)?;

    if project.search().find(path.as_str())?.is_none() {
      return Ok(Vec::new());
    }

    let game: Arc<LtxResolution> = project.read_resolution(&path)?;
    let Some(section) = game
      .ltx
      .get_from(&name, "music_tracks")
      .map(str::trim)
      .filter(|section| !section.is_empty())
    else {
      return Ok(Vec::new());
    };

    Ok(
      game
        .ltx
        .section(section)
        .map(|tracks| {
          tracks
            .iter()
            .map(|(track, parameters)| Self::read_track(track, parameters, files))
            .collect()
        })
        .unwrap_or_default(),
    )
  }

  /// `SMusicTrack::Load`: `from_hour, to_hour, volume, pause_from_s, pause_to_s` as [`Self::scan_track`] reads them;
  /// a pause with equal ends widens by one second.
  fn read_track(track: &str, parameters: &str, files: &LevelSoundFiles<'_>) -> WorldMusicTrack {
    let ([from_hour, to_hour], volume, mut pause) = Self::scan_track(parameters);

    if pause[0] == pause[1] {
      pause[1] += 1;
    }

    let stereo = files.get(track);
    let halves = (stereo.info.is_none()).then(|| [files.get(&format!("{track}_l")), files.get(&format!("{track}_r"))]);
    let to_millis = |value: i64, scale: i64| u32::try_from(value.saturating_mul(scale).max(0)).unwrap_or(u32::MAX);

    WorldMusicTrack {
      name: track.to_owned(),
      stereo,
      halves,
      hours: [to_millis(from_hour, 3_600_000), to_millis(to_hour, 3_600_000)],
      volume,
      pause: [to_millis(pause[0], 1000), to_millis(pause[1], 1000)],
    }
  }

  /// A track's hours, volume and pause seconds as `sscanf(params, "%d,%d,%f,%d,%d", ...)` stores them: in turn until a
  /// value or the comma after it does not scan, each the longest number its conversion takes (`12abc` is 12), every
  /// later one keeping its default of 0, or 1 for the volume. Only spaces before a number are skipped, so `1 ,2` stops
  /// at the space.
  pub(crate) fn scan_track(parameters: &str) -> ([i64; 2], f32, [i64; 2]) {
    let mut hours: [i64; 2] = [0, 0];
    let mut volume: f32 = 1.0;
    let mut pause: [i64; 2] = [0, 0];

    // Where a value stops the scan, it and those after it keep what they hold.
    let _ = Self::scan_values(parameters, &mut hours, &mut volume, &mut pause);

    (hours, volume, pause)
  }

  /// Stores each value of `parameters` in turn, none once one does not scan.
  fn scan_values(parameters: &str, hours: &mut [i64; 2], volume: &mut f32, pause: &mut [i64; 2]) -> Option<()> {
    let mut rest: &str;

    (hours[0], rest) = Self::scan_integer(parameters)?;
    (hours[1], rest) = Self::scan_integer(rest.strip_prefix(',')?)?;
    (*volume, rest) = Self::scan_float(rest.strip_prefix(',')?)?;
    (pause[0], rest) = Self::scan_integer(rest.strip_prefix(',')?)?;
    (pause[1], _) = Self::scan_integer(rest.strip_prefix(',')?)?;

    Some(())
  }

  /// `%d`: spaces skipped, a sign, then digits, at least one; a number past `i64` saturates.
  fn scan_integer(text: &str) -> Option<(i64, &str)> {
    let text: &str = text.trim_start();
    let signed: usize = usize::from(text.starts_with(['+', '-']));
    let digits: usize = text[signed..].bytes().take_while(u8::is_ascii_digit).count();

    if digits == 0 {
      return None;
    }

    let (number, rest) = text.split_at(signed + digits);
    let value: i64 = number
      .parse::<i64>()
      .unwrap_or(if number.starts_with('-') { i64::MIN } else { i64::MAX });

    Some((value, rest))
  }

  /// `%f`: spaces skipped, a sign, digits with at most one point and at least one digit, then an exponent where digits
  /// follow its `e`; or `inf`, `infinity` or `nan` in any case.
  fn scan_float(text: &str) -> Option<(f32, &str)> {
    let text: &str = text.trim_start();
    let bytes: &[u8] = text.as_bytes();
    let mut end: usize = usize::from(text.starts_with(['+', '-']));

    for word in ["infinity", "inf", "nan"] {
      if text
        .get(end..end + word.len())
        .is_some_and(|it| it.eq_ignore_ascii_case(word))
      {
        let (number, rest) = text.split_at(end + word.len());

        return number.parse::<f32>().ok().map(|value| (value, rest));
      }
    }

    let integer: usize = bytes[end..].iter().take_while(|it| it.is_ascii_digit()).count();

    end += integer;

    let fraction: usize = if bytes.get(end) == Some(&b'.') {
      bytes[end + 1..].iter().take_while(|it| it.is_ascii_digit()).count()
    } else {
      0
    };

    if bytes.get(end) == Some(&b'.') {
      end += 1 + fraction;
    }

    if integer + fraction == 0 {
      return None;
    }

    if matches!(bytes.get(end), Some(b'e' | b'E')) {
      let sign: usize = usize::from(matches!(bytes.get(end + 1), Some(b'+' | b'-')));
      let exponent: usize = bytes[end + 1 + sign..]
        .iter()
        .take_while(|it| it.is_ascii_digit())
        .count();

      if exponent > 0 {
        end += 1 + sign + exponent;
      }
    }

    let (number, rest) = text.split_at(end);

    number.parse::<f32>().ok().map(|value| (value, rest))
  }
}
