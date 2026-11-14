use serde::{Deserialize, Serialize};
use xrf_editable_level::{
  EditableAiMapTool, EditableDetailObjectTool, EditableFogVolumeObject, EditableGlowObject, EditableGroupTool,
  EditableLevelPart, EditableLightObject, EditablePartObject, EditableParticleObject, EditablePortalObject,
  EditableSceneObject, EditableSectorObject, EditableShapeObject, EditableSoundEnvironmentObject,
  EditableSoundSourceObject, EditableSpawnPointObject, EditableTextPart, EditableToolObject, EditableWallmarkTool,
  EditableWayObject,
};

use crate::scene::{SceneProblem, SceneProblemKind, SceneToolFormat};

/// A Level Editor tool a scene's part file belongs to, by the name the file carries, and what the editor requires of
/// the tool's part before it loads it.
///
/// Declared in the order of the editor's class ids (`ObjClassID`), which is the order it keeps and lists its tools in.
/// Each crosses IPC as the name its part files carry, written per variant rather than derived: no case convention
/// spells `ps` or `sound_src`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SceneToolKind {
  /// Groups of other tools' objects, `OBJCLASS_GROUP`.
  #[serde(rename = "group")]
  Group,
  /// Light flares, `OBJCLASS_GLOW`.
  #[serde(rename = "glow")]
  Glow,
  /// Library objects placed, `OBJCLASS_SCENEOBJECT`.
  #[serde(rename = "scene_object")]
  SceneObject,
  /// Lights, `OBJCLASS_LIGHT`.
  #[serde(rename = "light")]
  Light,
  /// Shapes, `OBJCLASS_SHAPE`.
  #[serde(rename = "shape")]
  Shape,
  /// Sounds planted, `OBJCLASS_SOUND_SRC`.
  #[serde(rename = "sound_src")]
  SoundSource,
  /// Spawn points, `OBJCLASS_SPAWNPOINT`.
  #[serde(rename = "spawn")]
  Spawn,
  /// Patrol paths, `OBJCLASS_WAY`.
  #[serde(rename = "way")]
  Way,
  /// Sectors, `OBJCLASS_SECTOR`.
  #[serde(rename = "sector")]
  Sector,
  /// Portals, `OBJCLASS_PORTAL`.
  #[serde(rename = "portal")]
  Portal,
  /// Reverb volumes, `OBJCLASS_SOUND_ENV`.
  #[serde(rename = "sound_env")]
  SoundEnvironment,
  /// Particle effects planted, `OBJCLASS_PS`.
  #[serde(rename = "ps")]
  Particle,
  /// Detail objects, `OBJCLASS_DO`.
  #[serde(rename = "detail_object")]
  DetailObject,
  /// The AI map, `OBJCLASS_AIMAP`.
  #[serde(rename = "ai_map")]
  AiMap,
  /// Wall marks, `OBJCLASS_WM`.
  #[serde(rename = "wallmark")]
  Wallmark,
  /// Fog volumes, `OBJCLASS_FOG_VOL`.
  #[serde(rename = "fog_volume")]
  FogVolume,
}

impl SceneToolKind {
  /// Every tool of the CoP SDK's editor that saves a part, in the order the editor lists them.
  pub(crate) const ALL: [Self; 16] = [
    Self::Group,
    Self::Glow,
    Self::SceneObject,
    Self::Light,
    Self::Shape,
    Self::SoundSource,
    Self::Spawn,
    Self::Way,
    Self::Sector,
    Self::Portal,
    Self::SoundEnvironment,
    Self::Particle,
    Self::DetailObject,
    Self::AiMap,
    Self::Wallmark,
    Self::FogVolume,
  ];

