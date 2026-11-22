use xrf_error::{XrfError, XrfResult};

use crate::level::editable_level_guid::EditableLevelGuid;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_object_part::EditableObjectPart;
use crate::scene::editable_group_member::EditableGroupMember;
use crate::scene::editable_group_tool::EditableGroupTool;
use crate::scene::editable_level_scene::EditableLevelScene;

/// One object tool's objects in a scene as the editor loads them: group members of the tool's class first, since the
/// group tool (class 0) loads first, then the tool's own part.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableToolObjects<O> {
  /// The groups' members of the tool's class, in the order the group tool's files hold them.
  pub members: Vec<EditableGroupMember<O>>,
  /// The tool's own part, or an empty one where the scene has no such tool, as the editor holds every tool.
  pub part: EditableObjectPart<O>,
}

impl<O: EditableToolObject> EditableToolObjects<O> {
  /// The objects of tool `O` the scene holds; a tool the scene lacks reads as a new empty part under `guid`.
  ///
  /// # Errors
  ///
  /// Returns an error as [`EditableObjectPart::read_from_scene_tool`] or [`EditableGroupTool::read_grouped_members`]
  /// does.
  pub fn read(scene: &EditableLevelScene, guid: EditableLevelGuid) -> XrfResult<Self> {
    Ok(Self {
      members: Self::read_members(scene)?,
      part: Self::read_part(scene, guid)?,
    })
  }

  /// The objects of tool `O` the scene holds, the groups' members kept where the tool's own part does not read: the
  /// part then stands empty under `guid` and its error is returned beside it. The editor loads the group tool before
  /// the others, so it holds those members when it fails the tool's own part; the members' indexes come first, so
  /// keeping them moves no other object's.
  ///
  /// # Errors
  ///
  /// Returns an error as [`EditableGroupTool::read_grouped_members`] does.
  pub fn read_keeping_members(
    scene: &EditableLevelScene,
    guid: EditableLevelGuid,
  ) -> XrfResult<(Self, Option<XrfError>)> {
    let members: Vec<EditableGroupMember<O>> = Self::read_members(scene)?;

    Ok(match Self::read_part(scene, guid) {
      Ok(part) => (Self { members, part }, None),
      Err(error) => (
        Self {
          members,
          part: EditableObjectPart::new(guid, Vec::new()),
        },
        Some(error),
      ),
    })
  }

  /// The groups' members of the tool's class, none where the scene has no group tool.
  fn read_members(scene: &EditableLevelScene) -> XrfResult<Vec<EditableGroupMember<O>>> {
    match scene.get_tool(EditableGroupTool::TOOL_NAME) {
      Some(groups) => EditableGroupTool::read_grouped_members::<O>(groups),
      None => Ok(Vec::new()),
    }
  }

  /// The tool's own part, an empty one under `guid` where the scene has no such tool.
  fn read_part(scene: &EditableLevelScene, guid: EditableLevelGuid) -> XrfResult<EditableObjectPart<O>> {
    match scene.get_tool(O::TOOL_NAME) {
      Some(tool) => EditableObjectPart::read_from_scene_tool(tool),
      None => Ok(EditableObjectPart::new(guid, Vec::new())),
    }
  }

  /// Every object in the editor's loaded order, members first, each with its index in that order, which a scene's
  /// object is named by.
  pub fn iter_indexed(&self) -> impl Iterator<Item = (u32, &O)> {
    (0u32..).zip(
      self
        .members
        .iter()
        .map(|member| &member.object)
        .chain(&self.part.objects),
    )
  }

  /// The tool's part as the editor holds it once loaded, the members' objects before its own.
  pub fn into_part(self) -> EditableObjectPart<O> {
    let Self { members, mut part } = self;
    let mut objects: Vec<O> = members.into_iter().map(|member| member.object).collect();

    objects.append(&mut part.objects);
    part.objects = objects;

    part
  }
}
