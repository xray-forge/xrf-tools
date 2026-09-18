use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectHangingLamp {
  pub base: AlifeObjectDynamicVisual,
  pub skeleton: AlifeObjectSkeleton,
  pub main_color: u32,
  pub main_brightness: f32,
  pub color_animator: String,
  pub main_range: f32,
  pub light_flags: u16,
  pub startup_animation: String,
  pub fixed_bones: String,
  pub health: f32,
  pub virtual_size: f32,
  pub ambient_radius: f32,
  pub ambient_power: f32,
  pub ambient_texture: String,
  pub light_texture: String,
  pub light_bone: String,
  pub spot_cone_angle: f32,
  pub glow_texture: String,
  pub glow_radius: f32,
  pub light_ambient_bone: String,
  pub volumetric_quality: f32,
  pub volumetric_intensity: f32,
  pub volumetric_distance: f32,
}

impl ChunkReadWrite for AlifeObjectHangingLamp {
  /// Read hanging lamp data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      skeleton: reader.read_xr::<T, _>()?,
      main_color: reader.read_u32::<T>()?,
      main_brightness: reader.read_f32::<T>()?,
      color_animator: reader.read_w1251_string()?,
      main_range: reader.read_f32::<T>()?,
      light_flags: reader.read_u16::<T>()?,
      startup_animation: reader.read_w1251_string()?,
      fixed_bones: reader.read_w1251_string()?,
      health: reader.read_f32::<T>()?,
      virtual_size: reader.read_f32::<T>()?,
      ambient_radius: reader.read_f32::<T>()?,
      ambient_power: reader.read_f32::<T>()?,
      ambient_texture: reader.read_w1251_string()?,
      light_texture: reader.read_w1251_string()?,
      light_bone: reader.read_w1251_string()?,
      spot_cone_angle: reader.read_f32::<T>()?,
      glow_texture: reader.read_w1251_string()?,
      glow_radius: reader.read_f32::<T>()?,
      light_ambient_bone: reader.read_w1251_string()?,
      volumetric_quality: reader.read_f32::<T>()?,
      volumetric_intensity: reader.read_f32::<T>()?,
      volumetric_distance: reader.read_f32::<T>()?,
    })
  }

  /// Write skeleton data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr::<T, _>(&self.skeleton)?;

    writer.write_u32::<T>(self.main_color)?;
    writer.write_f32::<T>(self.main_brightness)?;
    writer.write_w1251_string(&self.color_animator)?;
    writer.write_f32::<T>(self.main_range)?;
    writer.write_u16::<T>(self.light_flags)?;
    writer.write_w1251_string(&self.startup_animation)?;
    writer.write_w1251_string(&self.fixed_bones)?;
    writer.write_f32::<T>(self.health)?;

    writer.write_f32::<T>(self.virtual_size)?;
    writer.write_f32::<T>(self.ambient_radius)?;
    writer.write_f32::<T>(self.ambient_power)?;
    writer.write_w1251_string(&self.ambient_texture)?;
    writer.write_w1251_string(&self.light_texture)?;
    writer.write_w1251_string(&self.light_bone)?;
    writer.write_f32::<T>(self.spot_cone_angle)?;
    writer.write_w1251_string(&self.glow_texture)?;
    writer.write_f32::<T>(self.glow_radius)?;

    writer.write_w1251_string(&self.light_ambient_bone)?;
    writer.write_f32::<T>(self.volumetric_quality)?;
    writer.write_f32::<T>(self.volumetric_intensity)?;
    writer.write_f32::<T>(self.volumetric_distance)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectHangingLamp {
  /// Import ALife hanging lamp object data from ltx config section.
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
      skeleton: AlifeObjectSkeleton::import(section_name, ltx)?,
      main_color: read_ltx_field("hanging_lamp.main_color", section)?,
      main_brightness: read_ltx_field("hanging_lamp.main_brightness", section)?,
      color_animator: read_ltx_field("hanging_lamp.color_animator", section)?,
      main_range: read_ltx_field("hanging_lamp.main_range", section)?,
      light_flags: read_ltx_field("hanging_lamp.light_flags", section)?,
      startup_animation: read_ltx_field("hanging_lamp.startup_animation", section)?,
      fixed_bones: read_ltx_field("hanging_lamp.fixed_bones", section)?,
      health: read_ltx_field("hanging_lamp.health", section)?,
      virtual_size: read_ltx_field("hanging_lamp.virtual_size", section)?,
      ambient_radius: read_ltx_field("hanging_lamp.ambient_radius", section)?,
      ambient_power: read_ltx_field("hanging_lamp.ambient_power", section)?,
      ambient_texture: read_ltx_field("hanging_lamp.ambient_texture", section)?,
      light_texture: read_ltx_field("hanging_lamp.light_texture", section)?,
      light_bone: read_ltx_field("hanging_lamp.light_bone", section)?,
      spot_cone_angle: read_ltx_field("hanging_lamp.spot_cone_angle", section)?,
      glow_texture: read_ltx_field("hanging_lamp.glow_texture", section)?,
      glow_radius: read_ltx_field("hanging_lamp.glow_radius", section)?,
      light_ambient_bone: read_ltx_field("hanging_lamp.light_ambient_bone", section)?,
      volumetric_quality: read_ltx_field("hanging_lamp.volumetric_quality", section)?,
      volumetric_intensity: read_ltx_field("hanging_lamp.volumetric_intensity", section)?,
      volumetric_distance: read_ltx_field("hanging_lamp.volumetric_distance", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("hanging_lamp.main_color", self.main_color.to_string())
      .set("hanging_lamp.main_brightness", self.main_brightness.to_string())
      .set("hanging_lamp.color_animator", &self.color_animator)
      .set("hanging_lamp.main_range", self.main_range.to_string())
      .set("hanging_lamp.light_flags", self.light_flags.to_string())
      .set("hanging_lamp.startup_animation", &self.startup_animation)
      .set("hanging_lamp.fixed_bones", &self.fixed_bones)
      .set("hanging_lamp.health", self.health.to_string())
      .set("hanging_lamp.virtual_size", self.virtual_size.to_string())
      .set("hanging_lamp.ambient_radius", self.ambient_radius.to_string())
      .set("hanging_lamp.ambient_power", self.ambient_power.to_string())
      .set("hanging_lamp.ambient_texture", &self.ambient_texture)
      .set("hanging_lamp.light_texture", &self.light_texture)
      .set("hanging_lamp.light_bone", &self.light_bone)
      .set("hanging_lamp.spot_cone_angle", self.spot_cone_angle.to_string())
      .set("hanging_lamp.glow_texture", &self.glow_texture)
      .set("hanging_lamp.glow_radius", self.glow_radius.to_string())
      .set("hanging_lamp.light_ambient_bone", &self.light_ambient_bone)
      .set("hanging_lamp.volumetric_quality", self.volumetric_quality.to_string())
      .set(
        "hanging_lamp.volumetric_intensity",
        self.volumetric_intensity.to_string(),
      )
      .set("hanging_lamp.volumetric_distance", self.volumetric_distance.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_ltx::LtxImportExport;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_hanging_lamp::AlifeObjectHangingLamp;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectHangingLamp = AlifeObjectHangingLamp {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 15,
          distance: 7634.124,
          direct_control: 253,
          level_vertex_id: 3456,
          flags: 34,
          custom_data: String::from("custom-data"),
          story_id: 6987,
          spawn_story_id: 3986,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 168,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 0,
        source_id: 978,
      },
      main_color: 52323,
      main_brightness: 1.0,
      color_animator: String::from("color-animator"),
      main_range: 0.5,
      light_flags: 425,
      startup_animation: String::from("setup-animation"),
      fixed_bones: String::from("fixed-bones"),
      health: 1.0,
      virtual_size: 0.7,
      ambient_radius: 24.0,
      ambient_power: 52.0,
      ambient_texture: String::from("ambient-texture"),
      light_texture: String::from("light-texture"),
      light_bone: String::from("light-bone"),
      spot_cone_angle: 5.23,
      glow_texture: String::from("glow-texture"),
      glow_radius: 15.43,
      light_ambient_bone: String::from("light-ambient-bone"),
      volumetric_quality: 1.3,
      volumetric_intensity: 2.2,
      volumetric_distance: 3.1,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 234);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 234);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 234 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectHangingLamp::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObjectHangingLamp = AlifeObjectHangingLamp {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 1,
          distance: 52.124,
          direct_control: 354,
          level_vertex_id: 3456,
          flags: 25,
          custom_data: String::from("custom-data"),
          story_id: 345,
          spawn_story_id: 346,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 156,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 32,
        source_id: 346,
      },
      main_color: 364,
      main_brightness: 1.0,
      color_animator: String::from("color-animator"),
      main_range: 0.5,
      light_flags: 425,
      startup_animation: String::from("setup-animation"),
      fixed_bones: String::from("fixed-bones"),
      health: 1.0,
      virtual_size: 0.27,
      ambient_radius: 24.0,
      ambient_power: 52.0,
      ambient_texture: String::from("ambient-texture"),
      light_texture: String::from("light-texture"),
      light_bone: String::from("light-bone"),
      spot_cone_angle: 25.23,
      glow_texture: String::from("glow-texture"),
      glow_radius: 16.43,
      light_ambient_bone: String::from("light-ambient-bone"),
      volumetric_quality: 21.3,
      volumetric_intensity: 22.2,
      volumetric_distance: 23.1,
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectHangingLamp::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectHangingLamp = AlifeObjectHangingLamp {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 16,
          distance: 263.124,
          direct_control: 25,
          level_vertex_id: 634,
          flags: 76,
          custom_data: String::from("custom-data"),
          story_id: 6987,
          spawn_story_id: 15,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 168,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 25,
        source_id: 15,
      },
      main_color: 25,
      main_brightness: 1.0,
      color_animator: String::from("color-animator"),
      main_range: 0.54,
      light_flags: 52,
      startup_animation: String::from("setup-animation"),
      fixed_bones: String::from("fixed-bones"),
      health: 1.0,
      virtual_size: 0.75,
      ambient_radius: 24.0,
      ambient_power: 13.0,
      ambient_texture: String::from("ambient-texture"),
      light_texture: String::from("light-texture"),
      light_bone: String::from("light-bone"),
      spot_cone_angle: 56.23,
      glow_texture: String::from("glow-texture"),
      glow_radius: 16.43,
      light_ambient_bone: String::from("light-ambient-bone"),
      volumetric_quality: 10.3,
      volumetric_intensity: 20.2,
      volumetric_distance: 30.1,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectHangingLamp>(&serialized)?, original);

    Ok(())
  }
}
