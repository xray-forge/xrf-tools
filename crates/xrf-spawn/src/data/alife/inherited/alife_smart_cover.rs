use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
use xrf_utils::narrow_format_size;

use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::inherited::alife_object_smart_cover::AlifeObjectSmartCover;
use crate::data::alife::inherited::alife_smart_cover_loophole::AlifeSmartCoverLoophole;
use crate::data::alife::spawn_layout::SpawnLayout;

/// `se_smart_cover`, the script class over the server smart cover. Below script version 9 its packet has no last
/// description or loopholes, which then hold the script constructor's empty values (`se_smart_cover.script`'s
/// `__init`); Call of Pripyat's script fills them from its smart cover descriptions, script config no packet carries.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeSmartCover {
  pub base: AlifeObjectSmartCover,
  pub last_description: String,
  pub loopholes: Vec<AlifeSmartCoverLoophole>,
}

impl AlifeSmartCover {
  /// The script smart cover over `base` with the constructor's empty last description and loopholes.
  fn new_without_script_fields(base: AlifeObjectSmartCover) -> Self {
    Self {
      base,
      last_description: String::new(),
      loopholes: Vec::new(),
    }
  }

  /// Refuse script fields the layout has no place for unless they hold the constructor's empty values.
  fn assert_loopholes_fit(&self, layout: SpawnLayout) -> XrfResult {
    if layout.has_smart_cover_loopholes() {
      return Ok(());
    }

    layout.assert_absent_default("last_description", &self.last_description, &String::new())?;
    layout.assert_absent_default("loopholes", &self.loopholes, &Vec::new())
  }
}

impl AlifeStateReadWrite for AlifeSmartCover {
  fn read_state<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self> {
    let base: AlifeObjectSmartCover = AlifeObjectSmartCover::read_state::<T, _>(reader, layout)?;

    if !layout.has_smart_cover_loopholes() {
      return Ok(Self::new_without_script_fields(base));
    }

    let last_description: String = reader.read_w1251_string()?;
    let count: u8 = reader.read_u8()?;
    let mut loopholes: Vec<AlifeSmartCoverLoophole> = reader.new_bounded_vec(
      count.into(),
      AlifeSmartCoverLoophole::MIN_SERIALIZED_SIZE,
      "smart cover loopholes",
    )?;

    for _ in 0..count {
      let name: String = reader.read_w1251_string()?;
      let enabled: u8 = reader.read_u8()?;

      loopholes.push(AlifeSmartCoverLoophole { name, enabled });
    }

    Ok(Self {
      base,
      last_description,
      loopholes,
    })
  }

  fn write_state<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult {
    self.assert_loopholes_fit(layout)?;
    self.base.write_state::<T, _>(writer, layout)?;

    if !layout.has_smart_cover_loopholes() {
      return Ok(());
    }

    writer.write_w1251_string(&self.last_description)?;
    writer.write_u8(narrow_format_size(self.loopholes.len(), "smart cover loopholes")?)?;

    for loophole in &self.loopholes {
      writer.write_w1251_string(&loophole.name)?;
      writer.write_u8(loophole.enabled)?;
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
    let base: AlifeObjectSmartCover = AlifeObjectSmartCover::import_state(section_name, ltx, layout)?;

    if !layout.has_smart_cover_loopholes() {
      return Ok(Self::new_without_script_fields(base));
    }

    Ok(Self {
      base,
      last_description: section.read_field("last_description")?,
      loopholes: AlifeSmartCoverLoophole::string_to_list(&section.read_field::<String>("loopholes")?)?,
    })
  }

  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.assert_loopholes_fit(layout)?;
    self.base.export_state(section_name, ltx, layout)?;

    if !layout.has_smart_cover_loopholes() {
      return Ok(());
    }

    ltx
      .with_section(section_name)
      .set("last_description", &self.last_description)
      .set("loopholes", self.loopholes.len())
      .set("loopholes", AlifeSmartCoverLoophole::list_to_string(&self.loopholes));

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::{Sphere, Vector3d};

  use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic::AlifeObjectDynamic;
  use crate::data::alife::inherited::alife_object_smart_cover::AlifeObjectSmartCover;
  use crate::data::alife::inherited::alife_smart_cover::AlifeSmartCover;
  use crate::data::alife::inherited::alife_smart_cover_loophole::AlifeSmartCoverLoophole;
  use crate::data::alife::spawn_layout::SpawnLayout;
  use crate::data::generic::shape::Shape;

  fn new_smart_cover(last_description: &str, loopholes: Vec<AlifeSmartCoverLoophole>) -> AlifeSmartCover {
    AlifeSmartCover {
      base: AlifeObjectSmartCover {
        base: AlifeObjectDynamic {
          base: AlifeObjectAbstract {
            game_vertex_id: 6734,
            distance: 38.287,
            direct_control: 234760,
            level_vertex_id: 29836,
            flags: 68,
            custom_data: String::from("custom-data"),
            story_id: 8723,
            spawn_story_id: 160278,
          },
        },
        shape: vec![Shape::Sphere(Sphere {
          center: Vector3d::new(2.5, 1.3, -4.125),
          radius: 5.5,
        })],
        description: String::from("description"),
        hold_position_time: 34.0,
        enter_min_enemy_distance: 23.0,
        exit_min_enemy_distance: 36.0,
        is_combat_cover: 1,
        can_fire: 1,
      },
      last_description: String::from(last_description),
      loopholes,
    }
  }

  fn layout(script_version: u16) -> SpawnLayout {
    SpawnLayout {
      version: 128,
      script_version,
    }
  }

  /// The bytes written at `layout`, and the cover read back from them whole.
  fn read_back(cover: &AlifeSmartCover, layout: SpawnLayout) -> XrfResult<(Vec<u8>, AlifeSmartCover)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    cover.write_state::<XRayByteOrder, _>(&mut writer, layout)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: AlifeSmartCover = AlifeSmartCover::read_state::<XRayByteOrder, _>(&mut reader, layout)?;

    reader.assert_read("Expect the smart cover to be read whole")?;

    Ok((writer.buffer, read))
  }

