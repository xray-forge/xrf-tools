use std::collections::{HashMap, HashSet};

use rayon::prelude::*;
use xrf_db::{OgfFile, OgfResidueCause, OmfFile, ShaderLibraryFile, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};
use xrf_output::{OutputOptions, OutputSequence, OutputSlot};
use xrf_vfs::XrayAssetType as AssetType;

use crate::GamedataFindingFactory;
use crate::project::meshes::mesh_assets_verification_result::GamedataMeshAssetsVerificationResult;
use crate::{Finding, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

pub(crate) struct MeshAssetsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
  shader_library: &'a ShaderLibraryFile,
}

impl<'a> MeshAssetsVerifier<'a> {
  pub(crate) fn new(
    project: &'a GamedataProject,
    options: &'a GamedataProjectVerifyOptions,
    shader_library: &'a ShaderLibraryFile,
  ) -> Self {
    Self {
      options,
      project,
      shader_library,
    }
  }

  pub(crate) fn verify(&self) -> XrfResult<GamedataMeshAssetsVerificationResult> {
    let options = self.options;
    let shader_library = self.shader_library;
    // Enumerated through the VFS, so an installation's archived meshes are verified too.
    let mesh_paths: Vec<String> = self
      .project
      .vfs()
      .scoped(&self.project.scope)
      .list_entries_of_type(AssetType::Ogf)
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    let checked_meshes_count: u32 = u32::try_from(mesh_paths.len())
      .map_err(|_| XrfError::new_verify_error("Mesh count exceeds the supported result range"))?;

    // Meshes are read in parallel and finish in whatever order their sizes and the scheduler decide,
    // so each one logs into its listed position and the sequence releases them in path order.
    let sequence: OutputSequence = OutputSequence::new(&options.output, mesh_paths.len());

    let mesh_findings: Vec<Vec<Finding>> = mesh_paths
      .par_iter()
      .enumerate()
      .map(|(index, relative_path)| {
        let slot: OutputSlot = sequence.new_slot(index);
        let output: &OutputOptions = slot.get_output();

        xrf_output::verbose!(output, "Verify mesh: {relative_path}");

        // Read through the VFS, so a mesh inside an archive volume is verified rather than reported missing.
        let path: &str = relative_path;

        match self.project.read_parsed(AssetType::Ogf, path, |chunk| {
          OgfFile::read_from_chunk::<XRayByteOrder, _>(chunk)
        }) {
          Ok(ogf) => match self.verify_mesh_findings(output, shader_library, &ogf, Some(path), None) {
            Ok(findings) if findings.is_empty() => Vec::new(),
            Ok(findings) => {
              xrf_output::error!(output, "Mesh is not valid: {}", path);

              findings
            }
            Err(error) => {
              xrf_output::error!(output, "Mesh verification failed: {} - {}", path, error);

              vec![GamedataFindingFactory::for_asset(
                GamedataVerificationRule::MeshesValidation,
                path,
                format!("Failed to verify mesh: {error}"),
              )]
            }
          },
          Err(error) => {
            xrf_output::error!(output, "Mesh verification failed: {} - {}", path, error);

            vec![GamedataFindingFactory::for_asset(
              GamedataVerificationRule::MeshesRead,
              path,
              format!("Failed to read mesh: {error}"),
            )]
          }
        }
      })
      .collect();

    let invalid_meshes_count: u32 = u32::try_from(mesh_findings.iter().filter(|findings| !findings.is_empty()).count())
      .map_err(|_| XrfError::new_verify_error("Invalid mesh count exceeds the supported result range"))?;

    let mut findings: Vec<Finding> = mesh_findings.into_iter().flatten().collect();

    // A referenced animation bank is verified once per visual that names it, and hundreds of hands models share one
    // bank. The finding is about the bank, so report it once rather than once per visual that led there.
    findings.sort_by(GamedataFindingFactory::cmp_by_asset_path_rule_and_message);
    findings.dedup();

    Ok(GamedataMeshAssetsVerificationResult {
      findings,
      invalid_meshes_count,
      checked_meshes_count,
      ..Default::default()
    })
  }

