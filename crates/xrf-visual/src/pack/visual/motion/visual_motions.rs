use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_ogf::{OgfBone, OgfBoneIkData, OgfFile};
use xrf_omf::OmfFile;
use xrf_skeleton::{SkeletonMotion, SkeletonMotionDefinition, SkeletonPart};
use xrf_vfs::{XrayAsset, XraySearch};

use crate::pack::visual::motion::visual_motion_bake::VisualMotionBake;
use crate::pack::visual::motion::visual_motion_baker::VisualMotionBaker;
use crate::pack::visual::motion::visual_motion_pose::VisualMotionPose;
use crate::resolve::visual_dependencies::VisualDependencies;

/// What reads one motion onto a skeleton, given its bones and their binds, and the motion's partition, definition and
/// keys.
type MotionReader<T> =
  fn(&[OgfBone], &[OgfBoneIkData], &[SkeletonPart], &SkeletonMotionDefinition, &SkeletonMotion) -> XrfResult<T>;

/// What posing a visual needs from its file, read once so a motion costs no re-read: its bones with their binds, and the
/// motions it carries itself. Motions it references are read from the animation files its dependencies located.
pub struct VisualMotions {
  pub bones: Vec<OgfBone>,
  pub binds: Vec<OgfBoneIkData>,
  /// Partitions of the motions this file embeds, empty when it animates only from referenced omf files.
  pub embedded_parts: Vec<SkeletonPart>,
  /// Motions the file carries itself, each definition with the key payload at its own ordinal.
  pub embedded_motions: Vec<(SkeletonMotionDefinition, SkeletonMotion)>,
}

impl VisualMotions {
  /// What a visual offers for posing, or nothing when it carries no bind pose to pose from.
  pub fn of(file: &OgfFile) -> Option<Self> {
    let bones: &Vec<OgfBone> = &file.bones.as_ref()?.bones;
    let binds: &Vec<OgfBoneIkData> = &file.ik_data.as_ref()?.bones;

    if bones.len() != binds.len() {
      return None;
    }

    Some(Self {
      bones: bones.clone(),
      binds: binds.clone(),
      embedded_motions: file
        .get_motions()
        .map(|(definition, motion)| (definition.clone(), motion.clone()))
        .collect(),
      embedded_parts: file
        .motion_parameters
        .as_ref()
        .map(|it| it.parts.clone())
        .unwrap_or_default(),
    })
  }

  /// Bakes one named motion onto the skeleton.
  ///
  /// # Errors
  ///
  /// Returns an error when no source the visual names carries the motion, or when the one that does cannot be posed onto
  /// this skeleton.
  pub fn bake_named(
    &self,
    search: &XraySearch,
    dependencies: &VisualDependencies,
    name: &str,
  ) -> XrfResult<VisualMotionPose> {
    self.read_named(search, dependencies, (name, "pose"), VisualMotionBaker::bake)
  }

  /// Describes one named motion as a bake of it onto the skeleton would hold it, posing no frame.
  ///
  /// # Errors
  ///
  /// Returns an error where [`Self::bake_named`] would.
  pub fn describe_named(
    &self,
    search: &XraySearch,
    dependencies: &VisualDependencies,
    name: &str,
  ) -> XrfResult<VisualMotionBake> {
    self.read_named(search, dependencies, (name, "describe"), VisualMotionBaker::describe)
  }

  /// Every motion name the visual can play, embedded ones first then each referenced file's.
  pub fn list_names(&self, search: &XraySearch, dependencies: &VisualDependencies) -> Vec<String> {
    let mut names: Vec<String> = self
      .embedded_motions
      .iter()
      .map(|(definition, _)| definition.name.clone())
      .collect();

    for dependency in &dependencies.motions {
      for asset in dependency.resolution.get_assets() {
        let Ok(omf) = Self::read_omf(search, asset) else {
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

  /// Reads one named motion through `read`: the embedded one of that name first, then the first referenced file's;
  /// `verb` says what was done in a refusal.
  fn read_named<T>(
    &self,
    search: &XraySearch,
    dependencies: &VisualDependencies,
    (name, verb): (&str, &str),
    read: MotionReader<T>,
  ) -> XrfResult<T> {
    if let Some((definition, motion)) = self
      .embedded_motions
      .iter()
      .find(|(definition, _)| definition.name == name)
    {
      return read(&self.bones, &self.binds, &self.embedded_parts, definition, motion)
        .map_err(|error| XrfError::new_invalid_error(format!("Failed to {verb} embedded motion '{name}': {error}")));
    }

    let mut searched: usize = 0;

    for dependency in &dependencies.motions {
      for asset in dependency.resolution.get_assets() {
        searched += 1;

        let Ok(omf) = Self::read_omf(search, asset) else {
          continue;
        };

        if let Some((definition, motion)) = omf.get_motion_by_name(name) {
          return read(&self.bones, &self.binds, &omf.parameters.parts, definition, motion)
            .map_err(|error| XrfError::new_invalid_error(format!("Failed to {verb} motion '{name}': {error}")));
        }
      }
    }

    Err(XrfError::new_not_found_error(format!(
      "Motion '{name}' is in none of the {searched} animation files this visual references"
    )))
  }

  /// One animation file a dependency located, read and parsed.
  fn read_omf(search: &XraySearch, asset: &XrayAsset) -> XrfResult<OmfFile> {
    let bytes: Vec<u8> = search
      .read_asset(asset)
      .map_err(|error| XrfError::new_read_error(format!("Failed to read '{}': {error}", asset.get_logical_path())))?;

    OmfFile::read_from_bytes::<XRayByteOrder>(bytes)
      .map_err(|error| XrfError::new_parsing_error(format!("Failed to parse '{}': {error}", asset.get_logical_path())))
  }
}
