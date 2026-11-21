use std::collections::HashMap;

use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_level::{BuildPrjMuModel, BuildPrjMuReference};

use crate::level::build::level_build_geometry::LevelBuildGeometry;
use crate::level::build::level_build_impostors::LevelBuildImpostors;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::LevelBuildTables;
use crate::level::build::level_build_textures::LevelBuildTextures;

/// A build's multi-use objects, `SceneBuilder::BuildMUObject`: each object's model, built once in its own space, its
/// impostor where it has one, and a reference per placement.
#[derive(Default)]
pub(crate) struct LevelBuildModels {
  pub(crate) models: Vec<BuildPrjMuModel>,
  /// Each model's index by the object it was built of, as a level places one object many times.
  model_indexes: HashMap<String, usize>,
  pub(crate) references: Vec<BuildPrjMuReference>,
  pub(crate) impostors: LevelBuildImpostors,
}

impl LevelBuildModels {
  /// Adds a reference placing `object` by `placed`'s transform in the sector of its first mesh. The first placement
  /// of a name also builds the model in that sector, after its impostor where it is flagged `eoUsingLOD`.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, when the model does not build, or it has an impostor and `textures` names
  /// no root or the impostor does not build ([`LevelBuildImpostors::add`]).
  pub(crate) fn add(
    &mut self,
    placed: &EditableSceneObject,
    object: &EditableObjectFile,
    sectors: &LevelBuildSectors,
    tables: &mut LevelBuildTables,
    game_materials: &GameMtlFile,
    textures: &LevelBuildTextures<'_>,
  ) -> XrfResult {
    let sector: u16 = LevelBuildGeometry::get_model_sector(placed, object, sectors);
    let model: usize = match self.model_indexes.get(&placed.reference) {
      Some(model) => *model,
      None => {
        let lod: u16 = if object.flags & EditableObjectFile::USING_LOD_FLAG == 0 {
          u16::MAX
        } else {
          if !textures.is_available() {
            return Err(XrfError::new_invalid_error(format!(
              "Multi-use object '{}' has an impostor, whose strips the Build merges into an atlas; name the textures \
               root",
              placed.reference
            )));
          }

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
        self
          .model_indexes
          .insert(placed.reference.clone(), self.models.len() - 1);
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