  fn verify_mesh_findings(
    &self,
    output: &OutputOptions,
    shader_library: &ShaderLibraryFile,
    ogf: &OgfFile,
    mesh_path: Option<&str>,
    inherited_bones_count: Option<usize>,
  ) -> XrfResult<Vec<Finding>> {
    let bones_count: Option<usize> = ogf
      .bones
      .as_ref()
      .map(|bones| bones.bones.len())
      .or(inherited_bones_count);

    let mut findings: Vec<Finding> = self.verify_mesh_texture_findings(output, ogf, mesh_path);

    findings.extend(self.verify_mesh_shader_findings(output, shader_library, ogf, mesh_path));
    findings.extend(self.verify_mesh_skeleton_findings(ogf, mesh_path));
    findings.extend(self.verify_mesh_geometry_findings(ogf, mesh_path, bones_count));
    findings.extend(Self::verify_motion_label_findings(
      ogf.get_diverging_labels_count(),
      ogf.get_motions().count(),
      mesh_path,
    ));
    findings.extend(Self::verify_mesh_residue_findings(ogf, mesh_path));

    // Verify all nested children in mesh object.
    if let Some(children) = &ogf.children {
      for child in &children.nested {
        findings.extend(self.verify_mesh_findings(output, shader_library, child, mesh_path, bones_count)?);
      }
    }

    // Verify all motion refs injected in OGF file.
    if let Some(kinematics) = &ogf.kinematics {
      for motion_ref in &kinematics.motion_refs {
        let motion_paths: Vec<String> = self
          .project
          .vfs()
          .scoped(&self.project.scope)
          .resolve_all(AssetType::Omf, motion_ref)?
          .into_iter()
          .map(|location| location.get_logical_path().to_string())
          .collect();

        if motion_paths.is_empty() {
          xrf_output::error!(output, "Mesh motion refs not found by path: {motion_ref}");

          findings.push(Self::new_mesh_finding(
            GamedataVerificationRule::MeshesMotionValidation,
            mesh_path,
            format!("Mesh references missing motion '{motion_ref}'"),
          ));
        } else {
          for motion_path in motion_paths {
            // Retained per path, because a shared animation bank is referenced by hundreds of visuals and each read is
            // a whole-entry decompression: four Anomaly banks account for 47GB of a sweep's 77GB without this.
            match self.project.read_parsed(AssetType::Omf, &motion_path, |chunk| {
              OmfFile::read_from_chunk::<XRayByteOrder, _>(chunk)
            }) {
              Ok(omf) => match self.verify_mesh_motion_findings(output, ogf, &omf, Some(&motion_path)) {
                Ok(motion_findings) => findings.extend(motion_findings),
                Err(error) => {
                  xrf_output::error!(
                    output,
                    "Mesh motion verification failed: {}, error: {}",
                    motion_path,
                    error
                  );

                  findings.push(GamedataFindingFactory::for_asset(
                    GamedataVerificationRule::MeshesMotionValidation,
                    &motion_path,
                    format!("Failed to verify referenced motion: {error}"),
                  ));
                }
              },
              Err(error) => {
                xrf_output::error!(
                  output,
                  "Mesh motion file failed to read: {}, error: {}",
                  motion_path,
                  error
                );

                findings.push(GamedataFindingFactory::for_asset(
                  GamedataVerificationRule::MeshesMotionRead,
                  &motion_path,
                  format!("Failed to read referenced motion: {error}"),
                ));
              }
            }
          }
        }
      }
    }

    // todo: Verify LOD?

    Ok(findings)
  }

