use std::collections::BTreeMap;

use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_level::{
  LevelCformFile, LevelFile, LevelGeomFile, LevelGeomSource, LevelHomFile, LevelSomFile, LevelWallmarksFile,
};
use xrf_ogf::OgfFile;
use xrf_shaders::ShaderCompilerFile;
use xrf_utils::format_path;

use crate::level::details::level_detail_library::LevelDetailLibrary;
use crate::level::entities::level_entity_conversion::LevelEntityConversion;
use crate::level::geometry::level_collision_index::LevelCollisionIndex;
use crate::level::geometry::level_decal_index::LevelDecalIndex;
use crate::level::geometry::level_geometry_conversion::LevelGeometryConversion;
use crate::level::geometry::level_geometry_decompile::LevelGeometryDecompile;
use crate::level::geometry::level_geometry_sources::LevelGeometrySources;
use crate::level::instance::level_instance_library::LevelInstanceLibrary;
use crate::level::instance::level_instance_placement::LevelInstancePlacement;
use crate::level::instance::level_tree_match::LevelTreeMatch;
use crate::level::level_object_library::LevelObjectLibrary;
use crate::level::parts::level_breakable_conversion::LevelBreakableConversion;
use crate::level::parts::level_map_conversion::LevelMapConversion;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_library_file::LevelSceneLibraryFile;
use crate::level::scene::level_scene_listing::LevelSceneListing;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_geometry_libraries::LevelGeometryLibraries;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::level::surfaces::level_shader_pairings::LevelShaderPairings;
use crate::level::surfaces::level_surface_inference::LevelSurfaceInference;
use crate::level::trees::level_tree_impostor_atlas::LevelTreeImpostorAtlas;
use crate::provenance::level_scene_geometry::LevelSceneGeometry;
use crate::provenance::level_scene_inference::LevelSceneInference;
use crate::provenance::level_scene_instance_placement::LevelSceneInstancePlacement;
use crate::provenance::level_scene_instances::LevelSceneInstances;
use crate::provenance::level_scene_library_object::LevelSceneLibraryObject;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;
use crate::provenance::level_scene_surfaces::LevelSceneSurfaces;
use crate::provenance::level_scene_tree_match::LevelSceneTreeMatch;
use crate::provenance::sidecar::Sidecar;

