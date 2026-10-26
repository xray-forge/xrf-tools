use byteorder::{ByteOrder, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, META_TYPE_FIELD, Section};
use xrf_math::Vector3d;
use xrf_utils::assert_equal;

use crate::data::editor::particle_editor_action::ParticleEditorAction;
use crate::data::effect::particle_effect_collision::ParticleEffectCollision;
use crate::data::effect::particle_effect_editor_data::ParticleEffectEditorData;
use crate::data::effect::particle_effect_frame::ParticleEffectFrame;
use crate::data::effect::particle_effect_sprite::ParticleEffectSprite;
use crate::data::particle_action::ParticleAction;
use crate::data::particle_description::ParticleDescription;
use crate::data::particle_record_chunks::ParticleRecordChunks;

/// One emitter, the engine's `CPEDef` (`Layers/xrRender/ParticleEffectDef.cpp`).
///
/// The optional records are chunks the engine reads only where `flags` asks for them (`dfSprite`, `dfFramed`,
/// `dfTimeLimit`, `dfCollision`, `dfVelocityScale`, `dfAlignToPath`); every shipped library stores exactly those.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEffect {
  pub version: u16,
  pub name: String,
  pub max_particles: u32,
  pub actions: Vec<ParticleAction>,
  pub flags: u32,
  pub frame: Option<ParticleEffectFrame>,
  /// What the effect draws; absent on an effect that draws nothing, which two of GAMMA's are.
  pub sprite: Option<ParticleEffectSprite>,
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

  /// `PED_VERSION`, the one version `CPEDef::Load` reads; any other fails the whole library's load.
  pub const VERSION: u16 = 1;

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

  /// The actions as the SDK edits them, where the effect keeps its editor data; every shipped effect does.
  ///
  /// # Errors
  ///
  /// Returns an error when the editor data does not decode.
  pub fn read_editor_actions<T: ByteOrder>(&self) -> XrfResult<Option<Vec<ParticleEditorAction>>> {
    self
      .editor_data
      .as_ref()
      .map(ParticleEffectEditorData::read_actions::<T>)
      .transpose()
  }

  /// Replaces the editor's actions, and the engine's with what they compile to, as the SDK saves an effect: the enabled
  /// ones, in their order.
  ///
  /// # Errors
  ///
  /// Returns an error when an enabled action does not compile; the effect is then left as it was.
  pub fn set_editor_actions<T: ByteOrder>(&mut self, actions: &[ParticleEditorAction]) -> XrfResult {
    let compiled: Vec<ParticleAction> = actions
      .iter()
      .filter(|action| action.is_enabled())
      .map(ParticleEditorAction::compile)
      .collect::<XrfResult<_>>()?;

    self.editor_data = Some(ParticleEffectEditorData::from_actions::<T>(actions)?);
    self.actions = compiled;

    Ok(())
  }
}

impl ChunkReadWrite for ParticleEffect {
  /// Reads an effect as `CPEDef::Load` does, from the chunks it is made of in any order.
  ///
  /// # Errors
  ///
  /// Returns a parsing error naming the chunk when a required one is missing or any is malformed, and when the version
  /// is not [`Self::VERSION`].
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    let version: u16 = ParticleRecordChunks::read_required(&chunks, Self::VERSION_CHUNK_ID, "version", |it| {
      it.read_u16_payload::<T>()
    })?;

    if version != Self::VERSION {
      return Err(XrfError::new_parsing_error(format!(
        "Failed to read particle effect of version {version}, only version {} is read by the engine",
        Self::VERSION
      )));
    }

