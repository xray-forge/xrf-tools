use std::sync::Arc;

use xrf_app_core::session::SessionSnapshot;
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxProject, LtxResolution};
use xrf_vfs::{XrayLogicalPath, XraySearch};
use xrf_world::{WorldLevelSounds, WorldMusicTrack};

use crate::level::{LevelConfigs, SelectedLevel};
use crate::objects::LevelObjectsDescription;
use crate::sound::{LevelSoundEmitters, LevelSoundFiles};

/// What a level plays whatever its weather, as the world takes it (`CLevelSoundManager::Load`, `CCustomZone::Load`):
/// its static sounds and zones' sounds (`LevelSoundEmitters`), and the music tracks of the section its `game.ltx`
/// section names by `music_tracks`, each sound read for what its file says.
pub struct LevelWorldSounds;

impl LevelWorldSounds {
  /// The config the level's music is named in.
  const GAME: &'static str = "game.ltx";

  /// Reads the level's sounds through `search`, each emitter named by its level object's place in the listing, none
  /// where the listing cannot be read, which its report says.
  ///
  /// # Errors
  ///
  /// Returns an error when the game's configs cannot be opened.
  pub fn read(level: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> XrfResult<WorldLevelSounds> {
    let objects: Option<LevelObjectsDescription> = LevelObjectsDescription::read(level, search).ok();
    let mut files: LevelSoundFiles<'_> = LevelSoundFiles::new(search);
    let emitters: LevelSoundEmitters = LevelSoundEmitters::read(level, search, objects.as_ref(), &mut files);
    let music: Vec<WorldMusicTrack> = Self::read_music(level, search, &mut files)?;

    Ok(WorldLevelSounds {
      engine: level.engine.engine,
      statics: emitters.statics,
      music,
      zones: emitters.zones,
    })
  }

  /// The tracks of the level's music section, in the config's order, each sound read through `files`; none where
  /// `game.ltx` or the section names none.
  fn read_music(
    level: &SessionSnapshot<SelectedLevel>,
    search: &XraySearch,
    files: &mut LevelSoundFiles<'_>,
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
            .map(|(track, parameters)| Self::to_track(track, parameters, files))
            .collect()
        })
        .unwrap_or_default(),
    )
  }

  /// `SMusicTrack::Load`: `from_hour, to_hour, volume, pause_from_s, pause_to_s` read as `sscanf` reads them, each value
  /// it cannot read left at its default; a pause whose two ends are equal widens by one.
  fn to_track(track: &str, parameters: &str, files: &mut LevelSoundFiles<'_>) -> WorldMusicTrack {
    let values: Vec<&str> = parameters.split(',').map(str::trim).collect();
    let integer = |index: usize| values.get(index).and_then(|value| value.parse::<i64>().ok());
    let from_hour: i64 = integer(0).unwrap_or(0);
    let to_hour: i64 = integer(1).unwrap_or(0);
    let volume: f32 = values.get(2).and_then(|value| value.parse::<f32>().ok()).unwrap_or(1.0);
    let mut pause: [i64; 2] = [integer(3).unwrap_or(0), integer(4).unwrap_or(0)];

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
}
