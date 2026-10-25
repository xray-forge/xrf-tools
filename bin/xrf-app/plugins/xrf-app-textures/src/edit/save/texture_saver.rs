use std::path::{Path, PathBuf};

use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, FileStamp};
use xrf_chunk::XRayByteOrder;
use xrf_job::{JobHandle, JobOutcome};
use xrf_thm::{ThmFile, ThmFormat};
use xrf_utils::{StagedFile, error_to_string, format_path};
use xrf_vfs::{XraySearch, XrayTransaction};

use crate::edit::{TextureDescriptorForm, TextureDescriptorSave, TextureSaveOutcome, TextureWrite};

/// One file the save has built and not yet published.
struct PreparedFile {
  path: PathBuf,
  bytes: Vec<u8>,
}

/// Publishing what the editor changed: the descriptor, and the base texture with it when one was chosen.
pub struct TextureSaver;

impl TextureSaver {
  /// Writes a node's descriptor and, when given, its texture, each staged beside its target and renamed over it,
  /// through `assets` where a mount holds it. A written texture makes the descriptor name its format, where the
  /// descriptor can name it at all.
  ///
  /// # Errors
  ///
  /// Returns an error when a target changed since it was read, when the descriptor cannot be built or serialized, or
  /// when a staged write cannot be moved into place.
  pub fn save(
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
      bytes: Self::to_descriptor_bytes(descriptor, &path, synced_format)?,
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
  fn to_descriptor_bytes(
    save: &TextureDescriptorSave,
    path: &Path,
    synced_format: Option<ThmFormat>,
  ) -> TauriResult<Vec<u8>> {
    let existing: Option<ThmFile> = match save.target.expected {
      Some(_) => Some(ThmFile::read_from_path::<XRayByteOrder, _>(&path).map_err(error_to_string)?),
      None => None,
    };
    let mut form: TextureDescriptorForm = save.form.clone();

    if let Some(format) = synced_format {
      form.format = format.into();
    }

    form
      .to_descriptor(existing)
      .write_to_bytes::<XRayByteOrder>()
      .map_err(error_to_string)
  }

  /// Moves every prepared file into place, answering what landed: the files a mount holds as one commit, so the cache,
  /// sessions and pages follow them as any save, then the rest one by one, since no snapshot reads a file outside every
  /// mount.
  fn publish(assets: &AssetMountState, prepared: Vec<PreparedFile>) -> TauriResult<Vec<String>> {
    let written: Vec<String> = prepared.iter().map(|file| file.path.display().to_string()).collect();
    let search: XraySearch = assets.search_writable();
    let (mounted, loose): (Vec<PreparedFile>, Vec<PreparedFile>) =
      prepared.into_iter().partition(|file| search.holds_file(&file.path));

    if !mounted.is_empty() {
      assets.commit(
        &mounted
          .into_iter()
          .fold(XrayTransaction::new(&search), |transaction, file| {
            transaction.with_file_write(file.path, file.bytes)
          }),
      )?;
    }

    for file in loose {
      StagedFile::write(&file.path, &file.bytes)
        .map_err(|error| format!("Cannot write '{}': {error}", format_path(&file.path)))?;
    }

    Ok(written)
  }
}
