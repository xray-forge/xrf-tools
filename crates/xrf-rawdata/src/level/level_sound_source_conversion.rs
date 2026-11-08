use xrf_editable_level::{
  EditableCustomObject, EditableLevelGuid, EditableLevelValue, EditableObjectPart, EditableObjectTransform,
  EditableSoundSourceObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelSndStaticFile, LevelSndStaticSound, LevelSndStaticWindow};
use xrf_math::Vector2d;

use crate::level::level_object_names::LevelObjectNames;

/// A level's planted sounds, between the scene's `sound_src.part` and the compiled `level.snd_static`, which the build
/// passes through with no compiler (`BuilderGame.cpp:65`).
pub struct LevelSoundSourceConversion;

impl LevelSoundSourceConversion {
  /// `SOUND_SRC_TOOLS_VERSION`, the sound source tool's `[main]` version.
  pub const TOOL_VERSION: u32 = 0;

  /// The sound source part a `level.snd_static` was exported from, each sound an object named after its file, standing
  /// where a new one does, looped, with the distances a new one takes until the editor reads the file's own. Each
  /// window becomes the seconds the part's text saves that export to the same milliseconds.
  ///
  /// # Errors
  ///
  /// Returns an error naming the sound whose window no saved seconds export to.
  pub fn decompile(
    file: &LevelSndStaticFile,
    guid: EditableLevelGuid,
    names: &mut LevelObjectNames,
  ) -> XrfResult<EditableObjectPart<EditableSoundSourceObject>> {
    let objects: Vec<EditableSoundSourceObject> = file
      .sounds
      .iter()
      .enumerate()
      .map(|(index, sound)| {
        let window = |window: &LevelSndStaticWindow, field: &str| {
          Self::get_seconds(window).ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Static sound {index} '{}': no saved seconds export to its {field} window of {}..{} ms",
              sound.sound, window.from, window.to
            ))
          })
        };

        Ok(EditableSoundSourceObject {
          object: EditableCustomObject::new(
            names.get_next_after_file(&sound.sound),
            EditableObjectTransform::default(),
          ),
          sound: sound.sound.clone(),
          flags: EditableSoundSourceObject::LOOPED,
          position: sound.position,
          volume: sound.volume,
          frequency: sound.frequency,
          min_distance: EditableSoundSourceObject::DEFAULT_MIN_DISTANCE,
          max_distance: EditableSoundSourceObject::DEFAULT_MAX_DISTANCE,
          max_ai_distance: EditableSoundSourceObject::DEFAULT_MAX_AI_DISTANCE,
          random_pause: window(&sound.pause, "pause")?,
          active_time: window(&sound.active, "active")?,
          play_time: window(&sound.play, "play")?,
        })
      })
      .collect::<XrfResult<Vec<EditableSoundSourceObject>>>()?;

    Ok(EditableObjectPart::new(guid, Self::TOOL_VERSION, None, objects))
  }

  /// The `level.snd_static` the build writes from the part, `ESoundSource::ExportGame`: each object's sound, position,
  /// volume, frequency and windows in milliseconds, in the part's order. None for a part of no objects, for which the
  /// build writes no file.
  pub fn export(part: &EditableObjectPart<EditableSoundSourceObject>) -> Option<LevelSndStaticFile> {
    (!part.objects.is_empty()).then(|| LevelSndStaticFile {
      sounds: part
        .objects
        .iter()
        .map(|object| LevelSndStaticSound {
          sound: object.sound.clone(),
          position: object.position,
          volume: object.volume,
          frequency: object.frequency,
          active: Self::get_window(&object.active_time),
          play: Self::get_window(&object.play_time),
          pause: Self::get_window(&object.random_pause),
        })
        .collect(),
    })
  }

  /// A window in milliseconds, as `ExportGame` converts the editor's seconds: `u32(seconds * 1000)`, in `f32`.
  fn get_window(seconds: &Vector2d) -> LevelSndStaticWindow {
    LevelSndStaticWindow {
      from: Self::get_milliseconds(seconds.x),
      to: Self::get_milliseconds(seconds.y),
    }
  }

  fn get_milliseconds(seconds: f32) -> u32 {
    (seconds * 1000.0) as u32
  }

  /// The seconds, as the part's `%f` text saves them, that export to a window's milliseconds; None when none near do.
  fn get_seconds(window: &LevelSndStaticWindow) -> Option<Vector2d> {
    Some(Vector2d::new(
      Self::get_saved_seconds(window.from)?,
      Self::get_saved_seconds(window.to)?,
    ))
  }

  /// The quotient rounded to six decimals can land under the millisecond the export truncates to (`0.511` exports as
  /// 510), so its neighbours on the coarser of the `%f` step and the float's own are tried too.
  fn get_saved_seconds(milliseconds: u32) -> Option<f32> {
    let seconds: f32 = (f64::from(milliseconds) / 1000.0) as f32;
    let step: f32 = (seconds.next_up() - seconds).max(1e-6);

    [0.0, 1.0, -1.0, 2.0, -2.0]
      .into_iter()
      .map(|steps: f32| EditableLevelValue::round_as_saved(seconds + steps * step))
      .find(|seconds| Self::get_milliseconds(*seconds) == milliseconds)
  }
}
