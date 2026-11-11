use std::collections::BTreeMap;
use std::path::PathBuf;

use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_level::{
  LevelCformFile, LevelFile, LevelGeomFile, LevelGeomSource, LevelHomFile, LevelSomFile, LevelWallmarksFile,
};
use xrf_ogf::OgfFile;
use xrf_shaders::ShaderCompilerFile;
use xrf_spawn::LevelSpawnFile;
use xrf_utils::format_path;

use crate::level::level_breakable_conversion::LevelBreakableConversion;
use crate::level::level_collision_index::LevelCollisionIndex;
use crate::level::level_decal_index::LevelDecalIndex;
use crate::level::level_entity_conversion::LevelEntityConversion;
use crate::level::level_geometry_conversion::{LevelGeometryConversion, LevelGeometryDecompile, LevelGeometrySources};
use crate::level::level_map_conversion::LevelMapConversion;
use crate::level::level_surface_inference::{LevelShaderPairings, LevelSurfaceInference, LevelSurfaces};
use crate::level::level_tree_impostor::LevelTreeImpostorAtlas;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_listing::LevelSceneListing;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::{
  LevelSceneGeometry, LevelSceneInference, LevelSceneRefusal, LevelSceneSource, LevelSceneSurfaceLibraries,
  LevelSceneSurfaces,
};
use crate::provenance::sidecar::Sidecar;

/// The level's geometry ([`LevelGeometryConversion`]): the objects of its static geometry, collision, occluders,
/// ladders and breakable parts, the scene objects placing them, and the sectors and portals, their surfaces dressed
/// from `level.cform` where the options ask it and the directory holds one. It runs first, so the sectors take the
/// first names of their prefix, and names the folded ladders' and breakables' packets in the draft, which the game step
/// leaves out of the spawn points.
pub(crate) struct LevelGeometryStep;

