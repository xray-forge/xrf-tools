use std::path::Path;

use byteorder::ByteOrder;
use xrf_editable_level::{EditableLevelGuid, EditableLevelPart};
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobScope};
use xrf_spawn::LevelSpawnFile;
use xrf_utils::format_path;

use crate::level::level_entity_conversion::LevelEntityConversion;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_decompile_report::{
  LevelDecompileGeometry, LevelDecompileReport, LevelDecompileReportTool, LevelDecompileRootSpawn,
  LevelDecompileSurfaces,
};
use crate::level::scene::level_decompile_request::LevelDecompileRequest;
use crate::level::scene::level_scene_decompile::LevelSceneDecompile;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_listing::LevelSceneListing;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_ai_map_step::LevelAiMapStep;
use crate::level::scene::steps::level_bundle_step::LevelBundleStep;
use crate::level::scene::steps::level_companion_step::LevelCompanionStep;
use crate::level::scene::steps::level_detail_step::LevelDetailStep;
use crate::level::scene::steps::level_game_step::LevelGameStep;
use crate::level::scene::steps::level_geometry_step::LevelGeometryStep;
use crate::level::scene::steps::level_map_step::LevelMapStep;
use crate::level::scene::steps::level_root_spawn::LevelRootSpawn;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;
use crate::provenance::level_scene_provenance::LevelSceneProvenance;

/// Decompiles a level's directory into a Level Editor scene, step by step over one draft: the map's settings, the
/// geometry and its objects, sectors and portals; the bundle's glows and lights; the files the build passes through from
/// a part (`BuilderGame.cpp`); the details and the AI map; and the game's ways and spawn points. Each step names the
/// files it reads; the decompile reads them all once and hashes them into the scene's GUID, so decompiling the same
/// level twice gives the same scene.
pub struct LevelSceneDecompiler;

impl LevelSceneDecompiler {
  /// The owner the scene's `[level_tag]` names.
  pub const OWNER: &'static str = LevelSceneDraft::OWNER;

  /// What a job reports a decompile under: its steps, each named as its detail, and the writing after them.
  pub const PROGRESS_STEPS: &'static str = "steps";
  pub const PROGRESS_WRITE: &'static str = "write";

  /// Each step's name in a job's detail: reading the directory, then the steps in the order they run.
  const STEP_NAMES: [&'static str; 8] = [
    "Reading the level",
    "Map settings",
    "Geometry, sectors and portals",
    "Glows and lights",
    "Passed-through parts",
    "Details",
    "AI map",
    "Ways and spawn points",
  ];

