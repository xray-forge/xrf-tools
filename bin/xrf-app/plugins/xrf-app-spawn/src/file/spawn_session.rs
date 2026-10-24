use std::path::PathBuf;

use xrf_app_core::TauriResult;
use xrf_chunk::XRayByteOrder;
use xrf_spawn::SpawnFile;

use crate::{SpawnChunk, SpawnChunkKind, SpawnSessionDescriptor};

/// One opened spawn, as read: the file and what describes it, decided together.
pub struct SpawnSession {
  pub descriptor: SpawnSessionDescriptor,
  pub file: SpawnFile,
}

impl SpawnSession {
  /// The spawn `file`, read from `path`.
  pub fn new(path: PathBuf, file: SpawnFile) -> Self {
    Self {
      descriptor: SpawnSessionDescriptor {
        path,
        header: file.header.clone(),
      },
      file,
    }
  }

  /// Reads the packed spawn at `path`.
  ///
  /// # Errors
  ///
  /// Returns an error naming the path where the file cannot be read as a spawn.
  pub fn read(path: PathBuf) -> TauriResult<Self> {
    let file: SpawnFile = SpawnFile::read_from_path::<XRayByteOrder, _>(&path)
      .map_err(|error| format!("Cannot open spawn '{}': {error}", path.display()))?;

    Ok(Self::new(path, file))
  }

  /// What describes the spawn, without its chunks.
  pub fn describe(&self) -> SpawnSessionDescriptor {
    self.descriptor.clone()
  }

  /// A copy of one chunk, for the page to show.
  pub fn read_chunk(&self, kind: SpawnChunkKind) -> SpawnChunk {
    match kind {
      SpawnChunkKind::AlifeSpawn => SpawnChunk::AlifeSpawn(self.file.alife_spawn.clone()),
      SpawnChunkKind::ArtefactSpawn => SpawnChunk::ArtefactSpawn(self.file.artefact_spawn.clone()),
      SpawnChunkKind::Patrols => SpawnChunk::Patrols(self.file.patrols.clone()),
      SpawnChunkKind::Graphs => SpawnChunk::Graphs(self.file.graphs.clone()),
    }
  }
}
