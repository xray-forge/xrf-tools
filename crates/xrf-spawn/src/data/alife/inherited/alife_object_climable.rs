use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
use crate::data::alife::inherited::alife_object_shape::AlifeObjectShape;
use crate::data::alife::spawn_layout::SpawnLayout;

/// `CSE_ALifeObjectClimable`. Up to spawn version 126 the packet stores no game material, which then holds the
/// constructor's `materials\fake_ladders`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectClimable {
  pub base: AlifeObjectShape,
  pub game_material: String,
}

impl AlifeObjectClimable {
  /// The game material where the packet does not store it.
  pub(crate) const DEFAULT_GAME_MATERIAL: &'static str = "materials\\fake_ladders";

  /// The climbable object with the constructor's game material over `base`.
  fn new_default_material(base: AlifeObjectShape) -> Self {
    Self {
      base,
      game_material: String::from(Self::DEFAULT_GAME_MATERIAL),
    }
  }

  /// Refuse a game material the layout has no place for unless it is the constructor's.
  fn assert_material_fits(&self, layout: SpawnLayout) -> XrfResult {
    if layout.has_climable_material() {
      return Ok(());
    }

    layout.assert_absent_default(
      "game_material",
      &self.game_material.as_str(),
      &Self::DEFAULT_GAME_MATERIAL,
    )
  }
}

impl AlifeStateReadWrite for AlifeObjectClimable {
  fn read_state<R: ChunkFieldReader>(reader: &mut R, layout: SpawnLayout) -> XrfResult<Self> {
    let base: AlifeObjectShape = reader.read_xr()?;

    if !layout.has_climable_material() {
      return Ok(Self::new_default_material(base));
    }

    Ok(Self {
      base,
      game_material: reader.read_w1251_string()?,
    })
  }

  fn write_state<W: ChunkFieldWriter>(&self, writer: &mut W, layout: SpawnLayout) -> XrfResult {
    self.assert_material_fits(layout)?;
    writer.write_xr(&self.base)?;

    if layout.has_climable_material() {
      writer.write_w1251_string(&self.game_material)?;
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
    let base: AlifeObjectShape = AlifeObjectShape::import(section_name, ltx)?;

    if !layout.has_climable_material() {
      return Ok(Self::new_default_material(base));
    }

    Ok(Self {
      base,
      game_material: section.read_field("climbable.game_material")?,
    })
  }

  fn export_state(&self, section_name: &str, ltx: &mut impl LtxWrite, layout: SpawnLayout) -> XrfResult {
    self.assert_material_fits(layout)?;
    self.base.export(section_name, ltx)?;

    if layout.has_climable_material() {
      ltx
        .with_section(section_name)
        .set("climbable.game_material", &self.game_material);
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write, read_to_string};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
  use xrf_error::{MapXrfErr, XrfResult};
  use xrf_ltx::Ltx;
  use xrf_math::{Affine3d, Sphere, Vector3d};
  use xrf_test_utils::GeneratedResources;

  use crate::data::alife::alife_state_read_write::AlifeStateReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_climable::AlifeObjectClimable;
  use crate::data::alife::inherited::alife_object_shape::AlifeObjectShape;
  use crate::data::alife::spawn_layout::SpawnLayout;
  use crate::data::generic::shape::Shape;

  fn new_climable(game_material: &str) -> AlifeObjectClimable {
    AlifeObjectClimable {
      base: AlifeObjectShape {
        base: AlifeObjectAbstract {
          game_vertex_id: 4223,
          distance: 723.23,
          direct_control: 0,
          level_vertex_id: 0,
          flags: 0,
          custom_data: String::from("custom-data"),
          story_id: 0,
          spawn_story_id: 0,
        },
        shape: vec![
          Shape::Sphere(Sphere {
            center: Vector3d::new(54.5, 0.5, 11.5),
            radius: 1.0,
          }),
          Shape::Box(Affine3d {
            x_axis: Vector3d::new(51.5, 2.5, 73.1),
            y_axis: Vector3d::new(55.1, 3.2, 2.3),
            z_axis: Vector3d::new(51.0, 3.0, 6.4),
            translation: Vector3d::new(59.2, 3.3, 3.0),
          }),
        ],
      },
      game_material: String::from(game_material),
    }
  }

  fn layout(version: u16) -> SpawnLayout {
    SpawnLayout {
      version,
      script_version: 8,
    }
  }

  fn read_back(climable: &AlifeObjectClimable, layout: SpawnLayout) -> XrfResult<(usize, AlifeObjectClimable)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    climable.write_state(&mut writer, layout)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: AlifeObjectClimable = AlifeObjectClimable::read_state(&mut reader, layout)?;

    reader.assert_read("Expect the climbable object to be read whole")?;

    Ok((writer.buffer.len(), read))
  }

  #[test]
  fn writes_its_material_above_version_126_only() -> XrfResult {
    let climable: AlifeObjectClimable = new_climable(AlifeObjectClimable::DEFAULT_GAME_MATERIAL);
    let (with_material, read_with) = read_back(&climable, layout(127))?;
    let (without_material, read_without) = read_back(&climable, layout(126))?;

    assert_eq!(with_material, 128);
    assert_eq!(
      without_material,
      with_material - AlifeObjectClimable::DEFAULT_GAME_MATERIAL.len() - 1
    );
    assert_eq!(read_with, climable);
    assert_eq!(read_without, climable);

    let climable: AlifeObjectClimable = new_climable("dest-material");

    assert_eq!(read_back(&climable, layout(128))?, (119, climable));

    Ok(())
  }

  #[test]
  fn refuses_to_drop_its_material_up_to_version_126() {
    let climable: AlifeObjectClimable = new_climable("dest-material");
    let error: String = climable
      .write_state(&mut ChunkWriter::new(), layout(124))
      .expect_err("a material the packet cannot hold")
      .to_string();

    assert!(
      error.starts_with("Invalid error: Field 'game_material' holds"),
      "{error}"
    );
    assert!(climable.export_state("data", &mut Ltx::new(), layout(124)).is_err());
  }

  #[test]
  fn test_import_export() -> XrfResult {
    for (climable, version) in [
      (new_climable("dest-material"), 128),
      (new_climable(AlifeObjectClimable::DEFAULT_GAME_MATERIAL), 124),
    ] {
      let mut ltx: Ltx = Ltx::new();

      climable.export_state("data", &mut ltx, layout(version))?;

      assert_eq!(
        ltx.get_from("data", "climbable.game_material").is_some(),
        version == 128,
        "{version}"
      );
      assert_eq!(
        AlifeObjectClimable::import_state("data", &ltx, layout(version))?,
        climable
      );
    }

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectClimable = new_climable("dest-material");

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original).map_serde_err()?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(
      serde_json::from_str::<AlifeObjectClimable>(&serialized).map_serde_err()?,
      original
    );

    Ok(())
  }
}
