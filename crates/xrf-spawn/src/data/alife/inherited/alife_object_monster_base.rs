use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_monster_abstract::AlifeObjectMonsterAbstract;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;

/// A mutant, `CSE_ALifeMonsterBase`: a monster with a physics skeleton, which every monster class Call of Pripyat
/// registers is, the psy dog's phantom included. Its update packet is the monster's; the skeleton adds none.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectMonsterBase {
  pub base: AlifeObjectMonsterAbstract,
  pub skeleton: AlifeObjectSkeleton,
  /// The object the monster is bound to, `m_spec_object_id`: the psy dog a phantom belongs to; `u16::MAX` for none.
  pub spec_object_id: u16,
}

impl AlifeObjectMonsterBase {
  /// Place the monster at `position`, facing `direction`.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.base.move_to(position, direction);
  }
}

impl AlifePacketsReadWrite for AlifeObjectMonsterBase {
  /// Read the monster from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectMonsterAbstract::read::<T, _>(spawn, update)?,
      skeleton: spawn.read_xr::<T, _>()?,
      spec_object_id: spawn.read_u16::<T>()?,
    })
  }

  /// Write the monster into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;

    spawn.write_xr::<T, _>(&self.skeleton)?;
    spawn.write_u16::<T>(self.spec_object_id)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectMonsterBase {
  /// Import monster data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectMonsterAbstract::import(section_name, ltx)?,
      skeleton: AlifeObjectSkeleton::import(section_name, ltx)?,
      spec_object_id: section.read_field("monster.spec_object_id")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("monster.spec_object_id", self.spec_object_id.to_string());

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectMonsterBase {
  /// A monster at `position` as the SDK compiles one, with no skeleton state and bound to nothing.
  pub fn new_mock(position: Vector3d) -> Self {
    let mut base: AlifeObjectMonsterAbstract = AlifeObjectMonsterAbstract::new_mock(position);

    base.base.base.visual_name = String::from("monsters\\dog\\dog_red");

    Self {
      base,
      skeleton: AlifeObjectSkeleton {
        name: String::new(),
        flags: 0,
        source_id: u16::MAX,
      },
      spec_object_id: u16::MAX,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_monster_base::AlifeObjectMonsterBase;

  fn new_object() -> AlifeObjectMonsterBase {
    let mut object: AlifeObjectMonsterBase = AlifeObjectMonsterBase::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    object.skeleton.name = String::from("idle_0");
    object.spec_object_id = 4021;

    object
  }

  /// The skeleton's animation, flags and source, then the bound object, close the spawn packet; the update packet is
  /// the monster's 52 bytes.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectMonsterBase = new_object();
    let (spawn_size, update_size, read) = original.read_back::<XRayByteOrder>()?;

    assert_eq!(update_size, 52);
    assert_eq!(read, original);
    assert_eq!(
      spawn_size,
      original.base.read_back::<XRayByteOrder>()?.0 + 7 + 1 + 2 + 2
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectMonsterBase = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectMonsterBase::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectMonsterBase = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectMonsterBase>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
