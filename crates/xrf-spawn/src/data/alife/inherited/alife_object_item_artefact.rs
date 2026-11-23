use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;
use xrf_ltx::{LtxRead, LtxWrite, Section};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;

/// An artefact, `CSE_ALifeItemArtefact`, which adds nothing to either packet of the item it is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemArtefact {
  pub base: AlifeObjectItem,
}

impl AlifeObjectItemArtefact {
  /// Whether an artefact of `section` lies hidden on the level until a detector reveals it: `can_be_controlled` with an
  /// `af_rank` other than 0. Visible on spawn, it hides on its first update (`SArtefactDetectorsSupport::UpdateOnFrame`,
  /// its switch time starting at 0) with its model, particles and light, and a detector shows it again for 5 seconds.
  pub fn is_detector_hidden(section: &Section) -> bool {
    section.get_bool("can_be_controlled") == Some(true) && section.get_f32("af_rank").is_some_and(|rank| rank != 0.0)
  }
}

impl AlifePacketsReadWrite for AlifeObjectItemArtefact {
  fn set_spawned_update(&mut self, header: &SpawnPacketHeader) {
    self.base.set_spawned_update(header);
  }

  /// Read an artefact from both packets.
  fn read<S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read(spawn, update, layout)?,
    })
  }

  /// Write an artefact into both packets.
  fn write<S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    layout: SpawnLayout,
  ) -> XrfResult {
    self.base.write(spawn, update, layout)
  }

  /// Import an artefact from ltx config section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, layout: SpawnLayout) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::import(section_name, ltx, layout)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.base.export(section_name, ltx, layout)
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::{MapXrfErr, XrfResult};
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
  use crate::data::alife::inherited::alife_object_item_artefact::AlifeObjectItemArtefact;
  use crate::data::alife::spawn_layout::SpawnLayout;

  fn new_object() -> AlifeObjectItemArtefact {
    AlifeObjectItemArtefact {
      base: AlifeObjectItem::new_mock(Vector3d::new(5.0, -1.5, 12.25)),
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemArtefact = new_object();

    assert_eq!(original.read_back(SpawnLayout::CALL_OF_PRIPYAT)?, (52, 54, original));

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemArtefact = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx, SpawnLayout::CALL_OF_PRIPYAT)?;

    assert_eq!(
      AlifeObjectItemArtefact::import("data", &ltx, SpawnLayout::CALL_OF_PRIPYAT)?,
      original
    );

    Ok(())
  }

  // Only a controlled artefact of a rank other than 0 hides; a section saying neither hides nothing.
  #[test]
  fn hides_until_detected_only_a_controlled_artefact_of_a_rank() -> XrfResult {
    let ltx: Ltx = Ltx::read_from_str(
      r"
[ranked]
can_be_controlled = true
af_rank = 2

[junk]
can_be_controlled = true
af_rank = 0

[uncontrolled]
can_be_controlled = false
af_rank = 1

[unranked]
can_be_controlled = true
",
    )?;
    let hides = |name: &str| {
      ltx
        .section(name)
        .is_some_and(AlifeObjectItemArtefact::is_detector_hidden)
    };

    assert!(hides("ranked"));
    assert!(!hides("junk"));
    assert!(!hides("uncontrolled"));
    assert!(!hides("unranked"));

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemArtefact = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectItemArtefact>(&serde_json::to_string(&original).map_serde_err()?)
        .map_serde_err()?,
      original
    );

    Ok(())
  }
}