impl LevelSceneStep for LevelGeometryStep {
  const FILES: &'static [&'static str] = &[
    LevelFile::FILE_NAME,
    LevelGeomFile::FILE_NAME,
    LevelCformFile::FILE_NAME,
    LevelHomFile::FILE_NAME,
    LevelSomFile::FILE_NAME,
    LevelWallmarksFile::FILE_NAME,
    LevelEntityConversion::SPAWN_FILE_NAME,
    LevelTreeImpostorAtlas::COLOR_FILE_NAME,
    LevelTreeImpostorAtlas::NORMAL_FILE_NAME,
  ];

  const LISTINGS: &'static [LevelSceneListing] = &[LevelSceneListing {
    directory: LevelBreakableConversion::DIRECTORY,
    takes: LevelBreakableConversion::is_part_file,
  }];

  fn decompile<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    let Some(bundle) = sources.get_bundle::<T>()? else {
      return Ok(());
    };
    let Some(visuals) = &bundle.visuals else {
      return Ok(());
    };
    let Some(geom) = sources.take_read(LevelGeomFile::FILE_NAME, LevelGeomSource::open_from_bytes::<T>)? else {
      return Ok(());
    };
    let mut collision: Option<LevelCollisionIndex> = sources.take_read(LevelCformFile::FILE_NAME, |bytes| {
      LevelCformFile::read_geometry_from_bytes::<T>(bytes).map(LevelCollisionIndex::new)
    })?;
    let hom: Option<LevelHomFile> = sources.take_read(LevelHomFile::FILE_NAME, LevelHomFile::read_from_bytes::<T>)?;
    let som: Option<LevelSomFile> = sources.take_read(LevelSomFile::FILE_NAME, LevelSomFile::read_from_bytes::<T>)?;
    // Read, not taken: the companion step decompiles the wall marks into their part.
    let decals: Option<LevelDecalIndex> = sources
      .peek_read(LevelWallmarksFile::FILE_NAME, LevelWallmarksFile::read_from_bytes::<T>)?
      .map(|wallmarks| LevelDecalIndex::new(&wallmarks));
    // Read, not taken: the game step decompiles the entities the geometry does not stand for.
    let spawn: Option<LevelSpawnFile> = sources.peek_read(
      LevelEntityConversion::SPAWN_FILE_NAME,
      LevelSpawnFile::read_from_bytes::<T>,
    )?;
    let breakable_parts: BTreeMap<String, OgfFile> = Self::read_breakable_parts::<T>(sources, draft);
    let impostors: Option<LevelTreeImpostorAtlas> = Self::read_impostors(sources, draft);
    let mut surfaces: Option<(LevelSurfaceInference, LevelSceneSurfaceLibraries)> = match (options.surfaces, &collision)
    {
      (LevelSurfaces::Infer, Some(_)) => Some(Self::read_surface_inference::<T>(options)?),
      _ => None,
    };
    let mut geometry: LevelGeometryDecompile = LevelGeometryConversion::decompile::<T, _>(
      LevelGeometrySources {
        name: &options.name,
        level: &bundle.file,
        visuals,
        geom: &geom,
        collision: collision.as_mut(),
        hom: hom.as_ref(),
        som: som.as_ref(),
        decals: decals.as_ref(),
        spawn: spawn.as_ref(),
        breakable_parts: &breakable_parts,
        impostors: impostors.as_ref(),
      },
      surfaces.as_mut().map(|(inference, _)| inference),
      &mut draft.context,
    )?;

    // Each sector takes its part of the map `level.ltx` names for its number, which is its place in the scene.
    if let Some(map) = &draft.map {
      for (sector, number) in geometry.sectors.objects.iter_mut().zip(0u16..) {
        if let Some(map_index) = map.sector_maps.get(&number) {
          sector.map_index = *map_index;
        }
      }
    }

    let has_objects: bool = !geometry.objects.is_empty();

    draft.infer_if(has_objects, &[LevelSceneInference::SectorShells]);
    draft.infer_if(has_objects && surfaces.is_some(), &[LevelSceneInference::SurfaceDress]);
    draft.infer_if(
      geometry.census.collision_only_faces > 0,
      &[LevelSceneInference::CollisionObject],
    );
    draft.infer_if(
      has_objects && (hom.is_some() || som.is_some()),
      &[LevelSceneInference::Occluders],
    );
    draft.infer_if(geometry.census.ladders > 0, &[LevelSceneInference::Ladders]);
    draft.infer_if(geometry.census.tree_models > 0, &[LevelSceneInference::TreeModels]);
    draft.infer_if(geometry.census.breakables > 0, &[LevelSceneInference::BreakableParts]);

    if has_objects {
      draft.add(geometry.scene_objects.to_scene_tool()?, Self::FILES, sources);
    }

    // A sector names the shells of its geometry and the collision meshes of its faces; the occluders are in none.
    if !geometry.sectors.objects.is_empty() {
      draft.add(
        geometry.sectors.to_scene_tool()?,
        &[
          LevelFile::FILE_NAME,
          LevelGeomFile::FILE_NAME,
          LevelCformFile::FILE_NAME,
          LevelMapConversion::FILE_NAME,
        ],
        sources,
      );
    }

    if !geometry.portals.objects.is_empty() {
      draft.add(geometry.portals.to_scene_tool()?, &[LevelFile::FILE_NAME], sources);
    }

    draft.refused.extend(geometry.refused);
    draft.folded_entities = geometry.folded_entities;
    draft.geometry = Some(LevelSceneGeometry {
      objects: geometry.objects.iter().map(|object| object.reference.clone()).collect(),
      census: geometry.census,
      surfaces: surfaces.map(|(inference, libraries)| LevelSceneSurfaces {
        libraries,
        census: inference.into_census(),
      }),
    });
    draft.objects = geometry.objects;
    draft.textures = geometry.textures;

    Ok(())
  }
}

