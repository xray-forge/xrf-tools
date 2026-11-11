use std::path::Path;

use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_level::{BuildPrjMuModel, BuildPrjMuReference};

use crate::level::build::level_build_geometry::LevelBuildGeometry;
use crate::level::build::level_build_impostors::LevelBuildImpostors;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::LevelBuildTables;

/// A build's multi-use objects (`SceneBuilder::BuildMUObject`, `BuilderRemote.cpp:837`): each object's model, built
/// once in its own space, its impostor where it has one, and a reference per placement.
#[derive(Default)]
pub(crate) struct LevelBuildModels {
  pub(crate) models: Vec<BuildPrjMuModel>,
  pub(crate) references: Vec<BuildPrjMuReference>,
  pub(crate) impostors: LevelBuildImpostors,
}

impl LevelBuildModels {
  /// The reference placing `object` by `placed`'s transform in the sector of its first mesh; the model of `object`
  /// built first where none of its name is held, in its own space and that first placement's sector, after its
  /// impostor where it is flagged `eoUsingLOD`, whose strips are read under `textures`.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, when the model does not build, or it has an impostor and `textures` names
  /// no root or its impostor does not build ([`LevelBuildImpostors::add`]).
  pub(crate) fn add(
    &mut self,
    placed: &EditableSceneObject,
    object: &EditableObjectFile,
    sectors: &LevelBuildSectors,
    tables: &mut LevelBuildTables,
    game_materials: &GameMtlFile,
    textures: Option<&Path>,
  ) -> XrfResult {
    let sector: u16 = LevelBuildGeometry::get_model_sector(placed, object, sectors);
    let model: usize = match self.models.iter().position(|model| model.name == placed.reference) {
      Some(model) => model,
      None => {
        let lod: u16 = if object.flags & EditableObjectFile::USING_LOD_FLAG == 0 {
          u16::MAX
        } else {
          let textures: &Path = textures.ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Multi-use object '{}' has an impostor, whose strips the Build merges into an atlas; name the textures \
               root",
              placed.reference
            ))
          })?;

          self
            .impostors
            .add(&placed.reference, object, sector, tables, textures)?
        };
        let mut geometry: LevelBuildGeometry = LevelBuildGeometry::default();

        geometry.add_model(placed, object, sectors, tables, game_materials)?;
        self.models.push(BuildPrjMuModel {
          name: placed.reference.clone(),
          vertices: geometry.vertices,
          faces: geometry.faces,
          lod,
          smoothing_groups: Some(geometry.smoothing_groups),
        });
        self.models.len() - 1
      }
    };

    self.references.push(BuildPrjMuReference {
      model: u32::try_from(model)
        .map_err(|_| XrfError::new_invalid_error("A build holds more multi-use models than its u32 indices number"))?,
      transform: placed.object.transform.to_matrix(),
      flags: 0,
      sector,
      sector_padding: 0,
      reserved: [0; 8],
    });

    Ok(())
  }
}
