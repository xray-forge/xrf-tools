use rayon::prelude::*;
use xrf_dds::DdsFile;
use xrf_error::XrfResult;
use xrf_material::{
  XrayBumpNaming, XrayMaterialBump, XrayMaterialDeclaration, XrayMaterialDescriptor, XrayMaterialResolver,
};
use xrf_texture::BumpAlignment;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayResolution, XraySearch};

use crate::commands::thm::bump_alignment::bump_alignment_set::{BumpAlignmentOutcome, BumpAlignmentSet};

/// Every texture set a root's descriptors declare a bump for, each compared with its diffuse.
pub struct BumpAlignmentSurvey {
  /// Descriptors read.
  pub descriptors: usize,
  /// One a descriptor declaring a bump the engine reads, in path order.
  pub sets: Vec<BumpAlignmentSet>,
}

impl BumpAlignmentSurvey {
  /// Surveys the descriptors `listed` holds, resolving what they name through `search` as the renderer does outside
  /// any level, so a set narrowed to a subtree still finds a bump kept outside it.
  pub fn run(search: &XraySearch, listed: &XraySearch) -> Self {
    let descriptors: Vec<XrayAsset> = listed.list_assets(XrayAssetType::Thm);
    // Decoding is the survey's whole cost, and each set's is its own; the order is the listing's.
    let sets: Vec<BumpAlignmentSet> = descriptors
      .par_iter()
      .filter_map(|descriptor| Self::survey(search, descriptor))
      .collect();

    Self {
      descriptors: descriptors.len(),
      sets,
    }
  }

  /// The set one descriptor declares, `None` for one declaring no bump the engine reads.
  fn survey(search: &XraySearch, descriptor: &XrayAsset) -> Option<BumpAlignmentSet> {
    let material: XrayMaterialDescriptor = XrayMaterialResolver::describe_descriptor(search, descriptor);
    let (XrayMaterialDeclaration::Declared { name, .. }, Some(bump)) = (&material.declaration, &material.bump) else {
      return None;
    };
    let reference: Option<String> = descriptor.to_reference();
    let diffuse: Option<XrayAsset> = reference
      .as_ref()
      .and_then(|reference| search.resolve(XrayAssetType::Dds, reference).ok())
      .and_then(XrayResolution::into_asset);
    let bump_file: Option<&XrayAsset> = Self::declared_file(bump);
    let is_borrowed: bool =
      reference.is_none_or(|reference| !name.eq_ignore_ascii_case(&XrayBumpNaming::bump_of(&reference)));

    Some(BumpAlignmentSet {
      descriptor: descriptor.get_logical_path().as_str().to_owned(),
      diffuse: diffuse
        .as_ref()
        .map(|asset| asset.get_logical_path().as_str().to_owned()),
      bump: name.clone(),
      bump_file: bump_file.map(|asset| asset.get_logical_path().as_str().to_owned()),
      is_borrowed,
      outcome: match (&diffuse, bump_file) {
        (None, _) => BumpAlignmentOutcome::MissingDiffuse,
        (_, None) => BumpAlignmentOutcome::MissingBump,
        (Some(diffuse), Some(bump)) => match Self::read(search, diffuse, bump) {
          Ok(alignment) => BumpAlignmentOutcome::Measured { alignment },
          Err(error) => BumpAlignmentOutcome::Unread {
            reason: error.to_string(),
          },
        },
      },
    })
  }

  /// The bump file the declaration named, not a stand-in the renderer binds in its place.
  fn declared_file(bump: &XrayMaterialBump) -> Option<&XrayAsset> {
    bump
      .bump
      .is_declared_file()
      .then(|| bump.bump.resolution.get_asset())
      .flatten()
  }

  fn read(search: &XraySearch, diffuse: &XrayAsset, bump: &XrayAsset) -> XrfResult<BumpAlignment> {
    BumpAlignment::read(
      &DdsFile::read_from_bytes(&search.read_asset(diffuse)?)?,
      &DdsFile::read_from_bytes(&search.read_asset(bump)?)?,
    )
  }
}