impl LevelGeometryStep {
  /// The trees' impostor atlas, where the directory holds both its files; one that does not read is refused, and the
  /// trees come back without impostors.
  fn read_impostors(sources: &mut LevelSceneSources, draft: &mut LevelSceneDraft) -> Option<LevelTreeImpostorAtlas> {
    let color: Option<Vec<u8>> = sources.take(LevelTreeImpostorAtlas::COLOR_FILE_NAME);
    let normal: Option<Vec<u8>> = sources.take(LevelTreeImpostorAtlas::NORMAL_FILE_NAME);

    LevelTreeImpostorAtlas::read_from_bytes(color?, normal?)
      .inspect_err(|error| {
        draft.refused.push(LevelSceneRefusal {
          file: String::from(LevelTreeImpostorAtlas::COLOR_FILE_NAME),
          object: String::from(LevelTreeImpostorAtlas::COLOR_FILE_NAME),
          reason: format!("The impostor atlas was not read, so the trees come back without impostors: {error}"),
        });
      })
      .ok()
  }

  /// The breakable parts the directory holds, by their paths in it; one that is no OGF is refused, and its breakable
  /// stays an entity.
  fn read_breakable_parts<T: ByteOrder>(
    sources: &mut LevelSceneSources,
    draft: &mut LevelSceneDraft,
  ) -> BTreeMap<String, OgfFile> {
    let mut parts: BTreeMap<String, OgfFile> = BTreeMap::new();

    for listing in Self::LISTINGS {
      for path in sources.get_listed(listing) {
        match sources.take_read(&path, OgfFile::read_from_bytes::<T>) {
          Ok(Some(part)) => {
            parts.insert(path, part);
          }
          Ok(None) => {}
          Err(error) => draft.refused.push(LevelSceneRefusal {
            object: path.clone(),
            file: path,
            reason: format!("The breakable part is no OGF, so its breakable stays an entity: {error}"),
          }),
        }
      }
    }

    parts
  }

  /// What dresses the surfaces: the libraries the options name, each recorded with its hash.
  fn read_surface_inference<T: ByteOrder>(
    options: &LevelDecompileOptions,
  ) -> XrfResult<(LevelSurfaceInference, LevelSceneSurfaceLibraries)> {
    let read = |path: Option<&PathBuf>, what: &str| -> XrfResult<(Vec<u8>, LevelSceneSource)> {
      let path: &PathBuf = path.ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Inferring surfaces needs the {what}; name it, or dress surfaces with the editor's defaults"
        ))
      })?;
      let bytes: Vec<u8> = std::fs::read(path)
        .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(path))))?;
      let record: LevelSceneSource = LevelSceneSource {
        file: format_path(path).to_string(),
        sha256: Sidecar::get_sha256_hex(&bytes),
      };

      Ok((bytes, record))
    };
    let (game_materials, game_materials_record) =
      read(options.game_materials.as_ref(), "game material library (gamemtl.xr)")?;
    let (compiler_shaders, compiler_shaders_record) = read(
      options.compiler_shaders.as_ref(),
      "compiler shader library (shaders_xrlc.xr)",
    )?;
    let pairings: LevelShaderPairings = LevelShaderPairings::read_from_roots::<T>(&options.libraries)?;
    let libraries: LevelSceneSurfaceLibraries = LevelSceneSurfaceLibraries {
      game_materials: game_materials_record,
      compiler_shaders: compiler_shaders_record,
      object_roots: options
        .libraries
        .iter()
        .map(|root| format_path(root).to_string())
        .collect(),
      unread_objects: pairings.unread_objects,
    };

    Ok((
      LevelSurfaceInference::new(
        GameMtlFile::read_from_bytes::<T>(game_materials)?,
        ShaderCompilerFile::read_from_bytes::<T>(compiler_shaders)?,
        pairings,
      ),
      libraries,
    ))
  }
}