  fn verify_mesh_texture_findings(
    &self,
    output: &OutputOptions,
    ogf: &OgfFile,
    mesh_path: Option<&str>,
  ) -> Vec<Finding> {
    let mut findings: Vec<Finding> = Vec::new();

    if let Some(texture) = &ogf.texture
      && self
        .project
        .vfs()
        .scoped(&self.project.scope)
        .dds_texture(&texture.texture_name)
        .ok()
        .flatten()
        .is_none()
    {
      xrf_output::error!(output, "Cannot read OGF texture: {}", texture.texture_name);

      findings.push(Self::new_mesh_finding(
        GamedataVerificationRule::MeshesValidation,
        mesh_path,
        format!("Mesh references missing texture '{}'", texture.texture_name),
      ));
    }

    findings
  }

  fn verify_mesh_shader_findings(
    &self,
    output: &OutputOptions,
    shader_library: &ShaderLibraryFile,
    ogf: &OgfFile,
    mesh_path: Option<&str>,
  ) -> Vec<Finding> {
    let Some(texture) = &ogf.texture else {
      return Vec::new();
    };

    if shader_library.contains_blender(&texture.shader_name) {
      return Vec::new();
    }

    xrf_output::error!(
      output,
      "Cannot resolve OGF shader '{}' in shaders.xr",
      texture.shader_name
    );

    vec![Self::new_mesh_finding(
      GamedataVerificationRule::MeshesValidation,
      mesh_path,
      format!(
        "Mesh references shader '{}' that is not defined in shaders.xr",
        texture.shader_name
      ),
    )]
  }

  fn verify_mesh_skeleton_findings(&self, ogf: &OgfFile, mesh_path: Option<&str>) -> Vec<Finding> {
    let Some(bones) = &ogf.bones else {
      return Vec::new();
    };

    Self::skeleton_topology_findings(
      bones
        .bones
        .iter()
        .map(|bone| (bone.name.as_str(), bone.parent.as_str())),
    )
    .into_iter()
    .map(|message| Self::new_mesh_finding(GamedataVerificationRule::MeshesValidation, mesh_path, message))
    .collect()
  }

  fn verify_mesh_geometry_findings(
    &self,
    ogf: &OgfFile,
    mesh_path: Option<&str>,
    bones_count: Option<usize>,
  ) -> Vec<Finding> {
    let Some(geometry) = &ogf.geometry else {
      return Vec::new();
    };

    let mut findings: Vec<Finding> = Vec::new();

    if let Some(indices) = &geometry.indices {
      if indices.len() % 3 != 0 {
        findings.push(Self::new_mesh_finding(
          GamedataVerificationRule::MeshesValidation,
          mesh_path,
          format!(
            "Mesh geometry contains {} indices, which is not divisible by 3",
            indices.len()
          ),
        ));
      }

      if let Some(vertex_count) = geometry.vertex_count
        && let Some(index) = indices.iter().find(|index| **index as u32 >= vertex_count)
      {
        findings.push(Self::new_mesh_finding(
          GamedataVerificationRule::MeshesValidation,
          mesh_path,
          format!("Mesh geometry index {index} references a vertex outside the {vertex_count} vertices"),
        ));
      }
    }

    if !geometry.skin_bone_indices.is_empty() {
      match bones_count {
        Some(bones_count) => {
          if let Some(bone_index) = geometry
            .skin_bone_indices
            .iter()
            .find(|index| **index as usize >= bones_count)
          {
            findings.push(Self::new_mesh_finding(
              GamedataVerificationRule::MeshesValidation,
              mesh_path,
              format!(
                "Mesh geometry skin vertex references bone {bone_index}, but the skeleton has {bones_count} bones"
              ),
            ));
          }
        }
        None => findings.push(Self::new_mesh_finding(
          GamedataVerificationRule::MeshesValidation,
          mesh_path,
          "Mesh geometry contains skinned vertices but no skeleton".to_string(),
        )),
      }
    }

    findings
  }

  fn skeleton_topology_findings<'bone>(bones: impl IntoIterator<Item = (&'bone str, &'bone str)>) -> Vec<String> {
    let bones: Vec<(&str, &str)> = bones.into_iter().collect();
    let mut findings: Vec<String> = Vec::new();
    let mut parents_by_name: HashMap<String, &str> = HashMap::with_capacity(bones.len());

