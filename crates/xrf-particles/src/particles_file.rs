use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::FileImportExport;
use xrf_utils::{StagedFile, assert, format_path};

use crate::chunks::particles_effects_chunk::ParticlesEffectsChunk;
use crate::chunks::particles_firstgen_chunk::ParticlesFirstgenChunk;
use crate::chunks::particles_groups_chunk::ParticlesGroupsChunk;
use crate::chunks::particles_header_chunk::ParticlesHeaderChunk;
use crate::data::effect::particle_effect::ParticleEffect;
use crate::data::group::particle_group::ParticleGroup;
use crate::data::particle_action::ParticleAction;
use crate::data::particle_action_type::ParticleActionType;
use crate::particles_file_issue::ParticlesFileIssue;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticlesFile {
  pub header: ParticlesHeaderChunk,
  pub effects: ParticlesEffectsChunk,
  pub groups: ParticlesGroupsChunk,
}

impl ParticlesFile {
  /// `PSLIB_FILENAME`, the library's path under `$game_data$`, the one `CPSLibrary::OnCreate` loads.
  pub const FILE_NAME: &'static str = "particles.xr";

  /// The root chunks the engine reads, which are the ones a library is written with.
  pub const ROOT_CHUNK_IDS: [u32; 3] = [
    ParticlesHeaderChunk::CHUNK_ID,
    ParticlesEffectsChunk::CHUNK_ID,
    ParticlesGroupsChunk::CHUNK_ID,
  ];