  #[test]
  fn writes_its_script_fields_from_script_version_9_only() -> XrfResult {
    let cover: AlifeSmartCover = new_smart_cover("", Vec::new());
    let (with_fields, read_with) = read_back(&cover, layout(9))?;
    let (without_fields, read_without) = read_back(&cover, layout(8))?;

    // An empty description's terminator and a zero loophole count.
    assert_eq!(with_fields.len(), without_fields.len() + 2);
    assert_eq!(read_with, cover);
    assert_eq!(read_without, cover);

    let cover: AlifeSmartCover = new_smart_cover(
      "last-description",
      vec![AlifeSmartCoverLoophole {
        name: String::from("loophole"),
        enabled: 1,
      }],
    );

    assert_eq!(read_back(&cover, layout(12))?.1, cover);

    Ok(())
  }

  #[test]
  fn refuses_to_drop_script_fields_below_script_version_9() {
    for (cover, field) in [
      (new_smart_cover("last-description", Vec::new()), "last_description"),
      (
        new_smart_cover(
          "",
          vec![AlifeSmartCoverLoophole {
            name: String::from("loophole"),
            enabled: 1,
          }],
        ),
        "loopholes",
      ),
    ] {
      let error: String = cover
        .write_state::<XRayByteOrder, _>(&mut ChunkWriter::new(), layout(8))
        .expect_err("a script field the packet cannot hold")
        .to_string();

      assert!(
        error.starts_with(&format!("Invalid error: Field '{field}' holds")),
        "{error}"
      );
      assert!(
        cover.export_state("data", &mut Ltx::new(), layout(8)).is_err(),
        "{field} in text"
      );
    }
  }

  /// Text below script version 9 leaves the script fields out and imports the constructor's empty values.
  #[test]
  fn exports_and_imports_without_script_fields_below_script_version_9() -> XrfResult {
    let cover: AlifeSmartCover = new_smart_cover("", Vec::new());
    let mut ltx: Ltx = Ltx::new();

    cover.export_state("data", &mut ltx, layout(8))?;

    assert_eq!(ltx.get_from("data", "last_description"), None);
    assert_eq!(ltx.get_from("data", "loopholes"), None);
    assert_eq!(AlifeSmartCover::import_state("data", &ltx, layout(8))?, cover);

    Ok(())
  }

  #[test]
  fn rejects_a_loophole_list_past_its_count_field() {
    let original: AlifeSmartCover = new_smart_cover(
      "last-description",
      vec![
        AlifeSmartCoverLoophole {
          name: String::from("loophole"),
          enabled: 1,
        };
        256
      ],
    );

    assert_eq!(
      original
        .write_state::<XRayByteOrder, _>(&mut ChunkWriter::new(), layout(12))
        .expect_err("expect the loophole count to exceed its format field")
        .to_string(),
      "Invalid error: smart cover loopholes exceeds the u8 format limit"
    );
  }

  #[test]
  fn rejects_loophole_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let original: AlifeSmartCover = new_smart_cover("last-description", Vec::new());
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write_state::<XRayByteOrder, _>(&mut writer, layout(12))?;

    // The loophole count is the last byte written, so raising it declares loopholes the payload cannot hold.
    let mut bytes: Vec<u8> = writer.buffer;

    *bytes.last_mut().expect("expect a trailing loophole count") = 255;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?;

    let error: String = AlifeSmartCover::read_state::<XRayByteOrder, _>(&mut reader, layout(12))
      .expect_err("expect the declared loophole count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("smart cover loopholes declares 255 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
