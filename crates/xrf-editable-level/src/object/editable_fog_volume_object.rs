use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_shape::EditableShape;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// What a fog volume does in its group, `EFogVolume::m_volumeType`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditableFogVolumeKind {
  /// `fvEmitter`, the group's fog body, simulated as its profile, a config under `$game_config$`, sets.
  Emitter { profile: String },
  /// `fvOcclusion`, a body the group's fog flows around.
  Occlusion,
}

/// A body of volumetric fog, `EFogVolume`: an object of `fog_volume.part`, a shape whose transform alone places the
/// unit box the renderer simulates in. The build exports `level.fog_vol` from its groups, each one emitter and the
/// occlusion volumes it flows around.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableFogVolumeObject {
  pub object: EditableCustomObject,
  pub shape: EditableShape,
  pub kind: EditableFogVolumeKind,
  /// `group_id`, which volumes export together; [`Self::UNGROUPED`] for one never grouped.
  pub group: u32,
}

impl EditableFogVolumeObject {
  /// `FOG_VOL_VERSION`, written over the shape's own `version`.
  pub const VERSION: u32 = 2;

  /// `fvEmitter` and `fvOcclusion`, `folume_type` as the editor spells the key.
  pub const EMITTER: u8 = 0;
  pub const OCCLUSION: u8 = 1;

  /// The group a volume is in before it is grouped, `u32(-1)`.
  pub const UNGROUPED: u32 = u32::MAX;

  /// The profile a new emitter takes.
  pub const DEFAULT_PROFILE: &'static str = "environment\\fog\\default.ltx";

  /// The emitter's profile; None for an occlusion volume.
  pub fn get_profile(&self) -> Option<&str> {
    match &self.kind {
      EditableFogVolumeKind::Emitter { profile } => Some(profile),
      EditableFogVolumeKind::Occlusion => None,
    }
  }
}

impl EditableToolObject for EditableFogVolumeObject {
  /// `OBJCLASS_FOG_VOL`.
  const CLASS_ID: i32 = 15;

  /// As `EFogVolume::LoadLTX` reads it: the type and group from version 1, an emitter's profile from version 2, else
  /// the default ones.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = EditableLevelValue::read_u32(section, "version")?;
    let (kind, group): (u8, u32) = if version > 0 {
      (
        section.get_parsed::<u8>("folume_type")?,
        EditableLevelValue::read_u32(section, "group_id")?,
      )
    } else {
      (Self::EMITTER, Self::UNGROUPED)
    };

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      shape: EditableShape::read_from_section(section)?,
      kind: match kind {
        Self::EMITTER => EditableFogVolumeKind::Emitter {
          profile: if version > 1 {
            section.get_required("profile")?.to_owned()
          } else {
            Self::DEFAULT_PROFILE.to_owned()
          },
        },
        Self::OCCLUSION => EditableFogVolumeKind::Occlusion,
        kind => {
          return Err(XrfError::new_parsing_error(format!(
            "Fog volume [{}] is of type {kind}, neither an emitter nor an occlusion",
            section.name
          )));
        }
      },
      group,
    })
  }

  /// As `EFogVolume::SaveLTX` writes it, at [`Self::VERSION`]; the profile only for an emitter.
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    self.shape.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set(
      "folume_type",
      match self.kind {
        EditableFogVolumeKind::Emitter { .. } => Self::EMITTER,
        EditableFogVolumeKind::Occlusion => Self::OCCLUSION,
      }
      .to_string(),
    );
    section.set("group_id", EditableLevelValue::format_u32(self.group));

    if let Some(profile) = self.get_profile() {
      section.set("profile", profile.to_owned());
    }
  }
}