    for (name, parent) in &bones {
      if name.is_empty() {
        findings.push("Mesh skeleton contains a bone with an empty name".to_string());
        continue;
      }

      let normalized_name: String = name.to_ascii_lowercase();

      if parents_by_name.insert(normalized_name, *parent).is_some() {
        findings.push(format!("Mesh skeleton contains duplicate bone name '{name}'"));
      }
    }

    let root_count: usize = bones.iter().filter(|(_, parent)| parent.is_empty()).count();

    if root_count != 1 {
      findings.push(format!(
        "Mesh skeleton must contain exactly one root bone, found {root_count}"
      ));
    }

    for (name, parent) in &bones {
      if !name.is_empty() && !parent.is_empty() && !parents_by_name.contains_key(&parent.to_ascii_lowercase()) {
        findings.push(format!(
          "Mesh skeleton bone '{name}' references missing parent '{parent}'"
        ));
      }
    }

    let mut checked_cycle_starts: HashSet<String> = HashSet::new();
    let mut reported_cycles: HashSet<String> = HashSet::new();

    for (name, _) in &bones {
      let name: String = name.to_ascii_lowercase();

      if name.is_empty() || !checked_cycle_starts.insert(name.clone()) {
        continue;
      }

      let mut chain: Vec<String> = vec![name.clone()];
      let mut current: String = name;

      while let Some(parent) = parents_by_name.get(&current) {
        if parent.is_empty() || !parents_by_name.contains_key(&parent.to_ascii_lowercase()) {
          break;
        }

        let normalized_parent: String = parent.to_ascii_lowercase();

        if let Some(cycle_start) = chain.iter().position(|entry| entry == &normalized_parent) {
          let mut cycle: Vec<String> = chain[cycle_start..].to_vec();
          let first: String = cycle.iter().min().expect("cycle has a member").clone();
          let first_index: usize = cycle
            .iter()
            .position(|entry| entry == &first)
            .expect("cycle contains its first member");
          cycle.rotate_left(first_index);
          cycle.push(first);
          let cycle: String = cycle.join(" -> ");

          if reported_cycles.insert(cycle.clone()) {
            findings.push(format!("Mesh skeleton contains parent cycle: {cycle}"));
          }

          break;
        }

        current = normalized_parent;
        chain.push(current.clone());
      }
    }

