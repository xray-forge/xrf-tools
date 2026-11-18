use xrf_error::XrfResult;

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_object_part::EditableObjectPart;
use crate::scene::editable_group_member::EditableGroupMember;
use crate::scene::editable_group_tool::EditableGroupTool;
use crate::scene::editable_level_scene::EditableLevelScene;

/// One object tool's objects in a scene as the editor loads them: the group tool's members of the tool's class first,
/// since it loads the group tool (class 0) before the others, then the objects of the tool's own part.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableToolObjects<O> {
  /// The groups' members of the tool's class, in the order the group tool's files hold them.
  pub members: Vec<EditableGroupMember<O>>,
  /// The tool's own part, or an empty one where the scene has no such tool, as the editor holds every tool.
  pub part: EditableObjectPart<O>,
}

impl<O: EditableToolObject> EditableToolObjects<O> {
  /// The objects of tool `O` the scene holds, a new tool's empty part standing for one it lacks, made with `guid`.
  ///
  /// # Errors
  ///
  /// Returns an error when the tool's part does not read as [`EditableObjectPart::read_from_scene_tool`] reads it, or
  /// the group tool's members of its class do not read as [`EditableGroupTool::read_grouped_members`] reads them.
  pub fn read(scene: &EditableLevelScene, guid: EditableLevelGuid) -> XrfResult<Self> {
    Ok(Self {
      members: match scene.get_tool(EditableGroupTool::TOOL_NAME) {
        Some(groups) => EditableGroupTool::read_grouped_members::<O>(groups)?,
        None => Vec::new(),
      },
      part: match scene.get_tool(O::TOOL_NAME) {
        Some(tool) => EditableObjectPart::read_from_scene_tool(tool)?,
        None => EditableObjectPart::new(guid, Vec::new()),
      },
    })
  }

  /// Every object in the order the editor holds them once loaded, the members then the part's own, each with its
  /// place in that order: the index a scene's object is named by.
  pub fn iter_indexed(&self) -> impl Iterator<Item = (u32, &O)> {
    (0u32..).zip(
      self
        .members
        .iter()
        .map(|member| &member.object)
        .chain(&self.part.objects),
    )
  }

  /// The tool's part as the editor holds it once loaded: its objects the members, then its own.
  pub fn into_part(self) -> EditableObjectPart<O> {
    let Self { members, mut part } = self;
    let mut objects: Vec<O> = members.into_iter().map(|member| member.object).collect();

    objects.append(&mut part.objects);
    part.objects = objects;

    part
  }
}
