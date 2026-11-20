use crate::scene::editable_level_scene::EditableLevelScene;
use crate::scene::editable_scene_part_file::EditableScenePartFile;

/// A scene as [`EditableLevelScene::read_from_source`] read it: every part the editor would load, and each part file
/// the read reached, the one a tool's walk stopped at among them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableSceneRead {
  pub scene: EditableLevelScene,
  /// Every file reached, by tool in the scene's order, then by place.
  pub files: Vec<EditableScenePartFile>,
}

impl EditableSceneRead {
  /// The files a tool's walk stopped at.
  pub fn get_problems(&self) -> impl Iterator<Item = &EditableScenePartFile> {
    self.files.iter().filter(|file| file.problem.is_some())
  }
}
