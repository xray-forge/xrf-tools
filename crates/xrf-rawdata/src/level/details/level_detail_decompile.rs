use xrf_editable_level::EditableDetailObjectTool;

use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;

/// A level's details decompiled: the tool, where any model was found in the library, the found objects' textures cut
/// from the level's atlas, and the models and textures left out.
pub(crate) struct LevelDetailDecompile {
  pub(crate) tool: Option<EditableDetailObjectTool>,
  pub(crate) textures: Vec<LevelSceneTextureFile>,
  pub(crate) refused: Vec<LevelSceneRefusal>,
}
