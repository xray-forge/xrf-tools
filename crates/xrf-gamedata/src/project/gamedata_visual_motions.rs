use std::collections::BTreeSet;
use std::sync::Arc;

use xrf_chunk::ChunkReader;
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_ogf::OgfFile;
use xrf_omf::OmfFile;
use xrf_spawn::XRayByteOrder;
use xrf_vfs::XrayAssetType;

use crate::GamedataProject;

/// The motions a skinned visual plays, through the motion banks its motion refs name: what the weapons and HUD
/// animation checks match `anm_*` fields against.
pub(crate) struct GamedataVisualMotions;

impl GamedataVisualMotions {
  /// The motion refs of a visual, or `None` for a static visual that carries none.
  ///
  /// # Errors
  ///
  /// Returns an error when the visual does not read up to its motion refs.
  pub(crate) fn read_refs(project: &GamedataProject, visual_path: &str) -> XrfResult<Option<Vec<String>>> {
    match project
      .read_bytes(visual_path)
      .and_then(ChunkReader::from_vec)
      .and_then(|mut chunk| OgfFile::read_motion_refs_from_chunk::<XRayByteOrder, _>(&mut chunk))
    {
      Ok(refs) => Ok(Some(refs)),
      Err(XrfError::NotFound { .. }) => Ok(None),
      Err(error) => Err(error),
    }
  }

  /// The motion banks one ref names, which may be a `*` mask, by logical path; empty where none resolves.
  ///
  /// # Errors
  ///
  /// Returns an error when the ref is not a valid reference.
  pub(crate) fn resolve_banks(project: &GamedataProject, motion_ref: &str) -> XrfResult<BTreeSet<String>> {
    Ok(
      project
        .resolve_all(XrayAssetType::Omf, motion_ref)?
        .into_iter()
        .filter(|location| location.is_type(XrayAssetType::Omf))
        .map(|location| location.get_logical_path().to_string())
        .collect(),
    )
  }

  /// The names of every motion one bank holds. The whole bank is retained, as the meshes and weapons checks read it too.
  ///
  /// # Errors
  ///
  /// Returns an error when the bank does not read.
  pub(crate) fn read_names(project: &GamedataProject, bank_path: &str) -> XrfResult<Vec<String>> {
    let bank: Arc<OmfFile> = project.read_parsed(bank_path, OmfFile::read_from_chunk::<XRayByteOrder, _>)?;

    Ok(bank.get_motion_names().into_iter().map(str::to_owned).collect())
  }

  /// Every motion a visual plays, or `None` for a static visual.
  ///
  /// # Errors
  ///
  /// Returns an error on cancellation, or when the visual, a ref or a bank does not read.
  pub(crate) fn read_all(
    project: &GamedataProject,
    visual_path: &str,
    job: &JobHandle,
  ) -> XrfResult<Option<BTreeSet<String>>> {
    let Some(refs) = Self::read_refs(project, visual_path)? else {
      return Ok(None);
    };

    let mut motions: BTreeSet<String> = BTreeSet::new();

    for motion_ref in &refs {
      for bank in Self::resolve_banks(project, motion_ref)? {
        job.check_cancelled()?;

        motions.extend(Self::read_names(project, &bank)?);
      }
    }

    Ok(Some(motions))
  }
}
