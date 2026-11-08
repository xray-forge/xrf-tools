use crate::part::editable_level_part::EditableLevelPart;

/// One tool's part files in a scene: `<name>.part`, then `<name>.part1`, `<name>.part2` and on for a tool that saves
/// more than one (`EScene::SaveToolLTX`, which the group tool does).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableSceneTool {
  /// The tool's class name, which names its files: `scene_object`, `spawn`, `ai_map` and the rest.
  pub name: String,
  pub parts: Vec<EditableLevelPart>,
}

impl EditableSceneTool {
  /// The file name of the tool's part at `index`: the first is `<name>.part`, the next ones carry their number.
  pub fn get_file_name(&self, index: usize) -> String {
    if index == 0 {
      format!("{}.{}", self.name, EditableLevelPart::EXTENSION)
    } else {
      format!("{}.{}{index}", self.name, EditableLevelPart::EXTENSION)
    }
  }
}
