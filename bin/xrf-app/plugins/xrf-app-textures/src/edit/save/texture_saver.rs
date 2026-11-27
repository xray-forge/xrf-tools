use std::path::{Path, PathBuf};

use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, FileStamp};
use xrf_job::{JobHandle, JobOutcome};
use xrf_thm::{ThmFile, ThmFormat};
use xrf_utils::{StagedFile, error_to_string, format_path};

use crate::edit::{TextureDescriptorSave, TextureSaveOutcome, TextureWrite};

/// One file the save has built and not yet published.
struct PreparedFile {
  path: PathBuf,
  bytes: Vec<u8>,
}

/// Publishing what the editor changed: the descriptor, and the base texture with it when one was chosen.
pub(crate) struct TextureSaver;

impl TextureSaver {
  /// Writes a node's descriptor and, when given, its texture, each staged beside its target and renamed over it,
  /// through `assets` where a mount holds it. A written texture makes the descriptor name its format, where the
  /// descriptor can name it at all.
  ///
  /// # Errors
  ///
  /// Returns an error when a target changed since it was read, when the descriptor cannot be built or serialized, or
  /// when a staged write cannot be moved into place.
  pub(crate) fn save(
    assets: &AssetMountState,
    job: &JobHandle,
    descriptor: &TextureDescriptorSave,
    texture: Option<TextureWrite>,
  ) -> TauriResult<TextureSaveOutcome> {
    let mut prepared: Vec<PreparedFile> = Vec::with_capacity(2);
    let mut synced_format: Option<ThmFormat> = None;

    if let Some(texture) = texture {
      FileStamp::check_unchanged(&texture.path, texture.expected)?;

      synced_format = texture.format.to_thm_format(descriptor.form.has_alpha());
      prepared.push(PreparedFile {
        path: texture.path,
        bytes: texture.bytes,
      });
    }

    let path: PathBuf = PathBuf::from(&descriptor.target.path);

    FileStamp::check_unchanged(&path, descriptor.target.expected)?;

    prepared.push(PreparedFile {
      bytes: Self::encode_descriptor(descriptor, &path, synced_format)?,
      path,
    });

    if job.is_cancelled() {
      return Ok(TextureSaveOutcome {
        outcome: JobOutcome::Cancelled,
        written: Vec::new(),
      });
    }

    Ok(TextureSaveOutcome {
      outcome: JobOutcome::Completed,
      written: Self::publish(assets, prepared)?,
    })
  }

  /// The descriptor as bytes, built onto whatever is on disk so every chunk the form does not model survives.
  fn encode_descriptor(
    save: &TextureDescriptorSave,
    path: &Path,
    synced_format: Option<ThmFormat>,
  ) -> TauriResult<Vec<u8>> {
    let existing: Option<ThmFile> = match save.target.expected {
      Some(_) => Some(ThmFile::read_from_path(&path).map_err(error_to_string)?),
      None => None,
    };

    save.form.to_bytes(existing, synced_format)
  }

  /// Stages every prepared file beside its target and publishes them through `assets`, answering what landed.
  fn publish(assets: &AssetMountState, prepared: Vec<PreparedFile>) -> TauriResult<Vec<String>> {
    let staged: Vec<StagedFile> = prepared
      .into_iter()
      .map(|file| {
        StagedFile::name_sibling(&file.path, "staged")
          .and_then(|staged| StagedFile::stage(&file.path, &staged, &file.bytes))
          .map_err(|error| format!("Cannot write '{}': {error}", format_path(&file.path)))
      })
      .collect::<TauriResult<_>>()?;

    Ok(
      assets
        .write_files(staged, None)?
        .iter()
        .map(|path| path.display().to_string())
        .collect(),
    )
  }
}
