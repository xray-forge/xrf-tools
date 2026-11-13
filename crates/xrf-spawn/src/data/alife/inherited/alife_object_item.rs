use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
#[cfg(test)]
use xrf_math::Vector3d;
use xrf_utils::{narrow_format_size, vector_from_string, vector_to_string};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
#[cfg(test)]
use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
use crate::data::physics::physics_net_state::PhysicsNetState;

/// An item, `CSE_ALifeItem`: a dynamic visual and an inventory item. Up to spawn version 123 the packet stores no
/// upgrades, and the item holds none, as `CSE_ALifeInventoryItem`'s constructor leaves `m_upgrades`
/// (`xrServer_Objects_ALife_Items.cpp:27`, read above 123 at `:92`).
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

  /// The byte `NET_Packet::w_float_q8` stores a condition over `0..=1` as: `u8(iFloor(c * 255 + 0.5))`, wrapping as
  /// the engine's cast does.
  pub fn quantize_condition(condition: f32) -> u8 {
    (condition * 255.0 + 0.5).floor() as i32 as u8
  }

  /// Refuse upgrades the layout has no place for unless there are none, as the constructor leaves them.
  fn assert_upgrades_fit(&self, layout: SpawnLayout) -> XrfResult {
    if layout.has_item_upgrades() {
      return Ok(());
    }

    layout.assert_absent_default("upgrades", &self.upgrades, &Vec::new())
  }

  /// Read the upgrade count and that many sections, as `load_data` reads `m_upgrades`.
  fn read_upgrades<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Vec<String>> {
    let count: u32 = reader.read_u32::<T>()?;
    // An upgrade is at least the terminator of an empty name.
    let mut upgrades: Vec<String> = reader.new_bounded_vec(count.into(), 1, "item upgrades")?;

    for _ in 0..count {
      upgrades.push(reader.read_w1251_string()?);
    }

    Ok(upgrades)
  }
}

impl AlifePacketsReadWrite for AlifeObjectItem {
  /// Nothing to set: a fresh item's physics state byte is zero, no state (`CSE_ALifeInventoryItem`'s constructor in
  /// OpenXRay; GSC's leaves it uninitialised).
  fn set_spawned_update(&mut self, _: &SpawnPacketHeader) {}

