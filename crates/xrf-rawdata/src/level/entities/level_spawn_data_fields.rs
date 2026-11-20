use byteorder::ByteOrder;
use xrf_chunk::{ChunkField, ChunkFieldKind, ChunkFieldReplay, ChunkFieldSource};
use xrf_editable_level::EditableLevelValue;
use xrf_error::{XrfError, XrfResult};
use xrf_ini::IniSection;
use xrf_math::Vector3d;
use xrf_spawn::AlifeObject;

/// An entity's spawn packet as the Level Editor saves it in `[object_N_spawndata]`: one field per value, numbered from
/// `000001` (`SIniFileStream::gen_name`) and printed as `CInifile` prints its kind. It is the source a
/// [`xrf_chunk::ChunkFieldReplay`] reads the packet from.
pub struct LevelSpawnDataFields<'a> {
  section: &'a IniSection,
  /// The number of the field read last.
  read: usize,
}

impl<'a> LevelSpawnDataFields<'a> {
  pub const fn new(section: &'a IniSection) -> Self {
    Self { section, read: 0 }
  }

  /// The key the `number`th field is saved under, `%06d`.
  pub fn format_key(number: usize) -> String {
    format!("{number:06}")
  }

  /// A field's value as the editor prints it: integers `%d`, so a `u32` past `i32::MAX` negative, floats `%f`,
  /// vectors `x, y, z`, strings quoted. A `u64` prints in decimal, as `CInifile::w_u64` does outside the editor build;
  /// no GSC scene holds one to show the editor build's print.
  pub fn format_value(field: &ChunkField) -> String {
    match field {
      ChunkField::U8(value) => value.to_string(),
      ChunkField::U16(value) => value.to_string(),
      ChunkField::U32(value) => EditableLevelValue::format_u32(*value),
      ChunkField::U64(value) => value.to_string(),
      ChunkField::F32(value) => EditableLevelValue::format_f32(*value),
      ChunkField::Vector3(value) => EditableLevelValue::format_vector3(&Vector3d::from(*value)),
      ChunkField::String(value) => EditableLevelValue::format_quoted(value),
    }
  }

  /// Writes a packet's fields into `section`, numbered from one.
  ///
  /// # Errors
  ///
  /// Returns an error for a string holding a double quote: `SIniFileStream::w_stringZ` quotes it without escaping, so
  /// it would not read back.
  pub fn write_section(fields: &[ChunkField], section: &mut IniSection) -> XrfResult {
    for (index, field) in fields.iter().enumerate() {
      if let ChunkField::String(value) = field
        && value.contains('"')
      {
        return Err(XrfError::new_invalid_error(format!(
          "Spawn data string '{value}' holds a double quote, which the editor cannot save"
        )));
      }

      section.set(Self::format_key(index + 1), Self::format_value(field));
    }

    Ok(())
  }

  /// The entity whose packet a spawn point's `[object_N_spawndata]` holds, read whole as the build reads it.
  ///
  /// # Errors
  ///
  /// Returns an error where the packet is not of a known class, its state does not read whole, or it holds a field
  /// past the packet's last.
  pub fn read_entity<T: ByteOrder>(data: &IniSection) -> XrfResult<AlifeObject> {
    let mut fields: LevelSpawnDataFields<'_> = LevelSpawnDataFields::new(data);
    let entity: AlifeObject = AlifeObject::read_spawn_packet::<T, _>(&mut ChunkFieldReplay::new(&mut fields))?;

    fields.assert_read()?;

    Ok(entity)
  }

  /// The visual the Level Editor draws the spawn point as: its packet's `cse_visual::visual_name`, whatever its
  /// section's config says; none for a class drawn as nothing or a packet naming no visual.
  ///
  /// # Errors
  ///
  /// Returns an error where the packet does not read ([`Self::read_entity`]).
  pub fn read_visual<T: ByteOrder>(data: &IniSection) -> XrfResult<Option<String>> {
    Ok(Self::read_entity::<T>(data)?.inherited.get_visual().map(str::to_owned))
  }

  /// Refuses a numbered field past the last one read: the editor would ignore it, so it belongs to another layout
  /// than the class read.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section and the first field left.
  pub fn assert_read(&self) -> XrfResult {
    let key: String = Self::format_key(self.read + 1);

    if self.section.get(&key).is_some() {
      return Err(XrfError::new_parsing_error(format!(
        "Spawn data [{}] holds field {key} past the packet's last",
        self.section.name
      )));
    }

    Ok(())
  }
}

impl ChunkFieldSource for LevelSpawnDataFields<'_> {
  /// The next numbered field, parsed as the editor's `SIniFileStream::r_*` of `kind` reads it.
  fn read_field(&mut self, kind: ChunkFieldKind) -> XrfResult<ChunkField> {
    let section: &IniSection = self.section;
    let key: String = Self::format_key(self.read + 1);

    self.read += 1;

    Ok(match kind {
      ChunkFieldKind::U8 => ChunkField::U8(section.get_parsed::<u8>(&key)?),
      ChunkFieldKind::U16 => ChunkField::U16(section.get_parsed::<u16>(&key)?),
      ChunkFieldKind::U32 => ChunkField::U32(EditableLevelValue::read_u32(section, &key)?),
      ChunkFieldKind::U64 => ChunkField::U64(section.get_parsed::<u64>(&key)?),
      ChunkFieldKind::F32 => ChunkField::F32(section.get_parsed::<f32>(&key)?),
      ChunkFieldKind::Vector3 => ChunkField::Vector3(EditableLevelValue::read_vector3(section, &key)?.to_array()),
      ChunkFieldKind::String => ChunkField::String(EditableLevelValue::read_quoted(section, &key)?.to_owned()),
    })
  }

  fn is_ended(&self) -> bool {
    self.section.get(&Self::format_key(self.read + 1)).is_none()
  }
}
