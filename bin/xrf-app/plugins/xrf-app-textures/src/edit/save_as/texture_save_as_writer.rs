use std::fs;
use std::path::{Path, PathBuf};

use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, FileStamp};
use xrf_chunk::XRayByteOrder;
use xrf_job::{JobHandle, JobOutcome};
use xrf_thm::{ThmFile, ThmFormat};
use xrf_utils::{OutputFiles, error_to_string, format_path};
use xrf_vfs::{XrayAsset, XraySearch, XrayTransaction};

use crate::edit::{
  TextureDescriptorForm, TextureSaveAsFile, TextureSaveAsOutcome, TextureSaveAsPlan, TextureSaveAsRole, TextureWrite,
  TexturesSaveAsRequest,
};

/// Writes a Save as copy of a texture in one commit.
pub(crate) struct TextureSaveAsWriter;

impl TextureSaveAsWriter {
  /// Writes the copy `request` asks for, using `texture` as its pixels when a re-encoded format was chosen.
  ///
  /// All files go in one transaction through the texture's roots, so either the whole copy is written or nothing is.
  /// The commit never writes into an archive and refuses to replace a file that appeared on disk after the roots
  /// were read.
  ///
  /// # Errors
  ///
  /// Returns an error when the plan is refused, either original file changed since the editor read it, a file
  /// already has a copy's name and the save is not forced, or a file cannot be read or written.
  pub(crate) fn write(
    assets: &AssetMountState,
    job: &JobHandle,
    request: &TexturesSaveAsRequest,
    texture: Option<TextureWrite>,
  ) -> TauriResult<TextureSaveAsOutcome> {
    let search: XraySearch = assets.search(&request.target.roots)?;
    let plan: TextureSaveAsPlan = TextureSaveAsPlan::build(&search, &request.target, &request.form, request.bump)?;

    // The copy must be of what the editor showed, so an original changed since then is refused.
    for source in [&request.sources.texture, &request.sources.descriptor] {
      FileStamp::check_unchanged(Path::new(&source.path), source.expected)?;
    }

    Self::assert_same_texture(&search, &plan, Path::new(&request.sources.texture.path))?;

    if !request.is_forced {
      Self::assert_free(&plan)?;
    }

    let mut form: TextureDescriptorForm = request.form.clone();

    for rename in &plan.renames {
      rename.apply_to(&mut form);
    }

    let synced_format: Option<ThmFormat> = texture
      .as_ref()
      .and_then(|texture| texture.format.to_thm_format(form.has_alpha()));
    let mut transaction: XrayTransaction = XrayTransaction::new(&search);

    for file in &plan.files {
      let bytes: Vec<u8> = match (file.role, &texture) {
        (TextureSaveAsRole::Texture, Some(texture)) => texture.bytes.clone(),
        (TextureSaveAsRole::Descriptor, _) => form.to_bytes(Self::read_descriptor(&search, file)?, synced_format)?,
        _ => Self::read_original(&search, file)?,
      };

      transaction = transaction.with_write(&file.to, bytes).map_err(error_to_string)?;
    }

    // Stopping is checked once, after everything is read and built and before the commit writes anything.
    if job.is_cancelled() {
      return Ok(TextureSaveAsOutcome {
        outcome: JobOutcome::Cancelled,
        reference: plan.destination,
        written: Vec::new(),
      });
    }

    assets.commit(&transaction)?;

    Ok(TextureSaveAsOutcome {
      outcome: JobOutcome::Completed,
      written: Self::locate_written(assets, request, &plan)?,
      reference: plan.destination,
    })
  }

  /// Refuses to copy any file but the one the editor read. A texture opened from outside every root is known only by
  /// its file name, and the roots may hold a different file with that name.
  fn assert_same_texture(search: &XraySearch, plan: &TextureSaveAsPlan, read: &Path) -> TauriResult {
    let planned: Option<PathBuf> = plan
      .files
      .iter()
      .find(|file| file.role == TextureSaveAsRole::Texture)
      .and_then(|file| file.from.as_deref())
      .map(|from| search.find(from))
      .transpose()
      .map_err(error_to_string)?
      .flatten()
      .as_ref()
      .and_then(XrayAsset::to_physical_path);
    let is_same: bool = planned
      .as_deref()
      .and_then(|planned| Some((fs::canonicalize(planned).ok()?, fs::canonicalize(read).ok()?)))
      .is_some_and(|(planned, read)| planned == read);

    if is_same {
      return Ok(());
    }

    Err(format!(
      "The open roots hold a different file under this texture's name than '{}', so Save as cannot copy it",
      format_path(read)
    ))
  }

  /// Refuses a copy whose names are already taken, worded as the replace prompt expects.
  fn assert_free(plan: &TextureSaveAsPlan) -> TauriResult {
    let taken: Vec<String> = plan
      .files
      .iter()
      .filter(|file| file.is_replacing)
      .map(|file| file.to.clone())
      .collect();

    if taken.is_empty() {
      return Ok(());
    }

    Err(OutputFiles::new_replace_refusal("Game data", &taken).to_string())
  }

  /// The original's bytes for one file of the copy.
  fn read_original(search: &XraySearch, file: &TextureSaveAsFile) -> TauriResult<Vec<u8>> {
    let from: &str = file
      .from
      .as_deref()
      .ok_or_else(|| format!("No original file to copy into '{}'", file.to))?;

    search.read(from).map_err(error_to_string)
  }

  /// The original's descriptor, if it has one. The copy's is built on it so chunks the form does not edit are kept.
  fn read_descriptor(search: &XraySearch, file: &TextureSaveAsFile) -> TauriResult<Option<ThmFile>> {
    if file.from.is_none() {
      return Ok(None);
    }

    Ok(Some(
      ThmFile::read_from_bytes::<XRayByteOrder>(Self::read_original(search, file)?).map_err(error_to_string)?,
    ))
  }

  /// Where each file of the copy was written on disk.
  fn locate_written(
    assets: &AssetMountState,
    request: &TexturesSaveAsRequest,
    plan: &TextureSaveAsPlan,
  ) -> TauriResult<Vec<String>> {
    let search: XraySearch = assets.search(&request.target.roots)?;

    plan
      .files
      .iter()
      .map(|file| {
        search
          .find(&file.to)
          .map_err(error_to_string)?
          .as_ref()
          .and_then(XrayAsset::to_physical_path)
          .map(|path| path.display().to_string())
          .ok_or_else(|| format!("'{}' was written but cannot be found on disk", file.to))
      })
      .collect()
  }
}
