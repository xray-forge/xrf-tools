use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::spawn_layout::SpawnLayout;

/// `CSE_ALifeInventoryBox`. Up to spawn version 124 the packet stores none of its own fields, which then hold the
/// constructor's: open to taking, not closed, the `inventory_box_use` tip (`xrServer_Objects_ALife.cpp:1879`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectInventoryBox {
  pub base: AlifeObjectDynamicVisual,
  pub can_take: u8,
  pub is_closed: u8,
  pub tip: String,
}

impl AlifeObjectInventoryBox {
  /// `can_take` where the packet does not store it.
  pub(crate) const DEFAULT_CAN_TAKE: u8 = 1;

  /// `is_closed` where the packet does not store it.
  pub(crate) const DEFAULT_IS_CLOSED: u8 = 0;

  /// The tip where the packet does not store it.
  pub(crate) const DEFAULT_TIP: &'static str = "inventory_box_use";

  /// The box with the constructor's access fields and tip over `base`.
  fn new_default_access(base: AlifeObjectDynamicVisual) -> Self {
    Self {
      base,
      can_take: Self::DEFAULT_CAN_TAKE,
      is_closed: Self::DEFAULT_IS_CLOSED,
      tip: String::from(Self::DEFAULT_TIP),
    }
  }

  /// Refuse access fields the layout has no place for unless they hold the constructor's values.
  fn assert_access_fits(&self, layout: SpawnLayout) -> XrfResult {
    if layout.has_inventory_box_access() {
      return Ok(());
    }

    layout.assert_absent_default("can_take", &self.can_take, &Self::DEFAULT_CAN_TAKE)?;
    layout.assert_absent_default("is_closed", &self.is_closed, &Self::DEFAULT_IS_CLOSED)?;
    layout.assert_absent_default("tip", &self.tip.as_str(), &Self::DEFAULT_TIP)
  }
}

impl AlifeStateReadWrite for AlifeObjectInventoryBox {
  fn read_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self> {
    let base: AlifeObjectDynamicVisual = reader.read_xr::<T, _>()?;

    if !layout.has_inventory_box_access() {
      return Ok(Self::new_default_access(base));
    }

    Ok(Self {
      base,
      can_take: reader.read_u8()?,
      is_closed: reader.read_u8()?,
      tip: reader.read_w1251_string()?,
    })
  }

  fn write_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult {
    self.assert_access_fits(layout)?;
    writer.write_xr::<T, _>(&self.base)?;

    if layout.has_inventory_box_access() {
      writer.write_u8(self.can_take)?;
      writer.write_u8(self.is_closed)?;
      writer.write_w1251_string(&self.tip)?;
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
    let base: AlifeObjectDynamicVisual = AlifeObjectDynamicVisual::import(section_name, ltx)?;

    if !layout.has_inventory_box_access() {
      return Ok(Self::new_default_access(base));
    }

    Ok(Self {
      base,
      can_take: section.read_field("inventory_box.can_take")?,
      is_closed: section.read_field("inventory_box.is_closed")?,
      tip: section.read_field("inventory_box.tip")?,
    })
  }

  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.assert_access_fits(layout)?;
    self.base.export(section_name, ltx)?;

    if layout.has_inventory_box_access() {
      ltx
        .with_section(section_name)
        .set("inventory_box.can_take", self.can_take)
        .set("inventory_box.is_closed", self.is_closed)
        .set("inventory_box.tip", &self.tip);
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_test_utils::GeneratedResources;

  use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_inventory_box::AlifeObjectInventoryBox;
  use crate::data::alife::spawn_layout::SpawnLayout;

  fn new_inventory_box(can_take: u8, is_closed: u8, tip: &str) -> AlifeObjectInventoryBox {
    AlifeObjectInventoryBox {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 2463,
          distance: 12.0,
          direct_control: 5634,
          level_vertex_id: 2533,
          flags: 64,
          custom_data: String::from("custom-data"),
          story_id: 2136,
          spawn_story_id: 0,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 0,
      },
      can_take,
      is_closed,
      tip: String::from(tip),
    }
  }

  fn layout(version: u16) -> SpawnLayout {
    SpawnLayout {
      version,
      script_version: 8,
    }
  }

  /// The bytes written at `layout`, and the box read back from them whole.
  fn read_back(
    inventory_box: &AlifeObjectInventoryBox,
    layout: SpawnLayout,
  ) -> XrfResult<(usize, AlifeObjectInventoryBox)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    inventory_box.write_state::<XRayByteOrder, _>(&mut writer, layout)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: AlifeObjectInventoryBox = AlifeObjectInventoryBox::read_state::<XRayByteOrder, _>(&mut reader, layout)?;

    reader.assert_read("Expect the inventory box to be read whole")?;

    Ok((writer.buffer.len(), read))
  }

  #[test]
  fn writes_access_fields_above_version_124_only() -> XrfResult {
    let inventory_box: AlifeObjectInventoryBox = new_inventory_box(1, 0, "inventory_box_use");
    let (with_access, read_with) = read_back(&inventory_box, layout(125))?;
    let (without_access, read_without) = read_back(&inventory_box, layout(124))?;

    // Two bytes and the tip with its terminator.
    assert_eq!(with_access, 71);
    assert_eq!(without_access, with_access - 2 - "inventory_box_use".len() - 1);
    assert_eq!(read_with, inventory_box);
    assert_eq!(read_without, inventory_box);

    let inventory_box: AlifeObjectInventoryBox = new_inventory_box(0, 1, "some-tip");

    assert_eq!(read_back(&inventory_box, layout(128))?.1, inventory_box);

    Ok(())
  }

  #[test]
  fn refuses_to_drop_access_fields_up_to_version_124() {
    for (inventory_box, field) in [
      (new_inventory_box(0, 0, "inventory_box_use"), "can_take"),
      (new_inventory_box(1, 1, "inventory_box_use"), "is_closed"),
      (new_inventory_box(1, 0, "some-tip"), "tip"),
    ] {
      let error: String = inventory_box
        .write_state::<XRayByteOrder, _>(&mut ChunkWriter::new(), layout(124))
        .expect_err("an access field the packet cannot hold")
        .to_string();

      assert!(
        error.starts_with(&format!("Invalid error: Field '{field}' holds")),
        "{error}"
      );
      assert!(
        inventory_box
          .export_state("data", &mut Ltx::new(), layout(124))
          .is_err(),
        "{field} in text"
      );
    }
  }

  #[test]
  fn test_import_export() -> XrfResult {
    for (inventory_box, version) in [
      (new_inventory_box(0, 1, "some-tip"), 128),
      (new_inventory_box(1, 0, "inventory_box_use"), 124),
    ] {
      let mut ltx: Ltx = Ltx::new();

      inventory_box.export_state("data", &mut ltx, layout(version))?;

      assert_eq!(
        ltx.get_from("data", "inventory_box.tip").is_some(),
        version == 128,
        "{version}"
      );
      assert_eq!(
        AlifeObjectInventoryBox::import_state("data", &ltx, layout(version))?,
        inventory_box
      );
    }

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectInventoryBox = new_inventory_box(1, 1, "some-tip");

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectInventoryBox>(&serialized)?, original);

    Ok(())
  }
}
