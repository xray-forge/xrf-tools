use xrf_chunk::{ChunkLayout, XRayByteOrder};
use xrf_editable_object::{EditableBone, EditableDescription, EditableMotion, EditableObjectFile, EditableTimeWidth};
use xrf_error::{XrfError, XrfResult};
use xrf_level::DetailModel;
use xrf_ogf::{OgfDescriptionChunk, OgfDescriptionTimeWidth, OgfFile, OgfModelType};

use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::compile::object_compile_options::TangentMethod;
use crate::object::decompile::geometry::repeated_faces::RepeatedFaces;
use crate::object::decompile::geometry::smoothing_inference::SmoothingInference;
use crate::object::decompile::geometry::source::ogf_surface::OgfSurface;
use crate::object::decompile::geometry::two_sided_pairs::TwoSidedPairs;
use crate::object::decompile::geometry::welded_mesh::WeldedMesh;
use crate::object::decompile::mesh::assembled_mesh::AssembledMesh;
use crate::object::decompile::mesh::mesh_assembler::MeshAssembler;
use crate::object::decompile::mesh::mesh_kind::MeshKind;
use crate::object::decompile::object_decompile::ObjectDecompile;
use crate::object::decompile::object_inference::ObjectInference;
use crate::object::decompile::skeleton::bone_converter::BoneConverter;
use crate::object::decompile::skeleton::motion_converter::{ConvertedMotions, MotionConverter};
use crate::object::detail_parameters::DetailParameters;

/// Turns an OGF the Actor Editor exported back into the editable object it was exported from, as near as the OGF
/// allows: see the crate's documentation for what is recovered and what inferred.
pub struct ObjectDecompiler;

impl ObjectDecompiler {
  /// What every decompiled object infers, since no OGF stores it.
  const INFERRED: [ObjectInference; 6] = [
    ObjectInference::SurfaceNames,
    ObjectInference::SurfaceCompilerShaders,
    ObjectInference::SurfaceGameMaterials,
    ObjectInference::MeshName,
    ObjectInference::Smoothing,
    ObjectInference::TwoSidedSurfaces,
  ];

  /// Decompiles an OGF the Actor Editor exported: a skeleton, its vertices following one bone each or blended across up
  /// to four, or a static visual (`MT_NORMAL`, `MT_PROGRESSIVE`, or `MT_HIERRARHY` holding them), which comes back as
  /// a static object without bones.
  ///
  /// # Errors
  ///
  /// Returns an error for an OGF of another model type, which no editable object exports to, and for one whose bones,
  /// IK data, vertex layouts or indices do not hold together.
  pub fn decompile(file: &OgfFile) -> XrfResult<ObjectDecompile> {
    match OgfModelType::from_raw(file.header.model_type) {
      Some(OgfModelType::SkeletonRigid | OgfModelType::SkeletonAnim) => Self::decompile_skeleton(file),
      Some(OgfModelType::Normal | OgfModelType::Progressive | OgfModelType::Hierarchy) => Self::decompile_static(file),
      other => Err(XrfError::new_not_implemented_error(format!(
        "An OGF of model type {} is not decompiled; only skeletons and static visuals are",
        other.map_or_else(
          || OgfModelType::label(file.header.model_type),
          |it| String::from(it.name())
        )
      ))),
    }
  }

  /// Decompiles visuals of a compiled level into one static object: the visuals drawn with one texture and shader and
  /// dressed alike are one surface, and every triangle is welded into one mesh, so faces of different visuals that met
  /// in the source meet again. Level normals are stored a byte a component, which the weld, the two-sided folding and
  /// the smoothing allow one quantisation step for.
  ///
  /// # Errors
  ///
  /// Returns an error for no visual, or one whose indices reach past its vertices.
  pub fn decompile_level_geometry(geometry: &[LevelVisualGeometry]) -> XrfResult<ObjectDecompile> {
    let mut surfaces: Vec<OgfSurface> = Vec::new();

    for visual in geometry {
      let surface: OgfSurface = OgfSurface::of_level(visual)?;

      match surfaces
        .iter_mut()
        .find(|it| it.texture == surface.texture && it.shader == surface.shader && it.dress == surface.dress)
      {
        Some(existing) => existing.triangles.extend(surface.triangles),
        None => surfaces.push(surface),
      }
    }

    // The object is static and compiles without tangents, so the method recorded for its compile is moot.
    Ok(Self::assemble_static(
      Self::assemble(&surfaces, MeshKind::Static)?,
      0,
      None,
      TangentMethod::ActorEditor,
    ))
  }

  /// Decompiles a detail model (`.dm`, `CDetail`) into a static object of one mesh and one surface, as `EDetail::Update`
  /// reads one: its shader and texture, and its triangles over vertices of a position and a texture coordinate. Its
  /// flags and scale range, which the object cannot hold, come back in [`ObjectDecompile::detail`]. A model stores no
  /// normal and no mirrored copy, so every edge comes back smooth and no surface two-sided.
  ///
  /// # Errors
  ///
  /// Returns an error for a model of no triangle, or an index past its vertices.
  pub fn decompile_detail(model: &DetailModel) -> XrfResult<ObjectDecompile> {
    let surface: OgfSurface = OgfSurface::of_detail(model)?;
    // The object is static and compiles without tangents, so the method recorded for its compile is moot.
    let decompile: ObjectDecompile = Self::assemble_static(
      Self::assemble(std::slice::from_ref(&surface), MeshKind::Static)?,
      0,
      None,
      TangentMethod::ActorEditor,
    );

    Ok(ObjectDecompile {
      detail: Some(DetailParameters {
        flags: model.flags,
        min_scale: model.min_scale,
        max_scale: model.max_scale,
      }),
      ..decompile
    })
  }