/// The level's geometry ([`LevelGeometryConversion`]): its objects, the scene objects placing them, and the sectors and
/// portals.
///
/// It runs right after the map step, so the sectors take the first names of their prefix, and records the folded
/// ladders' and breakables' packets in the draft for the game step to leave out.
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

  fn decompile(
    sources: &mut LevelSceneSources,
    options: &LevelDecompileOptions,
    draft: &mut LevelSceneDraft,
  ) -> XrfResult {
    let Some(bundle) = sources.get_bundle()? else {
      return Ok(());
    };
    let Some(visuals) = &bundle.visuals else {
      return Ok(());
    };
    let Some(geom) = sources.take_read(LevelGeomFile::FILE_NAME, LevelGeomSource::open_from_bytes)? else {
      return Ok(());
    };
    let mut collision: Option<LevelCollisionIndex> = sources.take_read(LevelCformFile::FILE_NAME, |bytes| {
      LevelCformFile::read_geometry_from_bytes(bytes).map(LevelCollisionIndex::new)
    })?;
    let hom: Option<LevelHomFile> = sources.take_read(LevelHomFile::FILE_NAME, LevelHomFile::read_from_bytes)?;
    let som: Option<LevelSomFile> = sources.take_read(LevelSomFile::FILE_NAME, LevelSomFile::read_from_bytes)?;
    // Read, not taken: the companion step decompiles the wall marks into their part.
    let decals: Option<LevelDecalIndex> = sources
      .peek_read(LevelWallmarksFile::FILE_NAME, LevelWallmarksFile::read_from_bytes)?
      .map(|wallmarks| LevelDecalIndex::new(&wallmarks));
    let breakable_parts: BTreeMap<String, OgfFile> = Self::read_breakable_parts(sources, draft);
    let impostors: Option<LevelTreeImpostorAtlas> = Self::read_impostors(sources, draft);
    let libraries: LevelGeometryLibraries = LevelGeometryLibraries::read(options, collision.is_some())?;
    let (pairings, library, details) = Self::read_object_library(
      options,
      &mut draft.object_library,
      &draft.job,
      libraries.get_compiler_shaders(),
      libraries.inferred.is_some(),
    )?;

    draft.detail_library = Some(details);

    let LevelGeometryLibraries {
      inferred,
      game_materials,
      record: libraries,
      ..
    } = libraries;
    let mut surfaces: Option<LevelSurfaceInference> =
      inferred
        .zip(pairings)
        .map(|((game_materials, compiler_shaders), pairings)| {
          LevelSurfaceInference::new(game_materials, compiler_shaders, pairings)
        });
    let mut geometry: LevelGeometryDecompile = LevelGeometryConversion::decompile(
      LevelGeometrySources {
        name: &options.name,
        level: &bundle.file,
        visuals,
        geom: &geom,
        collision: collision.as_mut(),
        hom: hom.as_ref(),
        som: som.as_ref(),
        decals: decals.as_ref(),
        spawn: draft.spawn.as_ref(),
        breakable_parts: &breakable_parts,
        impostors: impostors.as_ref(),
        library: library.as_ref(),
        game_materials: game_materials.as_ref(),
      },
      surfaces.as_mut(),
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

    draft.infer_if(
      geometry.census.instances + geometry.census.library_tree_models > 0,
      &[LevelSceneInference::LibraryInstances],
    );

    let instances: Option<LevelSceneInstances> = if library.is_some() {
      let (files, instances) = Self::copy_library(options, &geometry.instances, &geometry.trees)?;

      draft.library = files;
      Some(instances)
    } else {
      None
    };

    draft.refused.extend(geometry.refused);
    draft.folded_entities = geometry.folded_entities;
    draft.geometry = Some(LevelSceneGeometry {
      objects: geometry.objects.iter().map(|object| object.reference.clone()).collect(),
      census: geometry.census,
      libraries,
      surfaces: surfaces.map(|inference| LevelSceneSurfaces {
        object_roots: options
          .libraries
          .iter()
          .map(|root| format_path(root).to_string())
          .collect(),
        census: inference.into_census(),
      }),
      instances,
    });
    draft.objects = geometry.objects;
    draft.textures = geometry.textures;

    Ok(())
  }
}

impl LevelGeometryStep {
  /// Each placed library object read with its thumbnail for copying beside the scene, and the sidecar's record of them.
  ///
  /// # Errors
  ///
  /// Returns an error when a placed object's file cannot be read again.
  fn copy_library(
    options: &LevelDecompileOptions,
    placements: &[LevelInstancePlacement],
    trees: &[LevelTreeMatch],
  ) -> XrfResult<(Vec<LevelSceneLibraryFile>, LevelSceneInstances)> {
    let mut files: Vec<LevelSceneLibraryFile> = Vec::new();
    let mut objects: Vec<LevelSceneLibraryObject> = Vec::new();
    let placed = placements
      .iter()
      .map(|placement| (&placement.reference, &placement.path))
      .chain(trees.iter().map(|tree| (&tree.reference, &tree.path)));

    for (reference, path) in placed {
      if files.iter().any(|file| file.reference == *reference) {
        continue;
      }

      let object: Vec<u8> = std::fs::read(path)
        .map_err(|error| XrfError::new_read_error(format!("{} was not read: {error}", format_path(path))))?;
      let thumbnail: Option<Vec<u8>> =
        std::fs::read(path.with_extension(LevelSceneLibraryFile::THUMBNAIL_EXTENSION)).ok();

      objects.push(LevelSceneLibraryObject {
        reference: reference.clone(),
        source: format_path(path).to_string(),
        sha256: Sidecar::get_sha256_hex(&object),
      });
      files.push(LevelSceneLibraryFile {
        reference: reference.clone(),
        object,
        thumbnail,
      });
    }

    let instances: LevelSceneInstances = LevelSceneInstances {
      roots: options
        .libraries
        .iter()
        .map(|root| format_path(root).to_string())
        .collect(),
      objects,
      placements: placements
        .iter()
        .map(|placement| LevelSceneInstancePlacement {
          reference: placement.reference.clone(),
          object: placement.object.clone(),
          coverage: placement.coverage,
          error: placement.error,
          triangles: placement.triangles,
        })
        .collect(),
      trees: trees
        .iter()
        .map(|tree| LevelSceneTreeMatch {
          model: tree.model.clone(),
          reference: tree.reference.clone(),
          coverage: tree.coverage,
          error: tree.error,
        })
        .collect(),
    };

    Ok((files, instances))
  }

