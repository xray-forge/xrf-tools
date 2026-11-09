use xrf_error::{XrfError, XrfResult};
use xrf_ini::{IniFile, IniSection};

use crate::level::editable_level_value::EditableLevelValue;
use crate::object::editable_custom_object::EditableCustomObject;
use crate::object::editable_tool_object::EditableToolObject;
use crate::part::editable_part_object::EditablePartObject;

/// A light flare the scene places, `CGlow`: an object of `glow.part`, which the build hands the level compiler as a
/// `b_glow` (`SceneBuilder::BuildGlow`, `BuilderRemote.cpp:1161`) and the compiled level keeps in `fsL_GLOWS`.
#[derive(Clone, Debug, PartialEq)]
pub struct EditableGlowObject {
  pub object: EditableCustomObject,
  /// The radius it is drawn and culled at.
  pub radius: f32,
  /// The engine shader it draws with, `effects\glow` in every shipped level.
  pub shader: String,
  /// The flare's texture, such as `glow\glow_white`.
  pub texture: String,
  /// [`Self::FIXED_SIZE`] or none.
  pub flags: u16,
}

impl EditableGlowObject {
  /// `GLOW_VERSION`, the only one `CGlow::LoadLTX` reads.
  pub const VERSION: u32 = 0x12;

  /// `gfFixedSize`: the flare keeps its size whatever the distance, exported as the `b_glow`'s non-scalable flag.
  pub const FIXED_SIZE: u16 = 1;
}

impl EditableToolObject for EditableGlowObject {
  /// `OBJCLASS_GLOW`.
  const CLASS_ID: i32 = 1;
  const TOOL_NAME: &'static str = "glow";
  const TOOL_VERSION_KEY: &'static str = "glow_tool_version";
  const KEEPS_FLAGS: bool = true;

  /// As `CGlow::LoadLTX` reads it, refusing another version as it does.
  fn read_from_object(object: &EditablePartObject<'_>) -> XrfResult<Self> {
    let section: &IniSection = object.section;
    let version: u32 = section.get_parsed::<u32>("version")?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Glow [{}] is of version {version}, not {}",
        section.name,
        Self::VERSION
      )));
    }

    Ok(Self {
      object: EditableCustomObject::read_from_section(section)?,
      radius: section.get_parsed::<f32>("radius")?,
      shader: section.get_required("shader_name")?.to_owned(),
      texture: section.get_required("texture_name")?.to_owned(),
      // `r_u32` assigned to the 16-bit flags, which keeps the low half.
      flags: EditableLevelValue::read_u32(section, "flags")? as u16,
    })
  }

  /// As `CGlow::SaveLTX` writes it, at [`Self::VERSION`].
  fn write_to_ini(&self, ini: &mut IniFile, section: &str) {
    let section: &mut IniSection = ini.section_mut(section);

    self.object.write_to_section(section);
    section.set("version", Self::VERSION.to_string());
    section.set("radius", EditableLevelValue::format_f32(self.radius));
    section.set("shader_name", self.shader.clone());
    section.set("texture_name", self.texture.clone());
    section.set("flags", self.flags.to_string());
  }
}