  /// The files of a level's directory the decompile reads, every step's, in the order the steps run.
  pub fn get_source_files() -> Vec<&'static str> {
    let mut files: Vec<&'static str> = [
      LevelMapStep::FILES,
      LevelGeometryStep::FILES,
      LevelBundleStep::FILES,
      LevelCompanionStep::FILES,
      LevelDetailStep::FILES,
      LevelAiMapStep::FILES,
      LevelGameStep::FILES,
    ]
    .concat();
    let mut seen: Vec<&'static str> = Vec::with_capacity(files.len());

    files.retain(|file| {
      let is_new: bool = !seen.contains(file);

      seen.push(file);
      is_new
    });
    files
  }

  /// The files every step reads by what their names hold.
  fn get_listings() -> Vec<LevelSceneListing> {
    [
      LevelMapStep::LISTINGS,
      LevelGeometryStep::LISTINGS,
      LevelBundleStep::LISTINGS,
      LevelCompanionStep::LISTINGS,
      LevelDetailStep::LISTINGS,
      LevelAiMapStep::LISTINGS,
      LevelGameStep::LISTINGS,
    ]
    .concat()
  }

  /// The scene of the level in `directory`: a part per tool whose compiled file the directory holds, the objects of
  /// its geometry, and the main file its name and those files give.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory holds none of [`Self::get_source_files`], one cannot be read or is not the
  /// file its name says, the root spawn given cannot be read or holds no objects chunk or graph, or a step refuses what
  /// it holds.
  pub fn decompile<T: ByteOrder>(directory: &Path, options: &LevelDecompileOptions) -> XrfResult<LevelSceneDecompile> {
    Self::decompile_with::<T>(directory, options, &JobHandle::inert())
  }

  /// [`Self::decompile`], reporting each step to `job` and stopping between them once it is cancelled.
  ///
  /// # Errors
  ///
  /// As [`Self::decompile`], or a cancelled error once `job` is cancelled.
  pub fn decompile_with<T: ByteOrder>(
    directory: &Path,
    options: &LevelDecompileOptions,
    job: &JobHandle,
  ) -> XrfResult<LevelSceneDecompile> {
    let steps: JobScope = job.enter(Self::PROGRESS_STEPS, Some(Self::STEP_NAMES.len() as u64));
    let begin = |index: usize| -> XrfResult {
      if index > 0 {
        steps.advance();
      }

      job.check_cancelled()?;
      job.set_detail(Some(String::from(Self::STEP_NAMES[index])));

      Ok(())
    };

    begin(0)?;

    let mut sources: LevelSceneSources =
      LevelSceneSources::read(directory, &Self::get_source_files(), &Self::get_listings())?;
    let root_spawn: Option<LevelRootSpawn> = options
      .root_spawn
      .as_deref()
      .map(|path| LevelRootSpawn::read::<T>(path, directory))
      .transpose()?;
    let guid: EditableLevelGuid = sources.get_guid(root_spawn.as_ref().and_then(LevelRootSpawn::get_guid_hash));
    let spawn: Option<LevelSpawnFile> = sources.take_read(
      LevelEntityConversion::SPAWN_FILE_NAME,
      LevelSpawnFile::read_from_bytes::<T>,
    )?;
    let respawn: Option<LevelSpawnFile> = sources.take_read(
      LevelEntityConversion::RESPAWN_FILE_NAME,
      LevelSpawnFile::read_from_bytes::<T>,
    )?;
    let mut draft: LevelSceneDraft = LevelSceneDraft::new(guid, &options.name, root_spawn, spawn, respawn);

    begin(1)?;
    LevelMapStep::decompile::<T>(&mut sources, options, &mut draft)?;
    begin(2)?;
    LevelGeometryStep::decompile::<T>(&mut sources, options, &mut draft)?;
    begin(3)?;
    LevelBundleStep::decompile::<T>(&mut sources, options, &mut draft)?;
    begin(4)?;
    LevelCompanionStep::decompile::<T>(&mut sources, options, &mut draft)?;
    begin(5)?;
    LevelDetailStep::decompile::<T>(&mut sources, options, &mut draft)?;
    begin(6)?;
    LevelAiMapStep::decompile::<T>(&mut sources, options, &mut draft)?;
    begin(7)?;
    LevelGameStep::decompile::<T>(&mut sources, options, &mut draft)?;
    steps.advance();

    Ok(draft.finish(options, &sources))
  }

  /// The level of `request.source` decompiled into the scene `request.destination`, its objects and textures written
  /// under their roots ([`LevelDecompileRequest`]'s defaults resolved), and what was written, as both front-ends report
  /// it. `tool` and `tool_version` name the front-end in the sidecars.
  ///
  /// # Errors
  ///
  /// Returns an error when the request does not resolve, a file it would write exists and it is not forced, the
  /// decompile fails ([`Self::decompile`]) or is cancelled, or a file does not write.
  pub fn decompile_to_path<T: ByteOrder>(
    request: &LevelDecompileRequest,
    tool: &str,
    tool_version: &str,
    job: &JobHandle,
  ) -> XrfResult<LevelDecompileReport> {
    let options: LevelDecompileOptions = request.get_options(tool, tool_version)?;
    let (source, destination) = (&request.source, &request.destination);
    let (objects, textures) = (request.get_objects_root(), request.get_textures_root());

    if !request.is_forced
      && let Some(existing) = LevelSceneDecompile::get_existing_paths(destination).first()
    {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; overwrite to replace the scene",
        format_path(existing)
      )));
    }

    let decompiled: LevelSceneDecompile = Self::decompile_with::<T>(source, &options, job)?;

    if !request.is_forced
      && let Some(existing) = decompiled
        .get_existing_object_paths(&objects)
        .into_iter()
        .chain(decompiled.get_existing_texture_paths(&textures))
        .next()
    {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; overwrite to replace the scene's objects and textures",
        format_path(&existing)
      )));
    }

    job.check_cancelled()?;

    {
      let _writing: JobScope = job.enter(Self::PROGRESS_WRITE, None);

      job.set_detail(Some(format_path(destination).to_string()));
      decompiled.write_to_path::<T>(destination, &objects, &textures)?;
    }

    Ok(LevelDecompileReport {
      source: format_path(source).to_string(),
      destination: format_path(destination).to_string(),
      sidecar: format_path(&LevelSceneProvenance::sidecar_path(destination)).to_string(),
      sources: decompiled
        .provenance
        .sources
        .iter()
        .map(|source| source.file.clone())
        .collect(),
      tools: decompiled
        .scene
        .tools
        .iter()
        .map(|tool| {
          Ok(LevelDecompileReportTool {
            name: tool.name.clone(),
            objects: match tool.parts.first() {
              Some(EditableLevelPart::Text(part)) => Some(part.get_objects()?.len()),
              _ => None,
            },
          })
        })
        .collect::<XrfResult<_>>()?,
      inferred: decompiled.provenance.inferred,
      refused: decompiled.provenance.refused,
      root_spawn: decompiled.provenance.root_spawn.map(|root| LevelDecompileRootSpawn {
        file: root.file,
        objects: root.objects,
        disagreements: root.disagreements.len(),
        update_packets: root.update_packets.len(),
      }),
      objects_root: format_path(&objects).to_string(),
      textures_root: format_path(&textures).to_string(),
      textures: decompiled.textures.len(),
      geometry: decompiled.provenance.geometry.map(|geometry| LevelDecompileGeometry {
        objects: geometry.objects.len(),
        census: geometry.census,
        surfaces: geometry.surfaces.map(|surfaces| LevelDecompileSurfaces {
          exact_triangles: surfaces.census.exact_triangles,
          traced_triangles: surfaces.census.traced_triangles,
          uncollided_triangles: surfaces.census.uncollided_triangles,
          compiler_shaders: surfaces.census.compiler_shaders.len(),
          unknown_game_materials: surfaces.census.unknown_game_materials.len(),
        }),
      }),
    })
  }
}
