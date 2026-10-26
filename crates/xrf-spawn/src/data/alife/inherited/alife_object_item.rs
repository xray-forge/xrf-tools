use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
#[cfg(test)]
use xrf_math::Vector3d;
use xrf_utils::{to_format_size, vector_from_string, vector_to_string};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
#[cfg(test)]
use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::physics::physics_net_state::PhysicsNetState;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItem {
  pub base: AlifeObjectDynamicVisual,
  pub condition: f32,
  /// The upgrade sections installed on the item (`CSE_ALifeInventoryItem::m_upgrades`).
  pub upgrades: Vec<String>,
  /// The physics state the update packet carries, whose position the engine moves the item to over the spawn
  /// packet's.
  pub update_state: Option<PhysicsNetState>,
}

impl AlifeObjectItem {
  /// The condition a byte `NET_Packet::r_float_q8` reads over `0..=1` stands for, as an item's update packet stores it.
  pub fn dequantize_condition(stored: u8) -> f32 {
    f32::from(stored) / 255.0001
  }
}

impl AlifePacketsReadWrite for AlifeObjectItem {
  /// Read the item's condition and upgrades, then the physics state of its update packet.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    let base: AlifeObjectDynamicVisual = spawn.read_xr::<T, _>()?;
    let condition: f32 = spawn.read_f32::<T>()?;
    let count: u32 = spawn.read_u32::<T>()?;
    // An upgrade is at least the terminator of an empty name.
    let mut upgrades: Vec<String> = spawn.new_bounded_vec(count.into(), 1, "item upgrades")?;

    for _ in 0..count {
      upgrades.push(spawn.read_w1251_string()?);
    }

    Ok(Self {
      base,
      condition,
      upgrades,
      update_state: PhysicsNetState::read_optional::<T, _>(update)?,
    })
  }

  /// Write the item's condition and upgrades, then the physics state of its update packet.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    spawn.write_xr::<T, _>(&self.base)?;
    spawn.write_f32::<T>(self.condition)?;
    spawn.write_u32::<T>(to_format_size(self.upgrades.len(), "item upgrades")?)?;

    for upgrade in &self.upgrades {
      spawn.write_w1251_string(upgrade)?;
    }

    PhysicsNetState::write_optional::<T>(self.update_state.as_ref(), update)
  }
}

impl LtxImportExport for AlifeObjectItem {
  /// Import ALife item object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      condition: section.read_field("item.condition")?,
      upgrades: vector_from_string(&section.read_field::<String>("item.upgrades")?)?,
      update_state: PhysicsNetState::import_optional(section, "item.update_state")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item.condition", self.condition.to_string())
      .set("item.upgrades", vector_to_string(&self.upgrades));

    PhysicsNetState::export_optional(self.update_state.as_ref(), section_name, "item.update_state", ltx);

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectItem {
  /// An item placed at `position` with a physics state there, as most placed items carry one.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 1002,
          distance: 65.25,
          direct_control: 41262,
          level_vertex_id: 618923,
          flags: 32,
          custom_data: String::from("custom_data"),
          story_id: 500,
          spawn_story_id: 35,
        },
        visual_name: String::from("abcd"),
        visual_flags: 33,
      },
      condition: 0.5,
      upgrades: Vec::new(),
      update_state: Some(PhysicsNetState::new_mock(position)),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};

  use serde_json::to_string_pretty;
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;
  use xrf_test_utils::GeneratedResources;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(original.read_back::<XRayByteOrder>()?, (52, 54, original));

    Ok(())
  }

  /// An item with no physics state writes the single zero byte the engine writes for one.
  #[test]
  fn reads_and_writes_an_item_without_a_physics_state() -> XrfResult {
    let mut original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::ZERO);

    original.update_state = None;

    assert_eq!(original.read_back::<XRayByteOrder>()?, (52, 1, original));

    Ok(())
  }

  /// An item placed upgraded carries each upgrade's section after the count, `save_data` of `m_upgrades`.
  #[test]
  fn reads_and_writes_the_upgrades_an_item_carries() -> XrfResult {
    let mut original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::ZERO);

    original.upgrades = vec![String::from("up_firsta_ak74"), String::from("up_secondb_ak74")];

    assert_eq!(original.read_back::<XRayByteOrder>()?.2, original);

    Ok(())
  }

  #[test]
  fn dequantizes_a_stored_condition_as_the_engine_does() {
    assert_eq!(AlifeObjectItem::dequantize_condition(0), 0.0);
    assert!(AlifeObjectItem::dequantize_condition(255) < 1.0);
    assert!((AlifeObjectItem::dequantize_condition(255) - 1.0).abs() < 1e-6);
    assert!((AlifeObjectItem::dequantize_condition(128) - 0.502).abs() < 1e-3);
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();
    let mut original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::new(-4.5, 0.25, 7.0));

    original.condition = 0.1;
    original.upgrades = vec![String::from("up_firsta_ak74"), String::from("up_secondb_ak74")];

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut GeneratedResources::overwrite(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&ltx_filename))?;

    assert_eq!(AlifeObjectItem::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let mut original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    original.condition = 0.65;
    original.upgrades = vec![String::from("up_firsta_ak74"), String::from("up_secondb_ak74")];

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectItem>(&serialized)?, original);

    Ok(())
  }
}
