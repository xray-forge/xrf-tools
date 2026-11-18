use crate::level::editable_level_guid::EditableLevelGuid;

/// One part file a scene read reached: kept as its tool's part at `index`, or the file the tool's walk stopped at,
/// with the reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableScenePartFile {
  /// The tool the file belongs to, and its place among the tool's files, `<tool>.part` being 0.
  pub tool: String,
  pub index: usize,
  pub name: String,
  /// Its size, 0 where it was not read.
  pub size: u64,
  /// The scene GUID it carries, where it read as a part.
  pub guid: Option<EditableLevelGuid>,
  /// Why the walk stopped at it, where it did; the tool keeps the parts before it.
  pub problem: Option<String>,
}