  /// A static visual's object: its children's surfaces in one mesh, no bones.
  fn decompile_static(file: &OgfFile) -> XrfResult<ObjectDecompile> {
    let surfaces: Vec<OgfSurface> = OgfSurface::collect(file)?;

    Ok(Self::assemble_static(
      Self::assemble(&surfaces, MeshKind::Static)?,
      if Self::is_progressive(file) {
        EditableObjectFile::PROGRESSIVE_FLAG
      } else {
        0
      },
      file.get_description().map(Self::copy_description),
      Self::get_tangent_method(file),
    ))
  }

  /// A static object of `assembled`'s mesh and surfaces, with `flags` and `description`, and no bones.
  fn assemble_static(
    assembled: AssembledMesh,
    flags: u32,
    description: Option<EditableDescription>,
    tangents: TangentMethod,
  ) -> ObjectDecompile {
    ObjectDecompile {
      object: EditableObjectFile {
        flags,
        flags_size: EditableObjectFile::FLAGS_SIZE,
        class_script: None,
        lods: None,
        meshes: Some(vec![assembled.mesh]),
        surfaces: Some(assembled.surfaces),
        bones: None,
        embedded_motions: None,
        motion_references: None,
        bone_parts: None,
        actor_transform: None,
        description,
        layout: ChunkLayout::default(),
      },
      inferred: Self::INFERRED.to_vec(),
      tangents,
      detail: None,
    }
  }

  /// `surfaces` welded into one mesh of `kind`, faces written again dropped, mirrored copies folded into two-sided
  /// faces and smoothing inferred.
  fn assemble(surfaces: &[OgfSurface], kind: MeshKind<'_>) -> XrfResult<AssembledMesh> {
    let mut welded: WeldedMesh = WeldedMesh::weld(surfaces)?;

    welded.faces = TwoSidedPairs::fold(RepeatedFaces::drop_repeats(std::mem::take(&mut welded.faces)));

    let smoothing: Vec<u32> = SmoothingInference::infer(&welded.faces);

    MeshAssembler::assemble(surfaces, &welded, smoothing, kind)
  }

  /// A skeleton's object: its children's surfaces in one skinned mesh, its bones and its motions.
  fn decompile_skeleton(file: &OgfFile) -> XrfResult<ObjectDecompile> {
    let surfaces: Vec<OgfSurface> = OgfSurface::collect(file)?;
    let bones: Vec<EditableBone> = BoneConverter::convert(file)?;
    let bone_names: Vec<String> = bones.iter().map(|bone| bone.definition.name.clone()).collect();
    let assembled: AssembledMesh = Self::assemble(
      &surfaces,
      MeshKind::Skinned {
        bone_names: &bone_names,
      },
    )?;
    let motions: Option<ConvertedMotions> = MotionConverter::convert::<XRayByteOrder>(file, &bone_names)?;
    let mut inferred: Vec<ObjectInference> = Self::INFERRED.to_vec();

    inferred.push(ObjectInference::BoneLengths);

    if let Some(motions) = &motions {
      inferred.push(ObjectInference::MotionCurves);

      if motions.is_lengthened {
        inferred.push(ObjectInference::MotionLengths);
      }
    }

    inferred.sort_unstable();

    Ok(ObjectDecompile {
      object: EditableObjectFile {
        flags: EditableObjectFile::DYNAMIC_FLAG
          | if Self::is_progressive(file) {
            EditableObjectFile::PROGRESSIVE_FLAG
          } else {
            0
          },
        flags_size: EditableObjectFile::FLAGS_SIZE,
        class_script: file.user_data.as_ref().map(|chunk| Self::trim_text(&chunk.user_data)),
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
        description: file.get_description().map(Self::copy_description),
        layout: ChunkLayout::default(),
      },
      inferred,
      tangents: Self::get_tangent_method(file),
      detail: None,
    })
  }

  /// The tangent method `file`'s basis was made with: MikkTSpace where its description's convertor names Blender, whose
  /// exports take Blender's tangents, else the Actor Editor's. On the models naming `blender-xray` MikkTSpace gives the
  /// stored tangent back within a degree on 88 to 97% of vertices (enhanced editions, Gunslinger) and 65% (GAMMA, 3,565
  /// models), the editor's pass on under 42%; on GAMMA's 669 naming plain `blender`, 76% against 39%. On every other
  /// model the editor's pass does better.
  fn get_tangent_method(file: &OgfFile) -> TangentMethod {
    let is_blender: bool = file
      .get_description()
      .is_some_and(|description| description.convertor.to_ascii_lowercase().contains("blender"));

    if is_blender {
      TangentMethod::MikkTSpace
    } else {
      TangentMethod::ActorEditor
    }
  }

  /// The text of a string chunk the OGF keeps whole, up to the terminator the export wrote: the object stores it as a
  /// string of its own, terminated again on write.
  pub(crate) fn trim_text(stored: &str) -> String {
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
  fn copy_description(description: &OgfDescriptionChunk) -> EditableDescription {
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
