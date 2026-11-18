use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;

use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_level_part::EditableLevelPart;
use crate::part::editable_part_object::EditablePartObject;
use crate::part::editable_text_part::EditableTextPart;
use crate::scene::editable_group_member::EditableGroupMember;
use crate::scene::editable_scene_tool::EditableSceneTool;

/// The group tool's parts, `group.part` and on (`ESceneGroupTool`): each group an `[object_N]` of class 0 and its
/// members `[object_N_ingroup_M]`, each with the sections named after it, such as a spawn point's
/// `[object_N_ingroup_M_spawndata]`. A member keeps its own transform in the level, not the group's, and the editor
/// loads it into its own tool's list beside the objects the tool's part holds.
pub struct EditableGroupTool;

impl EditableGroupTool {
  /// The tool's name, which names its files.
  pub const TOOL_NAME: &'static str = "group";

  /// `OBJCLASS_GROUP`, the class every group of the tool's parts is saved as.
  pub const CLASS_ID: i32 = 0;

  /// What a member's section name holds between its group's and its index.
  const MEMBER_INFIX: &'static str = "_ingroup_";

  /// The group's field counting its members, which the editor loads `0..count` of (`EScene::ReadObjectsLTX`).
  pub const MEMBER_COUNT_KEY: &'static str = "ingroup_count";

  /// Every group the tool's parts hold, `[object_N]`, as what every object saves first, in the order the files hold
  /// them.
  ///
  /// # Errors
  ///
  /// Returns an error when the tool holds a binary part, a part's objects do not read, or a group does not read.
  pub fn read_groups(tool: &EditableSceneTool) -> XrfResult<Vec<EditableCustomObject>> {
    let mut groups: Vec<EditableCustomObject> = Vec::new();

    for part in Self::get_text_parts(tool)? {
      for group in part.get_objects()? {
        groups.push(EditableCustomObject::read_from_section(group.section)?);
      }
    }

    Ok(groups)
  }

  /// Every member of class `T` of every group the tool's parts hold, each group's in the order the editor loads them,
  /// `0..ingroup_count` by number, and the groups in the order the files hold them; each with the name of the group
  /// holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when the tool holds a binary part, a part's objects do not read, a group has no member count, a
  /// counted member has no section, a member section stands past the count (the editor would leave it out), a group
  /// holding a member of class `T` has no name, a member's class is not a number, or a member of class `T` does not
  /// read.
  pub fn read_grouped_members<T: EditableToolObject>(
    tool: &EditableSceneTool,
  ) -> XrfResult<Vec<EditableGroupMember<T>>> {
    let mut members: Vec<EditableGroupMember<T>> = Vec::new();

    for part in Self::get_text_parts(tool)? {
      for group in part.get_objects()? {
        for member in Self::get_members(&group)? {
          if member.get_class_id()? == T::CLASS_ID {
            members.push(EditableGroupMember {
              group: group.get_name()?.to_owned(),
              object: T::read_from_object(&member)?,
            });
          }
        }
      }
    }

    Ok(members)
  }

  /// `group`'s members, `[<group>_ingroup_<i>]` for `i` in `0..ingroup_count`, as `EScene::ReadObjectsLTX` loads them.
  fn get_members<'a>(group: &EditablePartObject<'a>) -> XrfResult<Vec<EditablePartObject<'a>>> {
    let count: usize = group.section.get_parsed(Self::MEMBER_COUNT_KEY)?;
    let prefix: String = format!("{}{}", group.section.name, Self::MEMBER_INFIX);

    if let Some(stale) = group
      .children
      .iter()
      .find(|section| Self::get_member_index(&prefix, &section.name).is_some_and(|index| index >= count))
    {
      return Err(XrfError::new_parsing_error(format!(
        "Group [{}] counts {count} members but holds [{}] past them, which the editor leaves out",
        group.section.name, stale.name
      )));
    }

    // Every counted member needs a section of its own, so a count past them is refused before it sizes anything.
    if count > group.children.len() {
      return Err(XrfError::new_parsing_error(format!(
        "Group [{}] counts {count} members but holds only {} sections after it",
        group.section.name,
        group.children.len()
      )));
    }

    (0..count)
      .map(|index| {
        let name: String = format!("{prefix}{index}");
        let section = group
          .children
          .iter()
          .find(|section| section.name == name)
          .copied()
          .ok_or_else(|| {
            XrfError::new_parsing_error(format!(
              "Group [{}] counts {count} members but has no [{name}]",
              group.section.name
            ))
          })?;

        Ok(EditablePartObject {
          index,
          section,
          children: Self::get_member_children(&group.children, &section.name),
        })
      })
      .collect()
  }

  /// The tool's parts, every one text as groups are saved.
  fn get_text_parts(tool: &EditableSceneTool) -> XrfResult<Vec<&EditableTextPart>> {
    tool
      .parts
      .iter()
      .map(|part| match part {
        EditableLevelPart::Text(part) => Ok(part),
        EditableLevelPart::Binary(_) => Err(XrfError::new_parsing_error(format!(
          "Scene tool '{}' holds a binary part, where groups are text",
          tool.name
        ))),
      })
      .collect()
  }

  /// The index of a member section, `<prefix><digits>`; none for any other section, such as a member's own
  /// `<prefix><digits>_spawndata`.
  fn get_member_index(prefix: &str, name: &str) -> Option<usize> {
    name
      .strip_prefix(prefix)
      .filter(|digits| !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit()))
      .and_then(|digits| digits.parse().ok())
  }

  /// The sections named after a member, `<member>_<suffix>`.
  fn get_member_children<'a>(sections: &[&'a IniSection], member: &str) -> Vec<&'a IniSection> {
    sections
      .iter()
      .filter(|section| {
        section
          .name
          .strip_prefix(member)
          .is_some_and(|rest| rest.starts_with('_'))
      })
      .copied()
      .collect()
  }
}
