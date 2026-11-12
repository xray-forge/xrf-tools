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
use xrf_utils::format_path;

use crate::level::instance::level_instance_library::LevelInstanceLibrary;
use crate::level::instance::level_instance_placement::LevelInstancePlacement;
use crate::level::instance::level_tree_match::LevelTreeMatch;
use crate::level::level_breakable_conversion::LevelBreakableConversion;
use crate::level::level_collision_index::LevelCollisionIndex;
use crate::level::level_decal_index::LevelDecalIndex;
use crate::level::level_entity_conversion::LevelEntityConversion;
use crate::level::level_geometry_conversion::{LevelGeometryConversion, LevelGeometryDecompile, LevelGeometrySources};
use crate::level::level_map_conversion::LevelMapConversion;
use crate::level::level_surface_inference::{LevelShaderPairings, LevelSurfaceInference, LevelSurfaces};
use crate::level::level_tree_impostor::LevelTreeImpostorAtlas;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_decompile::LevelSceneLibraryFile;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_listing::LevelSceneListing;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::{
  LevelSceneGeometry, LevelSceneInference, LevelSceneInstancePlacement, LevelSceneInstances, LevelSceneLibraryObject,
  LevelSceneRefusal, LevelSceneSource, LevelSceneSurfaceLibraries, LevelSceneSurfaces, LevelSceneTreeMatch,
};
use crate::provenance::sidecar::Sidecar;

/// The level's geometry ([`LevelGeometryConversion`]): the objects of its static geometry, collision, occluders,
/// ladders and breakable parts, the scene objects placing them, and the sectors and portals, their surfaces dressed
/// from `level.cform` where the options ask it and the directory holds one. It runs before every step but the map's, so
/// the sectors take the first names of their prefix, and names the folded ladders' and breakables' packets in the
/// draft, which the game step leaves out of the spawn points.
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
    let breakable_parts: BTreeMap<String, OgfFile> = Self::read_breakable_parts::<T>(sources, draft);
    let impostors: Option<LevelTreeImpostorAtlas> = Self::read_impostors(sources, draft);
    let mut surfaces: Option<(LevelSurfaceInference, LevelSceneSurfaceLibraries)> = match (options.surfaces, &collision)
    {
      (LevelSurfaces::Infer, Some(_)) => Some(Self::read_surface_inference::<T>(options)?),
      _ => None,
    };
    let library: Option<LevelInstanceLibrary> = Self::read_instance_library::<T>(options)?;
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
        spawn: draft.spawn.as_ref(),
        breakable_parts: &breakable_parts,
        impostors: impostors.as_ref(),
        library: library.as_ref(),
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

    draft.infer_if(
      geometry.census.instances + geometry.census.library_tree_models > 0,
      &[LevelSceneInference::LibraryInstances],
    );

    let instances: Option<LevelSceneInstances> = match &library {
      Some(library) => {
        let (files, instances) = Self::copy_library(options, library, &geometry.instances, &geometry.trees)?;

        draft.library = files;
        Some(instances)
      }
      None => None,
    };

    draft.refused.extend(geometry.refused);
    draft.folded_entities = geometry.folded_entities;
    draft.geometry = Some(LevelSceneGeometry {
      objects: geometry.objects.iter().map(|object| object.reference.clone()).collect(),
      census: geometry.census,
      surfaces: surfaces.map(|(inference, libraries)| LevelSceneSurfaces {
        libraries,
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
  /// Each library object `placements` place, read from its library with its thumbnail to be copied beside the scene's
  /// objects, and the record of the roots, the copies and the placements.
  ///
  /// # Errors
  ///
  /// Returns an error when a placed object's file cannot be read again.
  fn copy_library(
    options: &LevelDecompileOptions,
    library: &LevelInstanceLibrary,
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
      unread_objects: library.unread_objects,
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

  /// The objects of the library roots the options name, looked for in the static geometry; none without a root. A
  /// surface renders where its compiler shader in the options' `shaders_xrlc.xr` says so, or where none is named.
  fn read_instance_library<T: ByteOrder>(options: &LevelDecompileOptions) -> XrfResult<Option<LevelInstanceLibrary>> {
    if options.libraries.is_empty() {
      return Ok(None);
    }

    let compiler_shaders: Option<ShaderCompilerFile> = options
      .compiler_shaders
      .as_ref()
      .map(|path| ShaderCompilerFile::read_from_path::<T, _>(path))
      .transpose()?;
    let mut library: LevelInstanceLibrary =
      LevelInstanceLibrary::read_from_roots::<T>(&options.libraries, compiler_shaders.as_ref())?;
    // The decompile's own folder holds what an earlier decompile of this level wrote, whose shells would be found whole.
    let own: String = format!("levels\\{}\\", options.name).to_lowercase();

    library
      .objects
      .retain(|object| !object.reference.to_lowercase().starts_with(&own));

    Ok(Some(library))
  }

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