  /// The tool whose part files carry `name`, none for one the editor does not save.
  pub(crate) fn of(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|kind| kind.get_name() == name)
  }

  /// The tool's `ClassName`, which names its part files.
  pub(crate) fn get_name(self) -> &'static str {
    match self {
      Self::SceneObject => EditableSceneObject::TOOL_NAME,
      Self::Sector => EditableSectorObject::TOOL_NAME,
      Self::Portal => EditablePortalObject::TOOL_NAME,
      Self::Light => EditableLightObject::TOOL_NAME,
      Self::Glow => EditableGlowObject::TOOL_NAME,
      Self::Spawn => EditableSpawnPointObject::TOOL_NAME,
      Self::Shape => EditableShapeObject::TOOL_NAME,
      Self::Way => EditableWayObject::TOOL_NAME,
      Self::Particle => EditableParticleObject::TOOL_NAME,
      Self::SoundSource => EditableSoundSourceObject::TOOL_NAME,
      Self::SoundEnvironment => EditableSoundEnvironmentObject::TOOL_NAME,
      Self::FogVolume => EditableFogVolumeObject::TOOL_NAME,
      Self::Group => EditableGroupTool::TOOL_NAME,
      Self::AiMap => EditableAiMapTool::TOOL_NAME,
      Self::DetailObject => EditableDetailObjectTool::TOOL_NAME,
      Self::Wallmark => EditableWallmarkTool::TOOL_NAME,
    }
  }

  /// How the tool saves its part.
  pub(crate) fn get_format(self) -> SceneToolFormat {
    match self.get_binary_class() {
      Some(_) => SceneToolFormat::Binary,
      None => SceneToolFormat::Object,
    }
  }

  /// Why the editor would not load `part`, the tool's file named `file`, as this tool's part, where it would not: a
  /// part saved as another tool's class, an object tool's part of another tool version, or one holding an object of
  /// another class. The objects themselves are read where they are described.
  pub(crate) fn check(self, file: &str, part: &EditableLevelPart) -> Option<SceneProblem> {
    match (part, self.get_binary_class()) {
      (EditableLevelPart::Binary(part), Some(class_id)) if part.class_id == class_id => None,
      (EditableLevelPart::Binary(part), _) => Some(self.report(
        SceneProblemKind::ForeignClass,
        file,
        format!(
          "Part is saved as chunks of class {}, which tool '{}' does not save",
          part.class_id,
          self.get_name()
        ),
      )),
      (EditableLevelPart::Text(_), Some(class_id)) => Some(self.report(
        SceneProblemKind::ForeignClass,
        file,
        format!(
          "Part is saved as text, where tool '{}' saves chunks of class {class_id}",
          self.get_name()
        ),
      )),
      (EditableLevelPart::Text(part), None) => self.check_text(file, part),
    }
  }

  /// The class an object tool's text part is checked against, by the type its objects read as.
  fn check_text(self, file: &str, part: &EditableTextPart) -> Option<SceneProblem> {
    match self {
      Self::SceneObject => self.check_objects::<EditableSceneObject>(file, part),
      Self::Sector => self.check_objects::<EditableSectorObject>(file, part),
      Self::Portal => self.check_objects::<EditablePortalObject>(file, part),
      Self::Light => self.check_objects::<EditableLightObject>(file, part),
      Self::Glow => self.check_objects::<EditableGlowObject>(file, part),
      Self::Spawn => self.check_objects::<EditableSpawnPointObject>(file, part),
      Self::Shape => self.check_objects::<EditableShapeObject>(file, part),
      Self::Way => self.check_objects::<EditableWayObject>(file, part),
      Self::Particle => self.check_objects::<EditableParticleObject>(file, part),
      Self::SoundSource => self.check_objects::<EditableSoundSourceObject>(file, part),
      Self::SoundEnvironment => self.check_objects::<EditableSoundEnvironmentObject>(file, part),
      Self::FogVolume => self.check_objects::<EditableFogVolumeObject>(file, part),
      Self::Group => self.check_classes(file, part, EditableGroupTool::CLASS_ID),
      // A binary tool's text part is refused before its text is looked at.
      Self::AiMap | Self::DetailObject | Self::Wallmark => None,
    }
  }

  /// An object tool's part as its `LoadLTX` checks it before reading an object: `[main]`'s version, then each object's
  /// class.
  fn check_objects<T: EditableToolObject>(self, file: &str, part: &EditableTextPart) -> Option<SceneProblem> {
    match part.get_header(T::TOOL_VERSION_KEY) {
      Err(error) => Some(self.report(
        SceneProblemKind::Unreadable,
        file,
        format!("Part's [main] does not read: {error}"),
      )),
      Ok(Some(header)) if header.version != T::TOOL_VERSION => Some(self.report(
        SceneProblemKind::WrongVersion,
        file,
        format!(
          "Part holds tool version {}, where tool '{}' reads version {}",
          header.version,
          T::TOOL_NAME,
          T::TOOL_VERSION
        ),
      )),
      Ok(_) => self.check_classes(file, part, T::CLASS_ID),
    }
  }

  /// The first object of the part that is not of `class_id`, or whose objects do not gather.
  fn check_classes(self, file: &str, part: &EditableTextPart, class_id: i32) -> Option<SceneProblem> {
    let objects: Vec<EditablePartObject<'_>> = match part.get_objects() {
      Ok(objects) => objects,
      Err(error) => {
        return Some(self.report(
          SceneProblemKind::Unreadable,
          file,
          format!("Part's objects do not read: {error}"),
        ));
      }
    };

    objects.iter().find_map(|object| match object.get_class_id() {
      Ok(class) if class == class_id => None,
      Ok(class) => Some(self.report(
        SceneProblemKind::ForeignClass,
        file,
        format!(
          "Object [{}] is of class {class}, not tool '{}''s {class_id}",
          object.section.name,
          self.get_name()
        ),
      )),
      Err(error) => Some(self.report(
        SceneProblemKind::Unreadable,
        file,
        format!("Object [{}] names no class: {error}", object.section.name),
      )),
    })
  }

  /// The class a binary tool's data chunk is saved as; none for an object tool, whose parts are text.
  fn get_binary_class(self) -> Option<u32> {
    match self {
      Self::AiMap => Some(EditableAiMapTool::CLASS_ID),
      Self::DetailObject => Some(EditableDetailObjectTool::CLASS_ID),
      Self::Wallmark => Some(EditableWallmarkTool::CLASS_ID),
      _ => None,
    }
  }

  /// A problem of this tool's file `file`.
  fn report(self, kind: SceneProblemKind, file: &str, message: String) -> SceneProblem {
    SceneProblem {
      kind,
      tool: self.get_name().to_owned(),
      file: Some(file.to_owned()),
      message,
    }
  }
}
