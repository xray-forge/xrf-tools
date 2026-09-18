use byteorder::{ByteOrder, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{
  ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, find_optional_chunk_by_id, find_required_chunk_by_id,
  read_f32_chunk, read_f32_vector_chunk, read_u16_chunk, read_u32_chunk, read_w1251_string_chunk,
};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_math::Vector3d;
use xrf_utils::assert_equal;

use crate::constants::META_TYPE_FIELD;
use crate::data::particles::particle_action::ParticleAction;
use crate::data::particles::particle_effect_collision::ParticleEffectCollision;
use crate::data::particles::particle_effect_description::ParticleDescription;
use crate::data::particles::particle_effect_editor_data::ParticleEffectEditorData;
use crate::data::particles::particle_effect_frame::ParticleEffectFrame;
use crate::data::particles::particle_effect_sprite::ParticleEffectSprite;
use crate::export::LtxImportExport;
use crate::file_import::{read_ltx_field, read_ltx_optional_field};

/// C++ src/Layers/xrRender/ParticleEffectDef.cpp
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEffect {
  pub version: u16,
  pub name: String,
  pub max_particles: u32,
  pub actions: Vec<ParticleAction>,
  pub flags: u32,
  pub frame: Option<ParticleEffectFrame>,
  pub sprite: ParticleEffectSprite,
  pub time_limit: Option<f32>,
  pub collision: Option<ParticleEffectCollision>,
  pub velocity_scale: Option<Vector3d>,
  pub description: Option<ParticleDescription>,
  pub rotation: Option<Vector3d>,
  pub editor_data: Option<ParticleEffectEditorData>,
}

impl ParticleEffect {
  pub const META_TYPE: &'static str = "particle_effect";

  pub const EFFECT_ACTIONS_LIMIT: usize = 10_000;

  pub const VERSION_CHUNK_ID: u32 = 1;
  pub const NAME_CHUNK_ID: u32 = 2;
  pub const MAX_PARTICLES_CHUNK_ID: u32 = 3;
  pub const ACTION_LIST_CHUNK_ID: u32 = 4;
  pub const FLAGS_CHUNK_ID: u32 = 5;
  pub const FRAME_CHUNK_ID: u32 = 6;
  pub const SPRITE_CHUNK_ID: u32 = 7;
  pub const TIME_LIMIT_CHUNK_ID: u32 = 8;
  pub const TIME_LIMIT_OLD_CHUNK_ID: u32 = 9;
  pub const SOURCE_TEXT_CHUNK_ID: u32 = 32;
  pub const COLLISION_CHUNK_ID: u32 = 33;
  pub const VELOCITY_SCALE_CHUNK_ID: u32 = 34;
  pub const DESCRIPTION_CHUNK_ID: u32 = 35;
  pub const EDITOR_DATA_CHUNK_ID: u32 = 36;
  pub const ROTATION_CHUNK_ID: u32 = 37;

  fn get_action_section(section_name: &str, index: usize) -> String {
    format!("{section_name}.action.{index}")
  }

  fn get_sprite_section(section_name: &str) -> String {
    format!("{section_name}.sprite")
  }

  fn get_frame_section(section_name: &str) -> String {
    format!("{section_name}.frame")
  }

  fn get_collision_section(section_name: &str) -> String {
    format!("{section_name}.collision")
  }

  fn get_description_section(section_name: &str) -> String {
    format!("{section_name}.description")
  }

  fn get_editor_data_section(section_name: &str) -> String {
    format!("{section_name}.editor_data")
  }
}

impl ChunkReadWrite for ParticleEffect {
  /// Read effects by position descriptor.
  /// Parses binary data into version chunk representation object.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let effect: Self = {
      Self {
        version: read_u16_chunk::<T, _>(
          &mut find_optional_chunk_by_id(&chunks, Self::VERSION_CHUNK_ID).expect("Particle name chunk not found"),
        )
        .map_err(|error| XrfError::new_parsing_error(format!("Failed to read particle version chunk: {}", error)))?,
        name: read_w1251_string_chunk(
          &mut find_optional_chunk_by_id(&chunks, Self::NAME_CHUNK_ID).expect("Particle name chunk not found"),
        )
        .map_err(|error| XrfError::new_parsing_error(format!("Failed to read particle name chunk: {}", error)))?,
        max_particles: read_u32_chunk::<T, _>(
          &mut find_optional_chunk_by_id(&chunks, Self::MAX_PARTICLES_CHUNK_ID)
            .expect("Particle max particles chunk not found"),
        )
        .map_err(|error| {
          XrfError::new_parsing_error(format!("Failed to read particle max_particles chunk: {}", error))
        })?,
        actions: find_required_chunk_by_id(&chunks, Self::ACTION_LIST_CHUNK_ID)?
          .read_xr_list::<T, _>()
          .map_err(|error| XrfError::new_parsing_error(format!("Failed to read particle actions chunk: {}", error)))?,
        flags: read_u32_chunk::<T, _>(
          &mut find_optional_chunk_by_id(&chunks, Self::FLAGS_CHUNK_ID).expect("Particle flags chunk not found"),
        )
        .map_err(|error| XrfError::new_parsing_error(format!("Failed to read particle flags chunk: {}", error)))?,
        frame: find_optional_chunk_by_id(&chunks, Self::FRAME_CHUNK_ID).map(|mut it| {
          it.read_xr::<T, _>()
            .expect("Invalid frame chunk data in particle effect")
        }),
        sprite: find_required_chunk_by_id(&chunks, Self::SPRITE_CHUNK_ID)?
          .read_xr::<T, _>()
          .map_err(|error| XrfError::new_parsing_error(format!("Failed to read particle sprite chunk: {}", error)))?,
        time_limit: find_optional_chunk_by_id(&chunks, Self::TIME_LIMIT_CHUNK_ID).map(|mut it| {
          read_f32_chunk::<T, _>(&mut it).expect("Invalid frame time limit chunk data in particle effect")
        }),
        collision: find_optional_chunk_by_id(&chunks, Self::COLLISION_CHUNK_ID).map(|mut it| {
          it.read_xr::<T, _>()
            .expect("Invalid collision chunk data in particle effect")
        }),
        velocity_scale: find_optional_chunk_by_id(&chunks, Self::VELOCITY_SCALE_CHUNK_ID).map(|mut it| {
          read_f32_vector_chunk::<T, _>(&mut it)
            .expect("Invalid velocity scale chunk data in particle effect")
            .into()
        }),
        description: find_optional_chunk_by_id(&chunks, Self::DESCRIPTION_CHUNK_ID).map(|mut it| {
          it.read_xr::<T, _>()
            .expect("Invalid description chunk data in particle effect")
        }),
        rotation: find_optional_chunk_by_id(&chunks, Self::ROTATION_CHUNK_ID).map(|mut it| {
          read_f32_vector_chunk::<T, _>(&mut it)
            .expect("Invalid rotation chunk data in particle effect")
            .into()
        }),
        editor_data: find_optional_chunk_by_id(&chunks, Self::EDITOR_DATA_CHUNK_ID).map(|mut it| {
          it.read_xr::<T, _>()
            .expect("Invalid editor data chunk in particle effect")
        }),
      }
    };

    reader.assert_read("Expect particle effect chunk to be ended")?;

    Ok(effect)
  }

  /// Write particle effect data into chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version_chunk_writer: ChunkWriter = ChunkWriter::new();
    version_chunk_writer.write_u16::<T>(self.version)?;
    version_chunk_writer.flush_chunk_into::<T>(writer, Self::VERSION_CHUNK_ID)?;

    let mut name_chunk_writer: ChunkWriter = ChunkWriter::new();
    name_chunk_writer.write_w1251_string(&self.name)?;
    name_chunk_writer.flush_chunk_into::<T>(writer, Self::NAME_CHUNK_ID)?;

    let mut max_particles_chunk_writer: ChunkWriter = ChunkWriter::new();
    max_particles_chunk_writer.write_u32::<T>(self.max_particles)?;
    max_particles_chunk_writer.flush_chunk_into::<T>(writer, Self::MAX_PARTICLES_CHUNK_ID)?;

    let mut actions_chunk_writer: ChunkWriter = ChunkWriter::new();
    actions_chunk_writer.write_xr_list::<T, _>(&self.actions)?;
    actions_chunk_writer.flush_chunk_into::<T>(writer, Self::ACTION_LIST_CHUNK_ID)?;

    let mut flags_chunk_writer: ChunkWriter = ChunkWriter::new();
    flags_chunk_writer.write_u32::<T>(self.flags)?;
    flags_chunk_writer.flush_chunk_into::<T>(writer, Self::FLAGS_CHUNK_ID)?;

    if let Some(frame) = &self.frame {
      let mut frame_chunk_writer: ChunkWriter = ChunkWriter::new();
      frame_chunk_writer.write_xr::<T, _>(frame)?;
      frame_chunk_writer.flush_chunk_into::<T>(writer, Self::FRAME_CHUNK_ID)?;
    }

    let mut sprite_chunk_writer: ChunkWriter = ChunkWriter::new();
    sprite_chunk_writer.write_xr::<T, _>(&self.sprite)?;
    sprite_chunk_writer.flush_chunk_into::<T>(writer, Self::SPRITE_CHUNK_ID)?;

    if let Some(time_limit) = self.time_limit {
      let mut time_limit_chunk_writer: ChunkWriter = ChunkWriter::new();
      time_limit_chunk_writer.write_f32::<T>(time_limit)?;
      time_limit_chunk_writer.flush_chunk_into::<T>(writer, Self::TIME_LIMIT_CHUNK_ID)?;
    }

    if let Some(collision) = &self.collision {
      let mut collision_chunk_writer: ChunkWriter = ChunkWriter::new();
      collision_chunk_writer.write_xr::<T, _>(collision)?;
      collision_chunk_writer.flush_chunk_into::<T>(writer, Self::COLLISION_CHUNK_ID)?;
    }

    if let Some(velocity_scale) = &self.velocity_scale {
      let mut velocity_scale_chunk_writer: ChunkWriter = ChunkWriter::new();
      velocity_scale_chunk_writer.write_xr::<T, Vector3d>(velocity_scale)?;
      velocity_scale_chunk_writer.flush_chunk_into::<T>(writer, Self::VELOCITY_SCALE_CHUNK_ID)?;
    }

    if let Some(description) = &self.description {
      let mut description_chunk_writer: ChunkWriter = ChunkWriter::new();
      description_chunk_writer.write_xr::<T, _>(description)?;
      description_chunk_writer.flush_chunk_into::<T>(writer, Self::DESCRIPTION_CHUNK_ID)?;
    }

    if let Some(rotation) = &self.rotation {
      let mut rotation_chunk_writer: ChunkWriter = ChunkWriter::new();
      rotation_chunk_writer.write_xr::<T, Vector3d>(rotation)?;
      rotation_chunk_writer.flush_chunk_into::<T>(writer, Self::ROTATION_CHUNK_ID)?;
    }

    if let Some(editor_data) = &self.editor_data {
      let mut editor_data_chunk_writer: ChunkWriter = ChunkWriter::new();
      editor_data_chunk_writer.write_xr::<T, _>(editor_data)?;
      editor_data_chunk_writer.flush_chunk_into::<T>(writer, Self::EDITOR_DATA_CHUNK_ID)?;
    }

    Ok(())
  }
}

