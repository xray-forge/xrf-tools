use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};
use xrf_math::{Vector2d, Vector3d};

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A sound the scene plants, `ESoundSource`: an object of `sound_src.part`, which the build exports into
/// `level.snd_static`.
///
/// The sound stands at its own position: `ESoundSource` overrides `SetPosition`, so the object's transform stays where
/// it was created and moves nothing. The distances are the sound file's own, which the editor reads again from the file
/// on load (`ESoundSource::ResetSource`), and the export leaves them out.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableSoundSourceObject {
  pub object: EditableCustomObject,
  /// The sound it plays, a path under `$game_sounds$` without its extension, such as `ambient\air_2`.
  pub sound: String,
  /// `flLooped`, `flPlaying` and `flSimulating`, only [`Self::LOOPED`] set by the editor: GSC's objects hold
  /// uninitialised bits above them, which nothing reads.
  pub flags: u32,
  pub position: Vector3d,
  pub volume: f32,
  pub frequency: f32,
  pub min_distance: f32,
  pub max_distance: f32,
  pub max_ai_distance: f32,
  /// The pause between playings, at random within the range, in seconds; zero for a looped sound.
  pub random_pause: Vector2d,
  /// The window of the game's day it plays in, in seconds from midnight; zero for round the clock.
  pub active_time: Vector2d,
  /// How long one playing lasts, at random within the range, in seconds; zero to play the sound once through.
  pub play_time: Vector2d,
}

impl EditableSoundSourceObject {
  /// `SOUND_SOURCE_VERSION`, the only one `LoadLTX` reads.
  pub const VERSION: u32 = 0x14;

  /// `stStaticSource`, the only kind of sound source.
  pub const STATIC_TYPE: u32 = 0;

  /// `flLooped`, which a new sound source sets.
  pub const LOOPED: u32 = 1;

  /// The distances a new sound source takes before its file gives its own (`ESoundSource::Construct`).
  pub const DEFAULT_MIN_DISTANCE: f32 = 1.0;
  pub const DEFAULT_MAX_DISTANCE: f32 = 300.0;
  pub const DEFAULT_MAX_AI_DISTANCE: f32 = 300.0;
}

impl EditableToolObject for EditableSoundSourceObject {
  /// `OBJCLASS_SOUND_SRC`.
  const CLASS_ID: i32 = 5;
  const TOOL_NAME: &'static str = "sound_src";

  /// As `ESoundSource::LoadLTX` reads it, refusing another version or kind as it does.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = section.get_parsed::<u32>("version")?;
    let kind: u32 = EditableLevelValue::read_u32(section, "snd_type")?;

    if version != Self::VERSION || kind != Self::STATIC_TYPE {
      return Err(XrfError::new_parsing_error(format!(
        "Sound source [{}] is of version {version} and type {kind}, not {} and {}",
        section.name,
        Self::VERSION,
        Self::STATIC_TYPE
      )));
    }

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      sound: section.get_required("snd_name")?.to_owned(),
      flags: EditableLevelValue::read_u32(section, "flags")?,
      position: EditableLevelValue::read_vector3(section, "snd_position")?,
      volume: section.get_parsed::<f32>("volume")?,
      frequency: section.get_parsed::<f32>("freq")?,
      min_distance: section.get_parsed::<f32>("min_dist")?,
      max_distance: section.get_parsed::<f32>("max_dist")?,
      max_ai_distance: section.get_parsed::<f32>("max_ai_dist")?,
      random_pause: EditableLevelValue::read_vector2(section, "random_pause")?,
      active_time: EditableLevelValue::read_vector2(section, "active_time")?,
      play_time: EditableLevelValue::read_vector2(section, "play_time")?,
    })
  }

  /// As `ESoundSource::SaveLTX` writes it, at [`Self::VERSION`].
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("snd_type", Self::STATIC_TYPE.to_string());
    section.set("snd_name", self.sound.clone());
    section.set("flags", EditableLevelValue::format_u32(self.flags));
    section.set("snd_position", EditableLevelValue::format_vector3(&self.position));
    section.set("volume", EditableLevelValue::format_f32(self.volume));
    section.set("freq", EditableLevelValue::format_f32(self.frequency));
    section.set("min_dist", EditableLevelValue::format_f32(self.min_distance));
    section.set("max_dist", EditableLevelValue::format_f32(self.max_distance));
    section.set("max_ai_dist", EditableLevelValue::format_f32(self.max_ai_distance));
    section.set("random_pause", EditableLevelValue::format_vector2(&self.random_pause));
    section.set("active_time", EditableLevelValue::format_vector2(&self.active_time));
    section.set("play_time", EditableLevelValue::format_vector2(&self.play_time));
  }
}
