use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_utils::to_format_size;

use crate::shader_library::shader_blender_property_kind::ShaderBlenderPropertyKind;
use crate::shader_library::shader_blender_property_value::ShaderBlenderPropertyValue;
use crate::shader_library::shader_blender_token::ShaderBlenderToken;
use crate::shader_library::shader_fixed_string::{read_shader_fixed_string, write_shader_fixed_string};

/// One property of a blender: what it is called, and what the author left in it.
///
/// Kept as a named record rather than decoded per blender class, because the class's own `Load` order is versioned -
/// `B_MODEL` alone ships as version 1 and 2 in the corpus - while the names are not. Reading by name is what lets one
/// reader answer for every version of every class, including the classes a mod added.
#[derive(Clone, Debug, PartialEq)]
pub struct ShaderBlenderProperty {
  pub name: String,
  pub value: ShaderBlenderPropertyValue,
}

impl ShaderBlenderProperty {
  /// The smallest a property can be: its type, an empty name, and no payload.
  pub const MIN_SERIALIZED_SIZE: u64 = size_of::<u32>() as u64 + 1;

  /// The value of an integer property, for a caller reading one knob out of a blender.
  ///
  /// `None` for a property of another type, so a knob read as the wrong type is absent rather than coerced.
  pub fn integer(&self) -> Option<i32> {
    match self.value {
      ShaderBlenderPropertyValue::Integer { value, .. } => Some(value),
      _ => None,
    }
  }

  /// The value of a boolean property; see [`Self::integer`].
  pub fn boolean(&self) -> Option<bool> {
    match self.value {
      ShaderBlenderPropertyValue::Bool(value) => Some(value),
      _ => None,
    }
  }

  /// Reads the payload of one type, which is the only part of a property whose width the type does not fix outright.
  fn read_value<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    kind: ShaderBlenderPropertyKind,
  ) -> XrfResult<ShaderBlenderPropertyValue> {
    Ok(match kind {
      ShaderBlenderPropertyKind::Marker => ShaderBlenderPropertyValue::Marker,
      ShaderBlenderPropertyKind::Matrix => ShaderBlenderPropertyValue::Matrix(Self::read_text(reader)?),
      ShaderBlenderPropertyKind::Constant => ShaderBlenderPropertyValue::Constant(Self::read_text(reader)?),
      ShaderBlenderPropertyKind::Texture => ShaderBlenderPropertyValue::Texture(Self::read_text(reader)?),
      ShaderBlenderPropertyKind::Object => ShaderBlenderPropertyValue::Object(Self::read_text(reader)?),
      ShaderBlenderPropertyKind::Text => ShaderBlenderPropertyValue::Text(Self::read_text(reader)?),
      ShaderBlenderPropertyKind::Integer => ShaderBlenderPropertyValue::Integer {
        value: reader.read_i32::<T>()?,
        minimum: reader.read_i32::<T>()?,
        maximum: reader.read_i32::<T>()?,
      },
      ShaderBlenderPropertyKind::Float => ShaderBlenderPropertyValue::Float {
        value: reader.read_f32::<T>()?,
        minimum: reader.read_f32::<T>()?,
        maximum: reader.read_f32::<T>()?,
      },
      ShaderBlenderPropertyKind::Bool => ShaderBlenderPropertyValue::Bool(reader.read_u32::<T>()? != 0),
      ShaderBlenderPropertyKind::Token => Self::read_token::<T, D>(reader)?,
      ShaderBlenderPropertyKind::ClassId => Self::read_class_id::<T, D>(reader)?,
    })
  }

  /// Reads a token list: the selected id, the entry count, and that many `(id, label)` entries.
  fn read_token<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<ShaderBlenderPropertyValue> {
    let selected: u32 = reader.read_u32::<T>()?;
    let count: u32 = reader.read_u32::<T>()?;
    let mut items: Vec<ShaderBlenderToken> = reader.new_bounded_vec(
      u64::from(count),
      ShaderBlenderPropertyKind::TOKEN_ITEM_SIZE as u64,
      "shader blender token items",
    )?;

    for _ in 0..count {
      items.push(ShaderBlenderToken {
        id: reader.read_u32::<T>()?,
        label: Self::read_text(reader)?,
      });
    }

    Ok(ShaderBlenderPropertyValue::Token { selected, items })
  }

  /// Reads a class list, whose count is the second half of the `xrP_CLSID` payload the selected class occupies the
  /// first of (`xrEngine/Properties.h`).
  ///
  /// No blender of the corpus carries one, so this layout follows the engine's struct rather than a witness.
  fn read_class_id<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<ShaderBlenderPropertyValue> {
    let selected: u64 = reader.read_u64::<T>()?;
    let count: u32 = reader.read_u32::<T>()?;
    let mut items: Vec<u64> =
      reader.new_bounded_vec(u64::from(count), size_of::<u64>() as u64, "shader blender classes")?;

    for _ in 0..count {
      items.push(reader.read_u64::<T>()?);
    }

    Ok(ShaderBlenderPropertyValue::ClassId { selected, items })
  }

  fn read_text<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<String> {
    read_shader_fixed_string(reader, ShaderBlenderPropertyKind::TEXT_SIZE, "blender property value")
  }

  fn write_text(writer: &mut ChunkWriter, value: &str) -> XrfResult {
    write_shader_fixed_string(
      writer,
      value,
      ShaderBlenderPropertyKind::TEXT_SIZE,
      "blender property value",
    )
  }
}

impl ChunkReadWrite for ShaderBlenderProperty {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let kind: ShaderBlenderPropertyKind = ShaderBlenderPropertyKind::of(reader.read_u32::<T>()?)?;

    Ok(Self {
      name: reader.read_w1251_string()?,
      value: Self::read_value::<T, D>(reader, kind)?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.value.kind().raw())?;
    writer.write_w1251_string(&self.name)?;

    match &self.value {
      ShaderBlenderPropertyValue::Marker => {}
      ShaderBlenderPropertyValue::Matrix(value)
      | ShaderBlenderPropertyValue::Constant(value)
      | ShaderBlenderPropertyValue::Texture(value)
      | ShaderBlenderPropertyValue::Object(value)
      | ShaderBlenderPropertyValue::Text(value) => Self::write_text(writer, value)?,
      ShaderBlenderPropertyValue::Integer {
        value,
        minimum,
        maximum,
      } => {
        writer.write_i32::<T>(*value)?;
        writer.write_i32::<T>(*minimum)?;
        writer.write_i32::<T>(*maximum)?;
      }
      ShaderBlenderPropertyValue::Float {
        value,
        minimum,
        maximum,
      } => {
        writer.write_f32::<T>(*value)?;
        writer.write_f32::<T>(*minimum)?;
        writer.write_f32::<T>(*maximum)?;
      }
      ShaderBlenderPropertyValue::Bool(value) => writer.write_u32::<T>(u32::from(*value))?,
      ShaderBlenderPropertyValue::Token { selected, items } => {
        writer.write_u32::<T>(*selected)?;
        writer.write_u32::<T>(to_format_size(items.len(), "shader blender token items")?)?;

        for item in items {
          writer.write_u32::<T>(item.id)?;
          Self::write_text(writer, &item.label)?;
        }
      }
      ShaderBlenderPropertyValue::ClassId { selected, items } => {
        writer.write_u64::<T>(*selected)?;
        writer.write_u32::<T>(to_format_size(items.len(), "shader blender classes")?)?;

        for item in items {
          writer.write_u64::<T>(*item)?;
        }
      }
    }

    Ok(())
  }
}