  /// Read the item's condition and upgrades, then the physics state of its update packet.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: spawn.read_xr::<T, _>()?,
      condition: spawn.read_f32::<T>()?,
      upgrades: if layout.has_item_upgrades() {
        Self::read_upgrades::<T, _>(spawn)?
      } else {
        Vec::new()
      },
      update_state: update.read_xr_optional::<T, _>()?,
    })
  }

  /// Write the item's condition and upgrades, then the physics state of its update packet.
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult {
    self.assert_upgrades_fit(layout)?;
    spawn.write_xr::<T, _>(&self.base)?;
    spawn.write_f32::<T>(self.condition)?;

    if layout.has_item_upgrades() {
      spawn.write_u32::<T>(narrow_format_size(self.upgrades.len(), "item upgrades")?)?;

      for upgrade in &self.upgrades {
        spawn.write_w1251_string(upgrade)?;
      }
    }

    update.write_xr_optional::<T, _>(self.update_state.as_ref())
  }

  /// Import ALife item object data from ltx config section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      condition: section.read_field("item.condition")?,
      upgrades: if layout.has_item_upgrades() {
        vector_from_string(&section.read_field::<String>("item.upgrades")?)?
      } else {
        Vec::new()
      },
      update_state: PhysicsNetState::import_optional(section, "item.update_state")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.assert_upgrades_fit(layout)?;
    self.base.export(section_name, ltx)?;

    ltx.with_section(section_name).set("item.condition", self.condition);

    if layout.has_item_upgrades() {
      ltx
        .with_section(section_name)
        .set("item.upgrades", vector_to_string(&self.upgrades));
    }

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
  use xrf_chunk::{ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;
  use xrf_test_utils::GeneratedResources;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
  use crate::data::alife::spawn_layout::SpawnLayout;

  fn layout(version: u16) -> SpawnLayout {
    SpawnLayout {
      version,
      script_version: 8,
    }
  }

  /// The upgrade count is the one spawn field the layout gates; the update packet is the same at every version.
  #[test]
  fn writes_upgrades_above_version_123_only() -> XrfResult {
    let original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let (with_upgrades, update_with, read_with) = original.read_back::<XRayByteOrder>(layout(124))?;
    let (without_upgrades, update_without, read_without) = original.read_back::<XRayByteOrder>(layout(123))?;

    assert_eq!(with_upgrades, 52);
    assert_eq!(without_upgrades, with_upgrades - 4);
    assert_eq!(update_without, update_with);
    assert_eq!(read_with, original);
    assert_eq!(read_without, original);

    Ok(())
  }

  #[test]
  fn refuses_to_drop_upgrades_up_to_version_123() {
    let mut original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::ZERO);

    original.upgrades = vec![String::from("up_firsta_ak74")];

    let error: String = original
      .write::<XRayByteOrder, _, _>(&mut ChunkWriter::new(), &mut ChunkWriter::new(), layout(123))
      .expect_err("upgrades the packet cannot hold")
      .to_string();

    assert!(error.starts_with("Invalid error: Field 'upgrades' holds"), "{error}");
    assert!(original.export("data", &mut Ltx::new(), layout(123)).is_err());
  }

  #[test]
  fn imports_and_exports_no_upgrades_up_to_version_123() -> XrfResult {
    let original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::new(-4.5, 0.25, 7.0));
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx, layout(123))?;

    assert_eq!(ltx.get_from("data", "item.upgrades"), None);
    assert_eq!(AlifeObjectItem::import("data", &ltx, layout(123))?, original);

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?,
      (52, 54, original)
    );

    Ok(())
  }

  /// An item with no physics state writes the single zero byte the engine writes for one.
  #[test]
  fn reads_and_writes_an_item_without_a_physics_state() -> XrfResult {
    let mut original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::ZERO);

    original.update_state = None;

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?,
      (52, 1, original)
    );

    Ok(())
  }

  /// An item placed upgraded carries each upgrade's section after the count, `save_data` of `m_upgrades`.
  #[test]
  fn reads_and_writes_the_upgrades_an_item_carries() -> XrfResult {
    let mut original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::ZERO);

    original.upgrades = vec![String::from("up_firsta_ak74"), String::from("up_secondb_ak74")];

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?.2,
      original
    );

    Ok(())
  }

  #[test]
  fn dequantizes_a_stored_condition_as_the_engine_does() {
    assert_eq!(AlifeObjectItem::dequantize_condition(0), 0.0);
    assert!(AlifeObjectItem::dequantize_condition(255) < 1.0);
    assert!((AlifeObjectItem::dequantize_condition(255) - 1.0).abs() < 1e-6);
    assert!((AlifeObjectItem::dequantize_condition(128) - 0.502).abs() < 1e-3);
  }

  /// Rounded to the nearest of 255 steps, and wrapped as the engine's `u8` cast wraps a value outside `0..=1`.
  #[test]
  fn quantizes_a_condition_as_the_engine_does() {
    assert_eq!(AlifeObjectItem::quantize_condition(0.0), 0);
    assert_eq!(AlifeObjectItem::quantize_condition(1.0), 255);
    assert_eq!(AlifeObjectItem::quantize_condition(0.502), 128);
    assert_eq!(AlifeObjectItem::quantize_condition(1.5), 127);
    assert_eq!(
      AlifeObjectItem::quantize_condition(AlifeObjectItem::dequantize_condition(37)),
      37
    );
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();
    let mut original: AlifeObjectItem = AlifeObjectItem::new_mock(Vector3d::new(-4.5, 0.25, 7.0));

    original.condition = 0.1;
    original.upgrades = vec![String::from("up_firsta_ak74"), String::from("up_secondb_ak74")];

    original.export("data", &mut ltx, SpawnLayout::CALL_OF_PRIPYAT)?;

    ltx.write_to(&mut GeneratedResources::overwrite(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&ltx_filename))?;

    assert_eq!(
      AlifeObjectItem::import("data", &source, SpawnLayout::CALL_OF_PRIPYAT)?,
      original
    );

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