  /// Read particles from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path)?)
  }

  /// Read particles from file.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from a chunk reader over any data source.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_from_chunks::<T, _>(&chunks)
  }

  /// Reads a library from its root chunks, as `CPSLibrary::Load` does: the header is required, and a library without
  /// effects or without groups holds none.
  ///
  /// # Errors
  ///
  /// Returns an error when the header is missing or is not version 1, when a root chunk is one the engine does not
  /// read, which a rewrite would drop, or when a first-generation chunk is present, which this crate does not read.
  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    assert(
      !chunks.iter().any(|it| it.id == ParticlesFirstgenChunk::CHUNK_ID),
      "Unexpected first-gen chunk in particles file, unpacking not implemented",
    )?;

    if let Some(unknown) = chunks.iter().find(|it| !Self::ROOT_CHUNK_IDS.contains(&it.id)) {
      return Err(XrfError::new_parsing_error(format!(
        "Unexpected chunk {} in particles file root, only {:?} are read",
        unknown.id,
        Self::ROOT_CHUNK_IDS
      )));
    }

    Ok(Self {
      header: ChunkReader::find_required_by_id(chunks, ParticlesHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      effects: ChunkReader::find_optional_by_id(chunks, ParticlesEffectsChunk::CHUNK_ID)
        .map(|mut it| it.read_xr::<T, _>())
        .transpose()?
        .unwrap_or_default(),
      groups: ChunkReader::find_optional_by_id(chunks, ParticlesGroupsChunk::CHUNK_ID)
        .map(|mut it| it.read_xr::<T, _>())
        .transpose()?
        .unwrap_or_default(),
    })
  }

  /// Write particles file data to the file by provided path.
  pub fn write_to_path<T: ByteOrder, P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    let mut bytes: Vec<u8> = Vec::new();

    self.write_to::<T>(&mut bytes)?;

    Ok(StagedFile::write_creating_directories(path.as_ref(), &bytes)?)
  }

  /// Write particles file data to the writer.
  pub fn write_to<T: ByteOrder>(&self, writer: &mut dyn Write) -> XrfResult {
    log::info!(
      "Writing particles file: version {}, {} effects, {} groups",
      self.header.version,
      self.effects.effects.len(),
      self.groups.groups.len(),
    );

    let mut header_chunk_writer: ChunkWriter = ChunkWriter::new();
    header_chunk_writer.write_xr::<T, _>(&self.header)?;
    header_chunk_writer.flush_chunk_into::<T>(writer, ParticlesHeaderChunk::CHUNK_ID)?;

    let mut effects_chunk_writer: ChunkWriter = ChunkWriter::new();
    effects_chunk_writer.write_xr::<T, _>(&self.effects)?;
    effects_chunk_writer.flush_chunk_into::<T>(writer, ParticlesEffectsChunk::CHUNK_ID)?;

    let mut group_chunk_writer: ChunkWriter = ChunkWriter::new();
    group_chunk_writer.write_xr::<T, _>(&self.groups)?;
    group_chunk_writer.flush_chunk_into::<T>(writer, ParticlesGroupsChunk::CHUNK_ID)?;

    Ok(())
  }

  /// Whether an effect or a group has the name, which the SDK keeps unique across both (`CPSLibrary::FindByName`).
  pub fn contains_name(&self, name: &str) -> bool {
    self.effects.get(name).is_some() || self.groups.get(name).is_some()
  }

  /// Adds an effect, as the SDK appends one.
  ///
  /// # Errors
  ///
  /// Returns an error when an effect or a group already has its name.
  pub fn insert_effect(&mut self, effect: ParticleEffect) -> XrfResult {
    self.assert_name_free(&effect.name)?;
    self.effects.insert(effect)
  }

  /// Adds a group, as the SDK appends one.
  ///
  /// # Errors
  ///
  /// Returns an error when an effect or a group already has its name.
  pub fn insert_group(&mut self, group: ParticleGroup) -> XrfResult {
    self.assert_name_free(&group.name)?;
    self.groups.insert(group)
  }

  /// Renames an effect and every group reference to it, the played effect and the children alike, as the SDK's
  /// effect editor does.
  ///
  /// # Errors
  ///
  /// Returns an error when no effect has the name `from`, or an effect or a group already has `to`.
  pub fn rename_effect(&mut self, from: &str, to: &str) -> XrfResult {
    self.assert_name_free(to)?;

    let mut effect: ParticleEffect = self
      .effects
      .remove(from)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Particle effect '{from}' is not defined")))?;

    effect.name = String::from(to);
    self.effects.insert(effect)?;

    for group_effect in self.groups.groups.iter_mut().flat_map(|group| group.effects.iter_mut()) {
      group_effect.rename_effect(from, to);
    }

    Ok(())
  }

  /// Renames a group; nothing in the library refers to a group by name.
  ///
  /// # Errors
  ///
  /// Returns an error when no group has the name `from`, or an effect or a group already has `to`.
  pub fn rename_group(&mut self, from: &str, to: &str) -> XrfResult {
    self.assert_name_free(to)?;

    let mut group: ParticleGroup = self
      .groups
      .remove(from)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Particle group '{from}' is not defined")))?;

    group.name = String::from(to);
    self.groups.insert(group)
  }

  /// Every fault the SDK's particle editor would refuse to save over, in library order: effects without a `KillOld`
  /// action, then groups playing undefined effects or spawning unnamed children.
  ///
  /// An effect is judged by its editor actions where it keeps them, as the SDK does, and by its compiled ones otherwise.
  pub fn validate<T: ByteOrder>(&self) -> Vec<ParticlesFileIssue> {
    let mut issues: Vec<ParticlesFileIssue> = Vec::new();

    for effect in &self.effects.effects {
      let has_kill_old: bool = match effect.read_editor_actions::<T>() {
        Ok(Some(actions)) => actions
          .iter()
          .any(|action| action.action_type == ParticleActionType::KillOld),
        Ok(None) => effect
          .actions
          .iter()
          .any(|action| matches!(action, ParticleAction::KillOld(_))),
        Err(error) => {
          issues.push(ParticlesFileIssue::UnreadableEditorData {
            effect: effect.name.clone(),
            message: error.to_string(),
          });
          continue;
        }
      };

      if !has_kill_old {
        issues.push(ParticlesFileIssue::MissingKillOld {
          effect: effect.name.clone(),
        });
      }
    }

    for group in &self.groups.groups {
      for group_effect in &group.effects {
        if self.effects.get(&group_effect.name).is_none() {
          issues.push(ParticlesFileIssue::UndefinedEffect {
            group: group.name.clone(),
            effect: group_effect.name.clone(),
          });
        }

        issues.extend(
          group_effect
            .list_unnamed_children()
            .map(|child| ParticlesFileIssue::UnnamedChild {
              group: group.name.clone(),
              effect: group_effect.name.clone(),
              child,
            }),
        );
      }
    }

    issues
  }

  fn assert_name_free(&self, name: &str) -> XrfResult {
    if self.contains_name(name) {
      return Err(XrfError::new_invalid_error(format!(
        "Particle effect or group '{name}' is already defined"
      )));
    }

    Ok(())
  }

  /// Read spawn file from provided path.
  pub fn import_from_path<P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    log::info!("Importing particles file: {}", format_path(path.as_ref()));

    Ok(Self {
      header: ParticlesHeaderChunk::import(path)?,
      effects: ParticlesEffectsChunk::import(path)?,
      groups: ParticlesGroupsChunk::import(path)?,
    })
  }

  /// Export unpacked ALife spawn file into provided path.
  pub fn export_to_path<P: AsRef<Path>>(&self, path: &P) -> XrfResult {
    fs::create_dir_all(path)?;

    self.header.export(path)?;
    self.effects.export(path)?;
    self.groups.export(path)?;

    Ok(())
  }
}
