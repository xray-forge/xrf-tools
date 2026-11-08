use xrf_chunk::{ChunkLayout, XRayByteOrder};
use xrf_editable_object::{EditableBone, EditableDescription, EditableMotion, EditableObjectFile, EditableTimeWidth};
use xrf_error::{XrfError, XrfResult};
use xrf_ogf::{OgfDescriptionChunk, OgfDescriptionTimeWidth, OgfFile, OgfModelType};

use crate::object::decompile::geometry::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::smoothing_inference::SmoothingInference;
use crate::object::decompile::geometry::two_sided_pairs::TwoSidedPairs;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::mesh::mesh_assembler::MeshAssembler;
use crate::object::decompile::object_decompile::ObjectDecompile;
use crate::object::decompile::object_inference::ObjectInference;
use crate::object::decompile::skeleton::bone_converter::BoneConverter;
use crate::object::decompile::skeleton::motion_converter::{ConvertedMotions, MotionConverter};

/// Turns an OGF the Actor Editor exported back into the editable object it was exported from, as near as the OGF
/// allows: see the crate's documentation for what is recovered and what inferred.
pub struct ObjectDecompiler;

impl ObjectDecompiler {
  /// What every decompiled object infers, since no OGF stores it.
  const INFERRED: [ObjectInference; 7] = [
    ObjectInference::SurfaceNames,
    ObjectInference::SurfaceCompilerShaders,
    ObjectInference::SurfaceGameMaterials,
    ObjectInference::MeshName,
    ObjectInference::BoneLengths,
    ObjectInference::Smoothing,
    ObjectInference::TwoSidedSurfaces,
  ];

  /// Decompiles a skeleton OGF, its vertices following one bone each or blended across up to four.
  ///
  /// # Errors
  ///
  /// Returns an error for an OGF that is not a skeleton or has static vertices, which are not decompiled yet, or whose
  /// bones, IK data or indices do not hold together.
  pub fn decompile(file: &OgfFile) -> XrfResult<ObjectDecompile> {
    match OgfModelType::from_raw(file.header.model_type) {
      Some(OgfModelType::SkeletonRigid | OgfModelType::SkeletonAnim) => {}
      other => {
        return Err(XrfError::new_not_implemented_error(format!(
          "An OGF of model type {} is not decompiled yet; only skeletons are",
          other.map_or_else(
            || OgfModelType::label(file.header.model_type),
            |it| String::from(it.name())
          )
        )));
      }
    }

    let surfaces: Vec<OgfSurface> = OgfSurface::collect(file)?;
    let mut welded: WeldedMesh = WeldedMesh::weld(&surfaces)?;

    welded.faces = TwoSidedPairs::fold(std::mem::take(&mut welded.faces));

    let smoothing: Vec<u32> = SmoothingInference::infer(&welded.faces);
    let bones: Vec<EditableBone> = BoneConverter::convert(file)?;
    let bone_names: Vec<String> = bones.iter().map(|bone| bone.definition.name.clone()).collect();
    let assembled: AssembledMesh = MeshAssembler::assemble(&surfaces, &welded, smoothing, &bone_names)?;
    let motions: Option<ConvertedMotions> = MotionConverter::convert::<XRayByteOrder>(file, &bone_names)?;
    let mut inferred: Vec<ObjectInference> = Self::INFERRED.to_vec();

    if let Some(motions) = &motions {
      inferred.push(ObjectInference::MotionCurves);

      if motions.is_lengthened {
        inferred.push(ObjectInference::MotionLengths);
      }
    }

    Ok(ObjectDecompile {
      object: EditableObjectFile {
        flags: EditableObjectFile::DYNAMIC_FLAG
          | if Self::is_progressive(file) {
            EditableObjectFile::PROGRESSIVE_FLAG
          } else {
            0
          },
        flags_size: EditableObjectFile::FLAGS_SIZE,
        class_script: file.user_data.as_ref().map(|chunk| Self::to_text(&chunk.user_data)),
        // Exported as a line (`w_string`), so its first line, as the engine reads it.
        lods: file.lods.as_ref().map(|chunk| String::from(chunk.get_visual_name())),
        meshes: Some(vec![assembled.mesh]),
        surfaces: Some(assembled.surfaces),
        bones: Some(bones),
        embedded_motions: motions
          .as_ref()
          .map(|it| EditableMotion::write_list::<XRayByteOrder>(&it.motions))
          .transpose()?,
        motion_references: file
          .kinematics
          .as_ref()
          .map(|chunk| chunk.motion_refs.clone())
          .filter(|references| !references.is_empty()),
        bone_parts: motions.map(|it| it.parts),
        actor_transform: None,
        description: file.get_description().map(Self::to_description),
        layout: ChunkLayout::default(),
      },
      inferred,
    })
  }

  /// The text of a string chunk the OGF keeps whole, up to the terminator the export wrote: the object stores it as a
  /// string of its own, terminated again on write.
  pub(crate) fn to_text(stored: &str) -> String {
    String::from(stored.split('\0').next().unwrap_or_default())
  }

  /// Whether any of the OGF's children carries sliding windows, which the export builds only for a progressive
  /// object.
  fn is_progressive(file: &OgfFile) -> bool {
    file.swi_data.as_ref().is_some_and(|swi| !swi.windows.is_empty())
      || file
        .children
        .iter()
        .flat_map(|children| &children.nested)
        .any(Self::is_progressive)
  }

  /// The object's own description, which the OGF's description copied.
  fn to_description(description: &OgfDescriptionChunk) -> EditableDescription {
    EditableDescription {
      creator: description.creator.clone(),
      created_at: description.created_at,
      modifier: description.editor.clone(),
      modified_at: description.edited_at,
      time_width: match description.time_width {
        OgfDescriptionTimeWidth::Narrow => EditableTimeWidth::Seconds32,
        OgfDescriptionTimeWidth::Wide => EditableTimeWidth::Seconds64,
      },
    }
  }
}