impl LtxImportExport for ParticleEffect {
  /// Import particle effect data from provided path.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Particle effect section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let meta_type: String = read_ltx_field(META_TYPE_FIELD, section)?;

    assert_equal(
      meta_type.as_str(),
      Self::META_TYPE,
      "Expected corrected meta type field for particle effect importing",
    )?;

    let mut action_index: usize = 0;
    let mut actions: Vec<ParticleAction> = Vec::new();

    loop {
      let action_section_name: String = Self::get_action_section(section_name, action_index);

      if ltx.has_section(&action_section_name) {
        actions.push(ParticleAction::import(&action_section_name, ltx)?);
        action_index += 1
      } else {
        break;
      }

      if action_index >= Self::EFFECT_ACTIONS_LIMIT {
        return Err(XrfError::new_parsing_error(
          "Failed to parse particle effect - reached maximum nested actions limit",
        ));
      }
    }

    Ok(Self {
      version: read_ltx_field("version", section)?,
      name: read_ltx_field("name", section)?,
      max_particles: read_ltx_field("max_particles", section)?,
      actions,
      flags: read_ltx_field("flags", section)?,
      frame: ParticleEffectFrame::import_optional(&Self::get_frame_section(section_name), ltx)?,
      sprite: ParticleEffectSprite::import(&Self::get_sprite_section(section_name), ltx)?,
      time_limit: read_ltx_optional_field("time_limit", section)?,
      collision: ParticleEffectCollision::import_optional(&Self::get_collision_section(section_name), ltx)?,
      velocity_scale: read_ltx_optional_field("velocity_scale", section)?,
      description: ParticleDescription::import_optional(&Self::get_description_section(section_name), ltx)?,
      rotation: read_ltx_optional_field("rotation", section)?,
      editor_data: ParticleEffectEditorData::import_optional(&Self::get_editor_data_section(section_name), ltx)?,
    })
  }

  /// Export particle effect data into provided path.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx
      .with_section(section_name)
      .set(META_TYPE_FIELD, Self::META_TYPE)
      .set("version", self.version.to_string())
      .set("name", &self.name)
      .set("actions_count", self.actions.len().to_string())
      .set("max_particles", self.max_particles.to_string())
      .set("flags", self.flags.to_string())
      .set_optional("time_limit", self.time_limit.map(|it| it.to_string()))
      .set_optional("rotation", self.rotation.as_ref().map(|it| it.to_string()))
      .set_optional("velocity_scale", self.velocity_scale.as_ref().map(|it| it.to_string()));

    self.sprite.export(&Self::get_sprite_section(section_name), ltx)?;

    for (index, action) in self.actions.iter().enumerate() {
      action.export(&Self::get_action_section(section_name, index), ltx)?
    }

    ParticleEffectFrame::export_optional(&Self::get_frame_section(section_name), ltx, self.frame.as_ref())?;
    ParticleEffectCollision::export_optional(&Self::get_collision_section(section_name), ltx, self.collision.as_ref())?;
    ParticleDescription::export_optional(
      &Self::get_description_section(section_name),
      ltx,
      self.description.as_ref(),
    )?;
    ParticleEffectEditorData::export_optional(
      &Self::get_editor_data_section(section_name),
      ltx,
      self.editor_data.as_ref(),
    )?;

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
  use xrf_math::Vector3d;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::particles::actions::particle_action_copy_vertex::ParticleActionCopyVertex;
  use crate::data::particles::actions::particle_action_damping::ParticleActionDamping;
  use crate::data::particles::particle_action::ParticleAction;
  use crate::data::particles::particle_action_type::ParticleActionType;
  use crate::data::particles::particle_effect::ParticleEffect;
  use crate::data::particles::particle_effect_collision::ParticleEffectCollision;
  use crate::data::particles::particle_effect_description::ParticleDescription;
  use crate::data::particles::particle_effect_editor_data::ParticleEffectEditorData;
  use crate::data::particles::particle_effect_frame::ParticleEffectFrame;
  use crate::data::particles::particle_effect_sprite::ParticleEffectSprite;
  use crate::export::LtxImportExport;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: ParticleEffect = ParticleEffect {
      version: 1,
      name: String::from("test-particle-effect"),
      max_particles: 5,
      actions: vec![
        ParticleAction::Damping(Box::new(ParticleActionDamping {
          action_flags: 31,
          action_type: ParticleActionType::Damping,
          damping: Vector3d::new_mock(),
          v_low_sqr: 1.1,
          v_high_sqr: 1.25,
        })),
        ParticleAction::CopyVertex(Box::new(ParticleActionCopyVertex {
          action_flags: 453,
          action_type: ParticleActionType::CopyVertex,
          copy_position: 1,
        })),
      ],
      flags: 140,
      frame: Some(ParticleEffectFrame {
        texture_size: (450.0, 360.0),
        reserved: (45.2, 51.2),
        frame_dimension_x: 320,
        frame_count: 60,
        frame_speed: 29.7,
      }),
      sprite: ParticleEffectSprite {
        shader_name: String::from("test-shader-name"),
        texture_name: String::from("test-texture-name"),
      },
      time_limit: Some(450.1),
      collision: Some(ParticleEffectCollision {
        collide_one_minus_friction: 0.55,
        collide_resilience: 45.2535,
        collide_sqr_cutoff: 25.6313,
      }),
      velocity_scale: Some(Vector3d::new_mock()),
      description: Some(ParticleDescription {
        creator: String::from("test-creator-name"),
        editor: String::from("test-editor-name"),
        created_time: 425,
        edit_time: 450,
      }),
      rotation: Some(Vector3d::new_mock()),
      editor_data: Some(ParticleEffectEditorData {
        value: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
      }),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 343);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 343);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 343 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    let read: ParticleEffect = ParticleEffect::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(read, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();
    let original: ParticleEffect = ParticleEffect {
      version: 1,
      name: String::from("test-particle-effect"),
      max_particles: 6,
      actions: vec![
        ParticleAction::Damping(Box::new(ParticleActionDamping {
          action_flags: 33,
          action_type: ParticleActionType::Damping,
          damping: Vector3d::new_mock(),
          v_low_sqr: 1.1,
          v_high_sqr: 1.25,
        })),
        ParticleAction::CopyVertex(Box::new(ParticleActionCopyVertex {
          action_flags: 34,
          action_type: ParticleActionType::CopyVertex,
          copy_position: 1,
        })),
      ],
      flags: 140,
      frame: Some(ParticleEffectFrame {
        texture_size: (451.0, 361.0),
        reserved: (45.2, 51.2),
        frame_dimension_x: 320,
        frame_count: 60,
        frame_speed: 29.7,
      }),
      sprite: ParticleEffectSprite {
        shader_name: String::from("test-shader-name"),
        texture_name: String::from("test-texture-name"),
      },
      time_limit: Some(450.1),
      collision: Some(ParticleEffectCollision {
        collide_one_minus_friction: 0.55,
        collide_resilience: 45.2535,
        collide_sqr_cutoff: 25.6313,
      }),
      velocity_scale: Some(Vector3d::new_mock()),
      description: Some(ParticleDescription {
        creator: String::from("test-creator-name"),
        editor: String::from("test-editor-name"),
        created_time: 456,
        edit_time: 458,
      }),
      rotation: Some(Vector3d::new_mock()),
      editor_data: Some(ParticleEffectEditorData {
        value: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
      }),
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(ParticleEffect::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: ParticleEffect = ParticleEffect {
      version: 1,
      name: String::from("test-particle-effect"),
      max_particles: 5,
      actions: vec![
        ParticleAction::Damping(Box::new(ParticleActionDamping {
          action_flags: 31,
          action_type: ParticleActionType::CopyVertex,
          damping: Vector3d::new_mock(),
          v_low_sqr: 5.2,
          v_high_sqr: 6.5,
        })),
        ParticleAction::CopyVertex(Box::new(ParticleActionCopyVertex {
          action_flags: 453,
          action_type: ParticleActionType::CopyVertex,
          copy_position: 0,
        })),
      ],
      flags: 150,
      frame: Some(ParticleEffectFrame {
        texture_size: (460.0, 380.0),
        reserved: (41.2, 42.2),
        frame_dimension_x: 640,
        frame_count: 80,
        frame_speed: 29.7,
      }),
      sprite: ParticleEffectSprite {
        shader_name: String::from("test-shader-name"),
        texture_name: String::from("test-texture-name"),
      },
      time_limit: Some(460.1),
      collision: Some(ParticleEffectCollision {
        collide_one_minus_friction: 0.540,
        collide_resilience: 455.2535,
        collide_sqr_cutoff: 255.6313,
      }),
      velocity_scale: Some(Vector3d::new_mock()),
      description: Some(ParticleDescription {
        creator: String::from("test-creator-name"),
        editor: String::from("test-editor-name"),
        created_time: 433,
        edit_time: 444,
      }),
      rotation: Some(Vector3d::new_mock()),
      editor_data: Some(ParticleEffectEditorData {
        value: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
      }),
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(serde_json::from_str::<ParticleEffect>(&serialized)?, original);

    Ok(())
  }
}
