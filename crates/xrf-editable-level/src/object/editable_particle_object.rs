use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniSection};

use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A particle effect the scene plants, `EParticlesObject`: an object of `ps.part`, which the build exports into
/// `level.ps_static`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableParticleObject {
  pub object: EditableCustomObject,
  /// The effect it plays, a definition inside `particles.xr` such as `industrial_particles\exhaust_isparenie_blue_down`.
  pub reference: String,
  /// The game modes it plays in, one bit each (`GameTypeChooser`), all set for every mode.
  pub game_types: u16,
}

impl EditableParticleObject {
  /// `CPSOBJECT_VERSION`, which the editor writes and every SDK 0.7 scene holds.
  pub const VERSION: u32 = 0x13;

  /// The first version saving the game modes; an older object plays in all of them.
  pub const GAME_TYPES_VERSION: u32 = 0x12;

  /// Every game mode, `GameTypeChooser::SetDefaults`.
  pub const ALL_GAME_TYPES: u16 = u16::MAX;
}

impl EditableToolObject for EditableParticleObject {
  /// `OBJCLASS_PS`.
  const CLASS_ID: i32 = 11;

  /// As `EParticlesObject::LoadLTX` reads it.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = section.get_parsed::<u32>("version")?;

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      reference: section.get_required("ref_name")?.to_owned(),
      game_types: if version >= Self::GAME_TYPES_VERSION {
        section.get_parsed::<u16>("game_type")?
      } else {
        Self::ALL_GAME_TYPES
      },
    })
  }

  /// As `EParticlesObject::SaveLTX` writes it, at [`Self::VERSION`].
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("ref_name", self.reference.clone());
    section.set("game_type", self.game_types.to_string());
  }
}
