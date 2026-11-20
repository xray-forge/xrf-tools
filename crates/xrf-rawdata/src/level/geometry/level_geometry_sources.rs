use std::collections::BTreeMap;

use xrf_chunk::ChunkDataSource;
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelFile, LevelGeomSource, LevelHomFile, LevelSomFile, LevelVisualsChunk};
use xrf_ogf::OgfFile;
use xrf_spawn::LevelSpawnFile;

use crate::level::geometry::level_collision_index::LevelCollisionIndex;
use crate::level::geometry::level_decal_index::LevelDecalIndex;
use crate::level::instance::level_instance_library::LevelInstanceLibrary;
use crate::level::trees::level_tree_impostor_atlas::LevelTreeImpostorAtlas;

/// What a level's geometry decompiles from; the optional files are those the level's directory may lack.
pub struct LevelGeometrySources<'a, D: ChunkDataSource> {
  pub name: &'a str,
  pub level: &'a LevelFile,
  pub visuals: &'a LevelVisualsChunk,
  pub geom: &'a LevelGeomSource<D>,
  pub collision: Option<&'a mut LevelCollisionIndex>,
  pub hom: Option<&'a LevelHomFile>,
  pub som: Option<&'a LevelSomFile>,
  /// The wall marks, whose copies in the static geometry are dropped, since the scene's `wallmark.part` holds them.
  pub decals: Option<&'a LevelDecalIndex>,
  /// The spawn list the Build wrote, whose ladders and breakables are folded back into objects.
  pub spawn: Option<&'a LevelSpawnFile>,
  /// The breakable parts the Build wrote beside it, by their paths in the level's directory, `meshes/brkbl#<N>.ogf`.
  pub breakable_parts: &'a BTreeMap<String, OgfFile>,
  /// The trees' impostor atlas, `level_lods.dds` and its normals, which the tree models' strips are cut from.
  pub impostors: Option<&'a LevelTreeImpostorAtlas>,
  /// The library objects looked for in the static geometry, each found placement placed by reference.
  pub library: Option<&'a LevelInstanceLibrary>,
  /// The game material library telling which placed library faces the Build cuts into ladders and breakables, used
  /// where surfaces are not inferred. With neither, no ladder or breakable folds into a placement.
  pub game_materials: Option<&'a GameMtlFile>,
}
