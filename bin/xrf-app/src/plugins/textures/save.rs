//! Publishing what the editor changed: the descriptor, the base texture, or both.

use std::path::{Path, PathBuf};

use serde::Serialize;
use xrf_db::{ThmFile, ThmFormat, ThmTextureFlag, ThmTextureFlags, XRayByteOrder};
use xrf_job::{JobHandle, JobOutcome};
use xrf_utils::{format_path, write_file_staged};

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::textures::descriptor_form::TextureDescriptorForm;
use crate::plugins::textures::encoding::TextureEncodingFormat;
use crate::plugins::textures::file_stamp::TextureFileStamp;
use crate::plugins::textures::request::{TextureDescriptorSave, TexturesSaveRequest};

/// What a save left on disk.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureSaveOutcome {
  /// Whether the save wrote what it was asked to or stopped because it was asked to.
  ///
  /// A cancelled save wrote nothing. Cancellation is read once, after both files have been prepared and before either
  /// is written, because there is no useful boundary inside two staged writes: stopping between them would leave a
  /// texture whose descriptor still describes the old one, which is the state a save exists to avoid.
  pub outcome: JobOutcome,
  /// The files written, in the order they were written.
  pub written: Vec<String>,
  /// The format the descriptor ended up naming, when writing a texture changed it.
  ///
  /// Reported rather than left to the caller to infer, because the rule is the SDK's: `tfDXT1` and `tfADXT1` are one
  /// encoder distinguished by the alpha flag, so only the descriptor's own flags can say which of them a BC1 texture
  /// is. Absent when nothing synced - no texture was written, or its format has no name in `ETFormat`.
  pub descriptor_format: Option<u32>,
}

/// One file the save has built and not yet published.
struct PreparedFile {
  path: PathBuf,
  bytes: Vec<u8>,
}

/// Write both halves of a node, each staged beside its target and renamed over it.
///
/// Two phases, and the split is the whole design. Everything before the cancellation check reads and builds; everything
/// after only publishes. That is what lets a stopped save write nothing at all, and it is why there is no boundary
/// inside the publishing: stopping between the two files would leave a texture whose descriptor still describes the one
/// it replaced, which is the state a save exists to avoid.
///
/// Within the publishing, order still matters and is the safe one: the texture first, then the descriptor. A descriptor
/// names a width, a height and a format that describe the file beside it, so if a disk failure does split the pair, the
/// half left standing should be the texture rather than a descriptor describing bytes that were never written.
///
/// # Errors
///
/// Returns an error when a target changed since it was read, when the descriptor cannot be built or serialized, or when
/// a staged write cannot be moved into place.
pub fn write_save(
  job: &JobHandle,
  request: &TexturesSaveRequest,
  texture_bytes: Option<Vec<u8>>,
) -> TauriResult<TextureSaveOutcome> {
  let mut prepared: Vec<PreparedFile> = Vec::with_capacity(2);
  let mut descriptor_format: Option<ThmFormat> = None;

  if let Some(save) = &request.texture {
    let path: PathBuf = PathBuf::from(&save.target.path);

    TextureFileStamp::check_unchanged(&path, save.target.expected)?;

    prepared.push(PreparedFile {
      path,
      bytes: texture_bytes
        .ok_or_else(|| String::from("No encoded texture is held for this node; compare the formats again"))?,
    });

    descriptor_format = to_descriptor_format(save.format, request.descriptor.as_ref());
  }

  if let Some(save) = &request.descriptor {
    let path: PathBuf = PathBuf::from(&save.target.path);

    TextureFileStamp::check_unchanged(&path, save.target.expected)?;

    prepared.push(PreparedFile {
      bytes: to_descriptor_bytes(save, &path, descriptor_format)?,
      path,
    });
  }

  if job.is_cancelled() {
    return Ok(TextureSaveOutcome {
      outcome: JobOutcome::Cancelled,
      written: Vec::new(),
      descriptor_format: None,
    });
  }

  Ok(TextureSaveOutcome {
    outcome: JobOutcome::Completed,
    written: publish(prepared)?,
    descriptor_format: descriptor_format.map(u32::from),
  })
}

/// The descriptor as bytes, built onto whatever is on disk so every chunk the form does not model survives.
///
/// A target the editor read is re-read here rather than rebuilt from the request alone: the form carries the couple of
/// dozen values a person edits, and the thumbnail and any chunk this reader could not fold have to come from the file.
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

/// Move every prepared file into place, answering what landed.
fn publish(prepared: Vec<PreparedFile>) -> TauriResult<Vec<String>> {
  prepared
    .into_iter()
    .map(|file| {
      write_file_staged(&file.path, &file.bytes)
        .map_err(|error| format!("Cannot write '{}': {error}", format_path(&file.path)))?;

      Ok(file.path.display().to_string())
    })
    .collect()
}

/// The `ETFormat` a written candidate makes the descriptor name, when the descriptor can name it at all.
///
/// BC7 has no member of `ETFormat`, so a descriptor keeps whatever it said and the panel states that an SDK rebuild
/// would return the texture to the named format. BC1 splits on the descriptor's own alpha flag, exactly as the SDK
/// does: `tfDXT1` and `tfADXT1` are one encoder told apart by whether the alpha means anything.
fn to_descriptor_format(
  format: TextureEncodingFormat,
  descriptor: Option<&TextureDescriptorSave>,
) -> Option<ThmFormat> {
  match format {
    TextureEncodingFormat::Bc1 => {
      let flags: ThmTextureFlags = ThmTextureFlags::from(descriptor.map_or(0, |save| save.form.flags));

      Some(if flags.has(ThmTextureFlag::HasAlpha) {
        ThmFormat::Dxt1Alpha
      } else {
        ThmFormat::Dxt1
      })
    }
    TextureEncodingFormat::Bc2 => Some(ThmFormat::Dxt3),
    TextureEncodingFormat::Bc3 => Some(ThmFormat::Dxt5),
    TextureEncodingFormat::Rgba8 => Some(ThmFormat::Rgba),
    TextureEncodingFormat::Bc7 => None,
  }
}
