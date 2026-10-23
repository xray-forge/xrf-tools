use xrf_error::XrfResult;
use xrf_ogf::{OgfBone, OgfBoneIkData};
use xrf_omf::OmfFile;
use xrf_skeleton::{SkeletonMotion, SkeletonMotionDefinition, SkeletonPart};
use xrf_spawn::XRayByteOrder;
use xrf_vfs::{XrayAsset, XrayResolution, XraySearch};
use xrf_visual::{VisualDependencies, VisualMotionBake, VisualMotionPose, bake_motion, describe_motion};

use crate::core::types::TauriResult;
use crate::plugins::visuals::skeleton::SelectedSkeleton;

/// What reads one motion onto a skeleton, given its bones and their binds, and the motion's partition, definition and
/// keys.
type MotionReader<T> =
  fn(&[OgfBone], &[OgfBoneIkData], &[SkeletonPart], &SkeletonMotionDefinition, &SkeletonMotion) -> XrfResult<T>;

/// Bakes one named motion onto a selected visual's skeleton.
///
/// # Errors
///
/// Returns an error when no source the visual names carries the motion, or when the one that does cannot be posed onto
/// this skeleton.
pub fn bake_named_motion(
  search: &XraySearch,
  skeleton: &SelectedSkeleton,
  dependencies: &VisualDependencies,
  name: &str,
) -> TauriResult<VisualMotionPose> {
  read_named_motion(search, skeleton, dependencies, (name, "pose"), bake_motion)
}

/// Describes one named motion as a bake of it onto a selected visual's skeleton would hold it, posing no frame.
///
/// # Errors
///
/// Returns an error where `bake_named_motion` would.
pub fn describe_named_motion(
  search: &XraySearch,
  skeleton: &SelectedSkeleton,
  dependencies: &VisualDependencies,
  name: &str,
) -> TauriResult<VisualMotionBake> {
  read_named_motion(search, skeleton, dependencies, (name, "describe"), describe_motion)
}

/// Reads one named motion through `read`: the embedded one of that name first, then the first referenced file's;
/// `verb` says what was done in a refusal.
fn read_named_motion<T>(
  search: &XraySearch,
  skeleton: &SelectedSkeleton,
  dependencies: &VisualDependencies,
  (name, verb): (&str, &str),
  read: MotionReader<T>,
) -> TauriResult<T> {
  if let Some((definition, motion)) = skeleton
    .embedded_motions
    .iter()
    .find(|(definition, _)| definition.name == name)
  {
    return read(
      &skeleton.bones,
      &skeleton.binds,
      &skeleton.embedded_parts,
      definition,
      motion,
    )
    .map_err(|error| format!("Failed to {verb} embedded motion '{name}': {error}"));
  }

  let mut searched: usize = 0;

  for dependency in &dependencies.motions {
    for asset in located_assets(&dependency.resolution) {
      searched += 1;

      let Ok(omf) = read_omf(search, asset) else {
        continue;
      };

      if let Some((definition, motion)) = omf.get_motion_by_name(name) {
        return read(
          &skeleton.bones,
          &skeleton.binds,
          &omf.parameters.parts,
          definition,
          motion,
        )
        .map_err(|error| format!("Failed to {verb} motion '{name}': {error}"));
      }
    }
  }

  Err(format!(
    "Motion '{name}' is in none of the {searched} animation files this visual references"
  ))
}

/// Every motion name the visual can play, embedded ones first then each referenced file's.
pub fn list_motion_names(
  search: &XraySearch,
  skeleton: &SelectedSkeleton,
  dependencies: &VisualDependencies,
) -> Vec<String> {
  let mut names: Vec<String> = skeleton
    .embedded_motions
    .iter()
    .map(|(definition, _)| definition.name.clone())
    .collect();

  for dependency in &dependencies.motions {
    for asset in located_assets(&dependency.resolution) {
      let Ok(omf) = read_omf(search, asset) else {
        continue;
      };

      for name in omf.get_motion_names() {
        if !names.iter().any(|it| it == name) {
          names.push(String::from(name));
        }
      }
    }
  }

  names
}

fn located_assets(resolution: &XrayResolution) -> &[XrayAsset] {
  resolution.get_assets()
}

fn read_omf(search: &XraySearch, asset: &XrayAsset) -> TauriResult<OmfFile> {
  let bytes: Vec<u8> = search
    .read_asset(asset)
    .map_err(|error| format!("Failed to read '{}': {error}", asset.get_logical_path()))?;

  OmfFile::read_from_bytes::<XRayByteOrder>(bytes)
    .map_err(|error| format!("Failed to parse '{}': {error}", asset.get_logical_path()))
}
