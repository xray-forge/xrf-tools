use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::{EditableObjectFile, EditableSurface};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelHomFile, LevelHomPolygon, LevelSomFile, LevelSomPolygon};
use xrf_math::{Matrix4x4, Vector3d};

use crate::level::level_occluder_conversion::LevelOccluderConversion;

/// The occluders a build writes beside `build.prj`, from every placed object whatever kind it is:
/// `SceneBuilder::BuildHOMModel` (`BuilderOGF.cpp:17`) takes each face of an object flagged `eoHOM` and each face of a
/// `materials\occ` surface of any other, and `BuildSOMModel` (`:95`) each face of an object flagged `eoSoundOccluder`
/// with its game material's sound occlusion factor. Faces go in placed, in each mesh's order of surfaces.
#[derive(Default)]
pub(crate) struct LevelBuildOccluders {
  hom: Vec<LevelHomPolygon>,
  som: Vec<LevelSomPolygon>,
}

impl LevelBuildOccluders {
  /// Adds the occluding faces of `object`, placed by `placed`.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, for a face naming a surface or point its object lacks, or a sound occluder's
  /// surface of a game material `game_materials` does not hold.
  pub(crate) fn add_object(
    &mut self,
    placed: &EditableSceneObject,
    object: &EditableObjectFile,
    game_materials: &GameMtlFile,
  ) -> XrfResult {
    let is_hom: bool = object.flags & EditableObjectFile::HOM_FLAG != 0;
    let is_som: bool = object.flags & EditableObjectFile::SOUND_OCCLUDER_FLAG != 0;
    let matrix: Matrix4x4 = placed.object.transform.to_matrix();
    let surfaces: &[EditableSurface] = object.surfaces.as_deref().unwrap_or_default();

    for mesh in object.meshes.iter().flatten() {
      for surface_faces in &mesh.surface_faces {
        let surface: &EditableSurface = surfaces
          .iter()
          .find(|surface| surface.name == surface_faces.surface)
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Object '{}' mesh '{}' names surface '{}' it does not hold",
              placed.reference, mesh.name, surface_faces.surface
            ))
          })?;
        let is_occluder: bool = is_hom || surface.game_material == LevelOccluderConversion::OCCLUDER_GAME_MATERIAL;
        let occlusion: Option<f32> = if is_som {
          Some(
            game_materials
              .find_material_by_name(&surface.game_material)
              .map(|material| material.sound_occlusion_factor)
              .ok_or_else(|| {
                XrfError::new_invalid_error(format!(
                  "{} Object '{}', surface '{}' contain undefined game material.",
                  mesh.name, placed.reference, surface.name
                ))
              })?,
          )
        } else {
          None
        };

        if !is_occluder && occlusion.is_none() {
          continue;
        }

        let is_two_sided: u32 = u32::from(surface.is_two_sided());

        for face in &surface_faces.faces {
          let corners = &mesh
            .faces
            .get(*face as usize)
            .ok_or_else(|| XrfError::new_invalid_error(format!("Mesh '{}' names face {face}", mesh.name)))?
            .corners;
          let mut vertices: [Vector3d; 3] = [Vector3d::ZERO; 3];

          for (vertex, corner) in vertices.iter_mut().zip(corners) {
            *vertex = matrix.transform_point(mesh.points.get(corner.point as usize).ok_or_else(|| {
              XrfError::new_invalid_error(format!("Mesh '{}' face {face} names a point it lacks", mesh.name))
            })?);
          }

          if is_occluder {
            self.hom.push(LevelHomPolygon {
              vertices,
              flags: is_two_sided,
            });
          }

          if let Some(occlusion) = occlusion {
            self.som.push(LevelSomPolygon {
              vertices,
              is_two_sided,
              occlusion,
            });
          }
        }
      }
    }

    Ok(())
  }

  /// `level.hom` and `level.som`, each where a face went into it: the Build writes neither file of no polygon.
  pub(crate) fn into_files(self) -> (Option<LevelHomFile>, Option<LevelSomFile>) {
    (
      (!self.hom.is_empty()).then_some(LevelHomFile {
        version: LevelHomFile::CURRENT_VERSION,
        polygons: self.hom,
      }),
      (!self.som.is_empty()).then_some(LevelSomFile {
        version: LevelSomFile::CURRENT_VERSION,
        polygons: self.som,
      }),
    )
  }
}
