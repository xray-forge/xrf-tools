use xrf_db::{OmfFile, XRayByteOrder};
use xrf_vfs::{XrayAsset, XrayProbe, XrayResolution};
use xrf_visual::{VisualDependencies, VisualMotionPose, bake_motion};

use crate::core::types::TauriResult;
use crate::plugins::visuals::skeleton::SelectedSkeleton;

/// Bakes one named motion onto a selected visual's skeleton.
///
/// The motion is looked for where the visual says it animates from: the motions it embeds first, then each omf its
/// kinematics chunk references, in the order the file lists them. First match wins, which is the order the engine
/// would resolve a name in.
///
/// # Errors
///
/// Returns an error when no source the visual names carries the motion, or when the one that does cannot be posed onto
/// this skeleton.
pub fn bake_named_motion(
  probe: &XrayProbe,
  skeleton: &SelectedSkeleton,
  dependencies: &VisualDependencies,
  name: &str,
) -> TauriResult<VisualMotionPose> {
  if let Some(motion) = skeleton.embedded_motions.iter().find(|it| it.name == name) {
    return bake_motion(&skeleton.bones, &skeleton.binds, &skeleton.embedded_parts, motion)
      .map_err(|error| format!("Failed to pose embedded motion '{name}': {error}"));
  }

  let mut searched: usize = 0;

  for dependency in &dependencies.motions {
    for asset in located_assets(&dependency.resolution) {
      searched += 1;

      let Ok(omf) = read_omf(probe, asset) else {
        continue;
      };

      if let Some(motion) = omf.motions.motions.iter().find(|it| it.name == name) {
        return bake_motion(&skeleton.bones, &skeleton.binds, &omf.parameters.parts, motion)
          .map_err(|error| format!("Failed to pose motion '{name}': {error}"));
      }
    }
  }

  Err(format!(
    "Motion '{name}' is in none of the {searched} animation files this visual references"
  ))
}

/// Every motion name the visual can play, embedded ones first then each referenced file's.
///
/// Listed by reading the referenced files rather than from the reference alone, because a reference names a file and a
/// file carries many motions. Duplicates are dropped so the first source that offers a name is the one a later bake
/// finds, which keeps the list and the bake in agreement.
pub fn list_motion_names(
  probe: &XrayProbe,
  skeleton: &SelectedSkeleton,
  dependencies: &VisualDependencies,
) -> Vec<String> {
  let mut names: Vec<String> = skeleton.embedded_motions.iter().map(|it| it.name.clone()).collect();

  for dependency in &dependencies.motions {
    for asset in located_assets(&dependency.resolution) {
      let Ok(omf) = read_omf(probe, asset) else {
        continue;
      };

      for motion in &omf.motions.motions {
        if !names.iter().any(|it| it == &motion.name) {
          names.push(motion.name.clone());
        }
      }
    }
  }

  names
}

fn located_assets(resolution: &XrayResolution) -> &[XrayAsset] {
  resolution.get_assets()
}

fn read_omf(probe: &XrayProbe, asset: &XrayAsset) -> TauriResult<OmfFile> {
  let bytes: Vec<u8> = probe
    .read_asset_bytes(asset)
    .map_err(|error| format!("Failed to read '{}': {error}", asset.get_logical_path()))?;

  OmfFile::read_from_bytes::<XRayByteOrder>(bytes)
    .map_err(|error| format!("Failed to parse '{}': {error}", asset.get_logical_path()))
}