  /// Reads each library object once for the shader pairings, where `is_inferring`, the objects to look for in the
  /// static geometry, where a root is named, and the detail objects the detail step matches, which it then need not
  /// read again. The walk stops once `job` is cancelled.
  fn read_object_library(
    options: &LevelDecompileOptions,
    library: &mut LevelObjectLibrary,
    job: &JobHandle,
    compiler_shaders: Option<&ShaderCompilerFile>,
    is_inferring: bool,
  ) -> XrfResult<(
    Option<LevelShaderPairings>,
    Option<LevelInstanceLibrary>,
    LevelDetailLibrary,
  )> {
    let mut pairings: Option<LevelShaderPairings> = is_inferring.then(LevelShaderPairings::default);
    let mut instances: Option<LevelInstanceLibrary> =
      (!options.libraries.is_empty()).then(LevelInstanceLibrary::default);
    let mut details: LevelDetailLibrary = LevelDetailLibrary::default();

    library.read_each(
      job,
      |_| true,
      |entry, object| {
        if let Some(pairings) = &mut pairings {
          pairings.add_object(&object);
        }

        if let Some(instances) = &mut instances {
          instances.add(entry.reference.clone(), entry.path.clone(), &object, compiler_shaders);
        }

        if LevelDetailLibrary::is_detail(entry) {
          details.add(entry, &object);
        }

        Ok(())
      },
    )?;

    if let Some(instances) = &mut instances {
      // An earlier decompile's own objects would be found whole as placements.
      let own: String = format!("levels\\{}\\", options.name).to_lowercase();

      instances.sort();
      instances
        .objects
        .retain(|object| !object.reference.to_lowercase().starts_with(&own));
    }

    Ok((pairings, instances, details))
  }

  /// The trees' impostor atlas, where the level has one. One missing a file or not reading is refused, and the trees
  /// come back without impostors.
  fn read_impostors(sources: &mut LevelSceneSources, draft: &mut LevelSceneDraft) -> Option<LevelTreeImpostorAtlas> {
    let color: Option<Vec<u8>> = sources.take(LevelTreeImpostorAtlas::COLOR_FILE_NAME);
    let normal: Option<Vec<u8>> = sources.take(LevelTreeImpostorAtlas::NORMAL_FILE_NAME);
    let mut refuse = |file: &str, reason: String| {
      draft.refused.push(LevelSceneRefusal {
        file: String::from(file),
        object: String::from(file),
        reason: format!("The impostor atlas was not read, so the trees come back without impostors: {reason}"),
      });
    };

    match (color, normal) {
      (None, None) => None,
      (Some(_), None) => {
        refuse(
          LevelTreeImpostorAtlas::NORMAL_FILE_NAME,
          String::from("its normal map is missing"),
        );

        None
      }
      (None, Some(_)) => {
        refuse(
          LevelTreeImpostorAtlas::COLOR_FILE_NAME,
          String::from("its color map is missing"),
        );

        None
      }
      (Some(color), Some(normal)) => LevelTreeImpostorAtlas::read_from_bytes(color, normal)
        .inspect_err(|error| refuse(LevelTreeImpostorAtlas::COLOR_FILE_NAME, error.to_string()))
        .ok(),
    }
  }

  /// The breakable parts the directory holds, by their paths in it; one that is no OGF is refused, and its breakable
  /// stays an entity.
  fn read_breakable_parts(sources: &mut LevelSceneSources, draft: &mut LevelSceneDraft) -> BTreeMap<String, OgfFile> {
    let mut parts: BTreeMap<String, OgfFile> = BTreeMap::new();

    for listing in Self::LISTINGS {
      for path in sources.get_listed(listing) {
        match sources.take_read(&path, OgfFile::read_from_bytes) {
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
}
