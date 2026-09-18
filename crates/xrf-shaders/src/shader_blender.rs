use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::shader_blender_class::ShaderBlenderClass;
use crate::shader_blender_property::ShaderBlenderProperty;
use crate::shader_fixed_string::{read_shader_fixed_string, write_shader_fixed_string};

/// One compiled blender of `shaders.xr`: what a shader name resolves to.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderBlender {
  pub class: ShaderBlenderClass,
  /// The shader name a mesh, a level surface or a config declares, such as `models\model_aref`.
  pub name: String,
  /// Machine the SDK last saved it on, kept because it is the file's only provenance.
  pub computer: String,
  /// Save time as the SDK stored it, `CBlender_DESC::cTime`.
  pub time: u32,
  /// The class's own format version, which decides the order its `Load` reads properties in.
  pub version: u16,
  pub properties: Vec<ShaderBlenderProperty>,
}

impl ShaderBlender {
  /// `CBlender_DESC::cName`, the fixed field the shader name lives in (`Layers/xrRender/Blender.h`).
  pub const NAME_SIZE: usize = 128;
  /// `CBlender_DESC::cComputer`.
  pub const COMPUTER_SIZE: usize = 32;

  /// Bytes `CBlender_DESC` occupies, which is what `chunk->r(&desc, sizeof(desc))` consumes before the properties
  /// (`Layers/xrRender/ResourceManager_Loader.cpp`).
  pub const DESCRIPTION_SIZE: usize = ShaderBlenderClass::TAG_SIZE
    + Self::NAME_SIZE
    + Self::COMPUTER_SIZE
    + size_of::<u32>()
    + size_of::<u16>()
    + Self::DESCRIPTION_PADDING_SIZE;

  /// `IBlender::oPriority`, written for every class by the base `Save` (`Layers/xrRender/Blender.cpp`).
  pub const PRIORITY_PROPERTY: &'static str = "Priority";
  /// `IBlender::oStrictSorting`, written for every class by the base `Save`.
  pub const STRICT_SORTING_PROPERTY: &'static str = "Strict sorting";

  /// What `#pragma pack(push, 4)` leaves after the two byte version to round the description out.
  const DESCRIPTION_PADDING_SIZE: usize = 2;

  /// The property of that name, or `None` for a class that writes none.
  pub fn find_property(&self, name: &str) -> Option<&ShaderBlenderProperty> {
    self.properties.iter().find(|property| property.name == name)
  }

  /// The value of an integer property of that name, when it is one.
  pub fn integer(&self, name: &str) -> Option<i32> {
    self.find_property(name).and_then(ShaderBlenderProperty::integer)
  }

  /// The value of a boolean property of that name, when it is one.
  pub fn boolean(&self, name: &str) -> Option<bool> {
    self.find_property(name).and_then(ShaderBlenderProperty::boolean)
  }

  /// Whether the author asked for this surface to be drawn in the sorted pass rather than with the rest.
  pub fn is_strict_sorting(&self) -> bool {
    self.boolean(Self::STRICT_SORTING_PROPERTY).unwrap_or(false)
  }
}

impl ChunkReadWrite for ShaderBlender {
  /// Reads one blender chunk whole: its description, then every property up to the chunk's end.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let class: ShaderBlenderClass = ShaderBlenderClass::from_raw(reader.read_u64::<T>()?);
    let name: String = read_shader_fixed_string(reader, Self::NAME_SIZE, "blender name")?;
    let computer: String = read_shader_fixed_string(reader, Self::COMPUTER_SIZE, "blender computer name")?;
    let time: u32 = reader.read_u32::<T>()?;
    let version: u16 = reader.read_u16::<T>()?;

    // The two bytes `pack(4)` leaves after the version, which the engine reads as part of its description struct.
    reader.read_u16::<T>()?;

    // Grown rather than reserved: the format declares no property count, so the only bound is the chunk itself and
    // reserving against it would size every blender's list by its longest possible reading.
    let mut properties: Vec<ShaderBlenderProperty> = Vec::new();

    while reader.has_data() {
      properties.push(reader.read_xr::<T, ShaderBlenderProperty>()?);
    }

    Ok(Self {
      class,
      name,
      computer,
      time,
      version,
      properties,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u64::<T>(self.class.raw())?;
    write_shader_fixed_string(writer, &self.name, Self::NAME_SIZE, "blender name")?;
    write_shader_fixed_string(writer, &self.computer, Self::COMPUTER_SIZE, "blender computer name")?;
    writer.write_u32::<T>(self.time)?;
    writer.write_u16::<T>(self.version)?;
    writer.write_u16::<T>(0)?;

    for property in &self.properties {
      property.write::<T>(writer)?;
    }

    Ok(())
  }
}
