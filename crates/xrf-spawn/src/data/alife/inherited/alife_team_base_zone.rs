use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;

/// A multiplayer team's base, `CSE_ALifeTeamBaseZone` (`zone_team_base`, class `Z_TEAMBS`): a space restrictor and the
/// team it belongs to, which the team modes respawn and score at.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeTeamBaseZone {
  pub base: AlifeObjectSpaceRestrictor,
  /// `m_team`, which the Level Editor offers from 0 to 16.
  pub team: u8,
}

impl ChunkFieldReadWrite for AlifeTeamBaseZone {
  /// Read the zone as `CSE_ALifeTeamBaseZone::STATE_Read` does: the restrictor, then the team.
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      team: reader.read_u8()?,
    })
  }

  /// Write the zone as `CSE_ALifeTeamBaseZone::STATE_Write` does.
  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_u8(self.team)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeTeamBaseZone {
  /// Import the zone from an ltx config section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectSpaceRestrictor::import(section_name, ltx)?,
      team: section.read_field("team_base_zone.team")?,
    })
  }

  /// Export the zone into an ltx file.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx.with_section(section_name).set("team_base_zone.team", self.team);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::{Affine3d, Vector3d};
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::alife::inherited::alife_team_base_zone::AlifeTeamBaseZone;
  use crate::data::generic::shape::Shape;

  fn team_base(team: u8) -> AlifeTeamBaseZone {
    AlifeTeamBaseZone {
      base: AlifeObjectSpaceRestrictor {
        base: AlifeObjectAbstract {
          game_vertex_id: 65535,
          distance: 0.0,
          direct_control: 1,
          level_vertex_id: 4294967295,
          flags: 0,
          custom_data: String::new(),
          story_id: 4294967295,
          spawn_story_id: 4294967295,
        },
        shape: vec![Shape::Box(Affine3d {
          x_axis: Vector3d::new(5.0, 0.0, 0.0),
          y_axis: Vector3d::new(0.0, 3.0, 0.0),
          z_axis: Vector3d::new(0.0, 0.0, 5.0),
          translation: Vector3d::new(0.0, 1.5, 0.0),
        })],
        restrictor_type: 3,
      },
      team,
    }
  }

  #[test]
  fn reads_the_team_after_the_restrictor() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "read_write.chunk");
    let original: AlifeTeamBaseZone = team_base(2);
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer.last(), Some(&2), "the team closes the packet");

    writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeTeamBaseZone::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn imports_what_it_exports() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
    let original: AlifeTeamBaseZone = team_base(1);
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;
    ltx.write_to(&mut GeneratedResources::overwrite(&filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&filename))?;

    assert_eq!(AlifeTeamBaseZone::import("data", &source)?, original);

    Ok(())
  }
}
