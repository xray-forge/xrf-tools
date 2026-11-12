use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::inherited::alife_object_dynamic::AlifeObjectDynamic;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::generic::shape::Shape;

/// `CSE_SmartCover`'s own fields. Below spawn version 128 the packet has no `can_fire`, which the engine then sets to
/// the combat cover flag (`xrServer_Objects_Alife_Smartcovers.cpp:110`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectSmartCover {
  pub base: AlifeObjectDynamic,
  pub shape: Vec<Shape>,
  pub description: String,
  pub hold_position_time: f32,
  pub enter_min_enemy_distance: f32,
  pub exit_min_enemy_distance: f32,
  pub is_combat_cover: u8,
  pub can_fire: u8,
}

impl AlifeObjectSmartCover {
  /// Refuse a layout whose smart cover takes its combat cover flag from the section's config rather than the packet:
  /// no value read or written here would be the one the engine holds.
  fn assert_combat_flag(layout: SpawnLayout) -> XrfResult {
    if layout.has_smart_cover_combat_flag() {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "A smart cover of spawn version {} takes its combat cover flag from its section's config, which the packet does \
       not carry; version {} is the first to store it",
      layout.version,
      SpawnLayout::SMART_COVER_COMBAT_FLAG_VERSION
    )))
  }

  /// Refuse a `can_fire` the layout has no field for unless it is the combat cover flag the engine copies into it.
  fn assert_fire_flag_fits(&self, layout: SpawnLayout) -> XrfResult {
    if layout.has_smart_cover_fire_flag() {
      return Ok(());
    }

    layout.assert_absent_default("can_fire", &self.can_fire, &self.is_combat_cover)
  }
}

impl AlifeStateReadWrite for AlifeObjectSmartCover {
  fn read_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self> {
    let base: AlifeObjectDynamic = reader.read_xr::<T, _>()?;
    let shape: Vec<Shape> = reader.read_xr_list::<T, Shape>()?;
    let description: String = reader.read_w1251_string()?;
    let hold_position_time: f32 = reader.read_f32::<T>()?;
    let enter_min_enemy_distance: f32 = reader.read_f32::<T>()?;
    let exit_min_enemy_distance: f32 = reader.read_f32::<T>()?;

    Self::assert_combat_flag(layout)?;

    let is_combat_cover: u8 = reader.read_u8()?;
    let can_fire: u8 = if layout.has_smart_cover_fire_flag() {
      reader.read_u8()?
    } else {
      is_combat_cover
    };

    Ok(Self {
      base,
      shape,
      description,
      hold_position_time,
      enter_min_enemy_distance,
      exit_min_enemy_distance,
      is_combat_cover,
      can_fire,
    })
  }

  fn write_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult {
    Self::assert_combat_flag(layout)?;
    self.assert_fire_flag_fits(layout)?;

    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr_list::<T, Shape>(&self.shape)?;
    writer.write_w1251_string(&self.description)?;
    writer.write_f32::<T>(self.hold_position_time)?;
    writer.write_f32::<T>(self.enter_min_enemy_distance)?;
    writer.write_f32::<T>(self.exit_min_enemy_distance)?;
    writer.write_u8(self.is_combat_cover)?;

    if layout.has_smart_cover_fire_flag() {
      writer.write_u8(self.can_fire)?;
    }

    Ok(())
  }

  fn import_state<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Self::assert_combat_flag(layout)?;

    let is_combat_cover: u8 = section.read_field("is_combat_cover")?;