    let effect: Self = Self {
      version,
      name: ParticleRecordChunks::read_required(&chunks, Self::NAME_CHUNK_ID, "name", |it| {
        it.read_w1251_string_payload()
      })?,
      max_particles: ParticleRecordChunks::read_required(
        &chunks,
        Self::MAX_PARTICLES_CHUNK_ID,
        "max_particles",
        |it| it.read_u32_payload::<T>(),
      )?,
      actions: ParticleRecordChunks::read_required(&chunks, Self::ACTION_LIST_CHUNK_ID, "actions", |it| {
        it.read_xr_list::<T, _>()
      })?,
      flags: ParticleRecordChunks::read_required(&chunks, Self::FLAGS_CHUNK_ID, "flags", |it| {
        it.read_u32_payload::<T>()
      })?,
      frame: ParticleRecordChunks::read_optional(&chunks, Self::FRAME_CHUNK_ID, "frame", |it| it.read_xr::<T, _>())?,
      sprite: ParticleRecordChunks::read_optional(&chunks, Self::SPRITE_CHUNK_ID, "sprite", |it| it.read_xr::<T, _>())?,
      time_limit: ParticleRecordChunks::read_optional(&chunks, Self::TIME_LIMIT_CHUNK_ID, "time_limit", |it| {
        it.read_f32_payload::<T>()
      })?,
      collision: ParticleRecordChunks::read_optional(&chunks, Self::COLLISION_CHUNK_ID, "collision", |it| {
        it.read_xr::<T, _>()
      })?,
      velocity_scale: ParticleRecordChunks::read_optional(
        &chunks,
        Self::VELOCITY_SCALE_CHUNK_ID,
        "velocity_scale",
        |it| it.read_f32_vector_payload::<T>().map(Into::into),
      )?,
      description: ParticleRecordChunks::read_optional(&chunks, Self::DESCRIPTION_CHUNK_ID, "description", |it| {
        it.read_xr::<T, _>()
      })?,
      rotation: ParticleRecordChunks::read_optional(&chunks, Self::ROTATION_CHUNK_ID, "rotation", |it| {
        it.read_f32_vector_payload::<T>().map(Into::into)
      })?,
      editor_data: ParticleRecordChunks::read_optional(&chunks, Self::EDITOR_DATA_CHUNK_ID, "editor_data", |it| {
        it.read_xr::<T, _>()
      })?,
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

    if let Some(sprite) = &self.sprite {
      let mut sprite_chunk_writer: ChunkWriter = ChunkWriter::new();
      sprite_chunk_writer.write_xr::<T, _>(sprite)?;
      sprite_chunk_writer.flush_chunk_into::<T>(writer, Self::SPRITE_CHUNK_ID)?;
    }

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

    let meta_type: String = section.read_field(META_TYPE_FIELD)?;

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
      version: section.read_field("version")?,
      name: section.read_field("name")?,
      max_particles: section.read_field("max_particles")?,
      actions,
      flags: section.read_field("flags")?,
      frame: ParticleEffectFrame::import_optional(&Self::get_frame_section(section_name), ltx)?,
      sprite: ParticleEffectSprite::import_optional(&Self::get_sprite_section(section_name), ltx)?,
      time_limit: section.read_optional_field("time_limit")?,
      collision: ParticleEffectCollision::import_optional(&Self::get_collision_section(section_name), ltx)?,
      velocity_scale: section.read_optional_field("velocity_scale")?,
      description: ParticleDescription::import_optional(&Self::get_description_section(section_name), ltx)?,
      rotation: section.read_optional_field("rotation")?,
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

    ParticleEffectSprite::export_optional(&Self::get_sprite_section(section_name), ltx, self.sprite.as_ref())?;

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
  use std::io::{Seek, SeekFrom, Write, read_to_string};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::data::actions::particle_action_copy_vertex::ParticleActionCopyVertex;
  use crate::data::actions::particle_action_damping::ParticleActionDamping;
  use crate::data::effect::particle_effect::ParticleEffect;
  use crate::data::effect::particle_effect_collision::ParticleEffectCollision;
  use crate::data::effect::particle_effect_editor_data::ParticleEffectEditorData;
  use crate::data::effect::particle_effect_frame::ParticleEffectFrame;
  use crate::data::effect::particle_effect_sprite::ParticleEffectSprite;
  use crate::data::particle_action::ParticleAction;
  use crate::data::particle_action_type::ParticleActionType;
  use crate::data::particle_description::ParticleDescription;

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
      sprite: Some(ParticleEffectSprite {
        shader_name: String::from("test-shader-name"),
        texture_name: String::from("test-texture-name"),
      }),
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
      &mut GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 343);

    let file: FileSlice = GeneratedResources::open_slice(&GeneratedResources::sample_name(file!(), &filename))?;

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
    let ltx_filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
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
      sprite: Some(ParticleEffectSprite {
        shader_name: String::from("test-shader-name"),
        texture_name: String::from("test-texture-name"),
      }),
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

    ltx.write_to(&mut GeneratedResources::overwrite(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&ltx_filename))?;

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
      sprite: Some(ParticleEffectSprite {
        shader_name: String::from("test-shader-name"),
        texture_name: String::from("test-texture-name"),
      }),
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

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(serde_json::from_str::<ParticleEffect>(&serialized)?, original);

    Ok(())
  }
}