    findings
  }

  fn verify_mesh_motion_findings(
    &self,
    output: &OutputOptions,
    ogf: &OgfFile,
    omf: &OmfFile,
    motion_path: Option<&str>,
  ) -> XrfResult<Vec<Finding>> {
    let mut findings: Vec<Finding> = Vec::new();

    if let Some(bones) = &ogf.bones {
      let omf_bones: Vec<&str> = omf.get_bones();

      if bones.bones.len() != omf_bones.len() {
        xrf_output::error!(
          output,
          "Not matching bones count in ogf and reference omf: {} <-> {} : {} <-> {}",
          bones.bones.len(),
          omf_bones.len(),
          bones
            .bones
            .iter()
            .map(|it| it.name.as_str())
            .collect::<Vec<_>>()
            .join(","),
          omf_bones.join(",")
        );

        findings.push(Self::new_motion_finding(
          GamedataVerificationRule::MeshesMotionValidation,
          motion_path,
          format!(
            "Motion bone count does not match mesh: {} mesh bones, {} motion bones",
            bones.bones.len(),
            omf_bones.len()
          ),
        ));
      } else if bones.bones.iter().any(|it| !omf_bones.contains(&it.name.as_str())) {
        xrf_output::error!(
          output,
          "Missing bones in OMF file for OGF mesh: {} <-> {}",
          bones
            .bones
            .iter()
            .map(|it| it.name.as_str())
            .collect::<Vec<_>>()
            .join(","),
          omf_bones.join(",")
        );

        let missing_bones: Vec<&str> = bones
          .bones
          .iter()
          .filter_map(|bone| (!omf_bones.contains(&bone.name.as_str())).then_some(bone.name.as_str()))
          .collect();

        findings.push(Self::new_motion_finding(
          GamedataVerificationRule::MeshesMotionValidation,
          motion_path,
          format!("Motion is missing mesh bones: {}", missing_bones.join(",")),
        ));
      }
    }

    findings.extend(Self::verify_motion_label_findings(
      omf.get_diverging_labels_count(),
      omf.get_motions().count(),
      motion_path,
    ));

    Ok(findings)
  }

  /// Reports payload labels that no longer name the motion they are stored with.
  ///
  /// A motion is named by its definition and reached through its ordinal, so a divergent label costs release playback
  /// nothing - but a `_DEBUG` engine build asserts on it, and it marks a bank an editor rewrote without keeping the
  /// two in step. Reported once per file: the file is the unit that gets fixed, and `xrf-cli omf info` names the
  /// individual motions on demand.
  fn verify_motion_label_findings(diverging_count: usize, motions_count: usize, path: Option<&str>) -> Vec<Finding> {
    if diverging_count == 0 {
      return Vec::new();
    }

    vec![Self::new_motion_finding(
      GamedataVerificationRule::MeshesMotionLabel,
      path,
      format!("{diverging_count} of {motions_count} payload labels do not match their motion names"),
    )]
  }

  /// A visual the reader accepts only because the engine never reads the bytes it ends with.
  ///
  /// Reported rather than tolerated silently: the file is malformed, and this is the one place a modder learns so
  /// before a patch normalizes it away. Nested children never carry residue, which is a property of the byte stream, so
  /// this yields nothing for them.
  fn verify_mesh_residue_findings(ogf: &OgfFile, mesh_path: Option<&str>) -> Vec<Finding> {
    let Some(residue) = &ogf.residue else {
      return Vec::new();
    };

    let ignored: usize = residue.bytes.len()
      + ogf
        .kinematics
        .as_ref()
        .map_or(0, |kinematics| kinematics.trailing.len());

    let detail: String = match &residue.cause {
      OgfResidueCause::TrailingFragment => String::from("too few to be another chunk header"),
      OgfResidueCause::SplitMotionRef { path } => {
        format!("an uncounted motion reference '{path}' split across the declared bounds of the motion refs chunk")
      }
    };

    vec![Self::new_mesh_finding(
      GamedataVerificationRule::MeshesChunkResidue,
      mesh_path,
      format!("Mesh carries {ignored} bytes the engine never reads, {detail}"),
    )]
  }

  fn new_mesh_finding(rule: GamedataVerificationRule, mesh_path: Option<&str>, message: String) -> Finding {
    match mesh_path {
      Some(path) => GamedataFindingFactory::for_asset(rule, path, message),
      None => GamedataFindingFactory::without_asset(rule, message),
    }
  }

  fn new_motion_finding(rule: GamedataVerificationRule, motion_path: Option<&str>, message: String) -> Finding {
    match motion_path {
      Some(path) => GamedataFindingFactory::for_asset(rule, path, message),
      None => GamedataFindingFactory::without_asset(rule, message),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::MeshAssetsVerifier;

  #[test]
  fn accepts_a_connected_skeleton_with_one_root() {
    let findings: Vec<String> =
      MeshAssetsVerifier::skeleton_topology_findings([("root", ""), ("spine", "root"), ("head", "spine")]);

    assert!(findings.is_empty());
  }

  #[test]
  fn reports_invalid_skeleton_topology() {
    let findings: Vec<String> = MeshAssetsVerifier::skeleton_topology_findings([
      ("root", ""),
      ("arm", "missing"),
      ("arm", "root"),
      ("leg", "foot"),
      ("foot", "leg"),
    ]);

    assert!(
      findings
        .iter()
        .any(|finding| finding == "Mesh skeleton contains duplicate bone name 'arm'")
    );
    assert!(
      findings
        .iter()
        .any(|finding| finding == "Mesh skeleton bone 'arm' references missing parent 'missing'")
    );
    assert!(
      findings
        .iter()
        .any(|finding| finding == "Mesh skeleton contains parent cycle: foot -> leg -> foot")
    );
  }
}