    Ok(Self {
      base: AlifeObjectDynamic::import(section_name, ltx)?,
      shape: Shape::import_list(section)?,
      description: section.read_field("description")?,
      hold_position_time: section.read_field("hold_position_time")?,
      enter_min_enemy_distance: section.read_field("enter_min_enemy_distance")?,
      exit_min_enemy_distance: section.read_field("exit_min_enemy_distance")?,
      is_combat_cover,
      can_fire: if layout.has_smart_cover_fire_flag() {
        section.read_field("can_fire")?
      } else {
        is_combat_cover
      },
    })
  }

  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    Self::assert_combat_flag(layout)?;
    self.assert_fire_flag_fits(layout)?;
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("description", &self.description)
      .set("hold_position_time", self.hold_position_time)
      .set("enter_min_enemy_distance", self.enter_min_enemy_distance)
      .set("exit_min_enemy_distance", self.exit_min_enemy_distance)
      .set("exit_min_enemy_distance", self.exit_min_enemy_distance)
      .set("is_combat_cover", self.is_combat_cover);

    if layout.has_smart_cover_fire_flag() {
      ltx.with_section(section_name).set("can_fire", self.can_fire);
    }

    Shape::export_list(&self.shape, section_name, ltx);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::{Affine3d, Sphere, Vector3d};

  use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic::AlifeObjectDynamic;
  use crate::data::alife::inherited::alife_object_smart_cover::AlifeObjectSmartCover;
  use crate::data::alife::spawn_layout::SpawnLayout;
  use crate::data::generic::shape::Shape;

  fn new_smart_cover() -> AlifeObjectSmartCover {
    AlifeObjectSmartCover {
      base: AlifeObjectDynamic {
        base: AlifeObjectAbstract {
          game_vertex_id: 1001,
          distance: 65.25,
          direct_control: 412421,
          level_vertex_id: 66231,
          flags: 33,
          custom_data: String::from("custom_data"),
          story_id: 400,
          spawn_story_id: 32,
        },
      },
      shape: vec![
        Shape::Sphere(Sphere {
          center: Vector3d::new(0.5, 0.3, -0.125),
          radius: 2.5,
        }),
        Shape::Box(Affine3d {
          x_axis: Vector3d::new(1.1, 1.1, 3.1),
          y_axis: Vector3d::new(1.4, 2.2, 3.3),
          z_axis: Vector3d::new(4.0, 3.0, 5.4),
          translation: Vector3d::new(9.2, 8.3, 2.0),
        }),
      ],
      description: String::from("test-description"),
      hold_position_time: 4.532,
      enter_min_enemy_distance: 32.4,
      exit_min_enemy_distance: 25.3,
      is_combat_cover: 1,
      can_fire: 1,
    }
  }

  fn layout(version: u16) -> SpawnLayout {
    SpawnLayout {
      version,
      script_version: 8,
    }
  }

  /// The bytes written at `layout`, and the cover read back from them whole.
  fn read_back(cover: &AlifeObjectSmartCover, layout: SpawnLayout) -> XrfResult<(usize, AlifeObjectSmartCover)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    cover.write_state::<XRayByteOrder, _>(&mut writer, layout)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: AlifeObjectSmartCover = AlifeObjectSmartCover::read_state::<XRayByteOrder, _>(&mut reader, layout)?;

    reader.assert_read("Expect the smart cover to be read whole")?;

    Ok((writer.buffer.len(), read))
  }

  #[test]
  fn writes_can_fire_from_version_128_only() -> XrfResult {
    let cover: AlifeObjectSmartCover = new_smart_cover();
    let (with_fire, read_with) = read_back(&cover, layout(128))?;
    let (without_fire, read_without) = read_back(&cover, layout(127))?;

    assert_eq!(with_fire, 136);
    assert_eq!(without_fire, with_fire - 1);
    assert_eq!(read_with, cover);
    assert_eq!(read_without, cover);

    Ok(())
  }

  /// Below version 128 the engine copies the combat cover flag into `can_fire`, so reading holds it there.
  #[test]
  fn reads_can_fire_as_the_combat_cover_flag_below_version_128() -> XrfResult {
    let cover: AlifeObjectSmartCover = AlifeObjectSmartCover {
      is_combat_cover: 0,
      can_fire: 0,
      ..new_smart_cover()
    };

    assert_eq!(read_back(&cover, layout(124))?.1.can_fire, 0);

    Ok(())
  }

  #[test]
  fn refuses_to_drop_a_can_fire_below_version_128() {
    let cover: AlifeObjectSmartCover = AlifeObjectSmartCover {
      is_combat_cover: 1,
      can_fire: 0,
      ..new_smart_cover()
    };
    let expected: &str = "Invalid error: Field 'can_fire' holds 0, which spawn version 124 and script version 8 have no \
                          place for: the engine reads 1 there";

    assert_eq!(
      cover
        .write_state::<XRayByteOrder, _>(&mut ChunkWriter::new(), layout(124))
        .expect_err("a can_fire the packet cannot hold")
        .to_string(),
      expected
    );
    assert_eq!(
      cover
        .export_state("data", &mut Ltx::new(), layout(124))
        .expect_err("a can_fire the text cannot hold")
        .to_string(),
      expected
    );
  }

  /// Version 122 stores the combat cover flag; below it the engine takes it from the section's config.
  #[test]
  fn refuses_a_smart_cover_below_version_122() -> XrfResult {
    let cover: AlifeObjectSmartCover = new_smart_cover();
    let (bytes, read) = read_back(&cover, layout(122))?;

    assert_eq!(bytes, 135);
    assert_eq!(read, cover);

    let error: String = cover
      .write_state::<XRayByteOrder, _>(&mut ChunkWriter::new(), layout(121))
      .expect_err("a combat cover flag the packet does not store")
      .to_string();

    assert_eq!(
      error,
      "Invalid error: A smart cover of spawn version 121 takes its combat cover flag from its section's config, which \
       the packet does not carry; version 122 is the first to store it"
    );

    let mut writer: ChunkWriter = ChunkWriter::new();

    cover.write_state::<XRayByteOrder, _>(&mut writer, layout(122))?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(
      AlifeObjectSmartCover::read_state::<XRayByteOrder, _>(&mut reader, layout(121))
        .expect_err("a combat cover flag the packet does not store")
        .to_string(),
      error
    );

    Ok(())
  }

  /// Text at a layout without `can_fire` leaves its key out and imports it as the combat cover flag.
  #[test]
  fn exports_and_imports_without_can_fire_below_version_128() -> XrfResult {
    let cover: AlifeObjectSmartCover = new_smart_cover();
    let mut ltx: Ltx = Ltx::new();

    cover.export_state("data", &mut ltx, layout(124))?;

    assert_eq!(ltx.get_from("data", "can_fire"), None);
    assert_eq!(AlifeObjectSmartCover::import_state("data", &ltx, layout(124))?, cover);

    let mut ltx: Ltx = Ltx::new();

    cover.export_state("data", &mut ltx, layout(128))?;

    assert_eq!(ltx.get_from("data", "can_fire"), Some("1"));
    assert_eq!(AlifeObjectSmartCover::import_state("data", &ltx, layout(128))?, cover);

    Ok(())
  }
}
