use xrf_editable_level::{
  EditableCustomObject, EditableLevelValue, EditableObjectPart, EditableObjectTransform, EditableSoundSourceObject,
};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelSndStaticFile, LevelSndStaticSound, LevelSndStaticWindow};
use xrf_math::Vector2d;

use crate::level::level_decompile_context::LevelDecompileContext;

/// A level's planted sounds, between the scene's `sound_src.part` and `level.snd_static`, which the build writes with
/// no compiler.
pub struct LevelSoundSourceConversion;

impl LevelSoundSourceConversion {
  /// The sound source part a `level.snd_static` was exported from: each sound a looped object named after its file,
  /// with a new one's distances, since the file keeps none. Each window becomes saved seconds that export to the same
  /// milliseconds.
  ///
  /// # Errors
  ///
  /// Returns an error naming the sound whose window no saved seconds export to.
  pub fn decompile(
    file: &LevelSndStaticFile,
    context: &mut LevelDecompileContext,
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
            context.names.get_next_after_file(&sound.sound),
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

    Ok(EditableObjectPart::new(context.guid, objects))
  }

  /// The `level.snd_static` the build writes from the part, `ESoundSource::ExportGame`, in the part's order; none for
  /// a part of no objects, as the build then writes no file.
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

  /// A window in milliseconds as `ExportGame` converts seconds, `u32(seconds * 1000)` in `f32`.
  fn get_window(seconds: &Vector2d) -> LevelSndStaticWindow {
    LevelSndStaticWindow {
      from: Self::get_milliseconds(seconds.x),
      to: Self::get_milliseconds(seconds.y),
    }
  }

  fn get_milliseconds(seconds: f32) -> u32 {
    (seconds * 1000.0) as u32
  }

  /// Seconds, as the part's `%f` text saves them, that export to a window's milliseconds; none when none near do.
  fn get_seconds(window: &LevelSndStaticWindow) -> Option<Vector2d> {
    Some(Vector2d::new(
      Self::get_saved_seconds(window.from)?,
      Self::get_saved_seconds(window.to)?,
    ))
  }

  /// The quotient at six decimals can fall under the millisecond the export truncates to (`0.511` exports as 510), so
  /// its saved neighbours are tried too.
  fn get_saved_seconds(milliseconds: u32) -> Option<f32> {
    EditableLevelValue::get_saved_neighbours((f64::from(milliseconds) / 1000.0) as f32)
      .into_iter()
      .find(|seconds| Self::get_milliseconds(*seconds) == milliseconds)
  }
}
