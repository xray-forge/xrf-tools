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

  /// Every member of class `T` of every group the tool's parts hold, in the order the files hold them, each with the
  /// name of the group holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when the tool holds a binary part, a part's objects do not read, a group holding a member of class
  /// `T` has no name, a member's class is not a number, or a member of class `T` does not read.
  pub fn read_grouped_members<T: EditableToolObject>(
    tool: &EditableSceneTool,
  ) -> XrfResult<Vec<EditableGroupMember<T>>> {
    let mut members: Vec<EditableGroupMember<T>> = Vec::new();

    for part in Self::get_text_parts(tool)? {
      for group in part.get_objects()? {
        let prefix: String = format!("{}{}", group.section.name, Self::MEMBER_INFIX);

        for section in &group.children {
          let Some(index) = section
            .name
            .strip_prefix(&prefix)
            .filter(|digits| digits.bytes().all(|byte| byte.is_ascii_digit()))
            .and_then(|digits| digits.parse::<usize>().ok())
          else {
            continue;
          };

          let member: EditablePartObject<'_> = EditablePartObject {
            index,
            section,
            children: Self::get_member_children(&group.children, &section.name),
          };

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
