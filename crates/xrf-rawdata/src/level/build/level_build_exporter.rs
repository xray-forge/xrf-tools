use std::collections::HashMap;
use std::path::Path;

use byteorder::ByteOrder;
use xrf_editable_level::{
  EditableBuildParams, EditableDetailObjectTool, EditableGlowObject, EditableLevelGuid, EditableLevelOptions,
  EditableLevelScene, EditableLightObject, EditableObjectPart, EditablePortalObject, EditableSceneObject,
  EditableSectorObject, EditableShape, EditableShapeObject, EditableWallmarkTool,
};
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_level::{BuildPrjFile, BuildPrjGlow, BuildPrjMuModel, BuildPrjMuReference, BuildPrjParams, BuildPrjPortal};
use xrf_math::{Aabb, Vector3d};

use crate::level::build::level_build_climbables::{LevelBuildClimbableExport, LevelBuildClimbables};
use crate::level::build::level_build_details::LevelBuildDetails;
use crate::level::build::level_build_export::{LevelBuildCensus, LevelBuildExport};
use crate::level::build::level_build_geometry::LevelBuildGeometry;
use crate::level::build::level_build_library::LevelBuildLibrary;
use crate::level::build::level_build_lights::LevelBuildLights;
use crate::level::build::level_build_occluders::LevelBuildOccluders;
use crate::level::build::level_build_portal::LevelBuildPortal;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_sources::LevelBuildSources;
use crate::level::build::level_build_tables::{LevelBuildTableRecords, LevelBuildTables};
use crate::level::level_entity_conversion::LevelEntityConversion;
use crate::level::level_map_conversion::LevelMapConversion;
use crate::level::level_scene_exporter::LevelSceneExporter;
use crate::provenance::level_scene_provenance::LevelSceneRefusal;

/// Exports a scene into the `build.prj` the Level Editor's Build hands xrLC (`SceneBuilder::CompileStatic` and
/// `SaveBuild`, `BuilderRemote.cpp:1421`, `:357`): the sky and the sun generated from the level options, then each tool
/// in the editor's order of classes, glows, static objects, lights, portals and wall marks; with it the occluders
/// (`BuildHOMModel`, `BuildSOMModel`) and the map's settings (`BuildLTX`) the Build writes from the placed objects, and
/// the details (`EDetailManager::Export`).
pub struct LevelBuildExporter;

impl LevelBuildExporter {
  /// `ebqDraft`, the quality whose materials the Build swaps for the plain vertex shaders, which this does not.
  const DRAFT_QUALITY: u16 = 0;

  /// The build project of `scene` and what goes beside it, its library objects, game materials and detail textures read
  /// from `sources`, in `T`'s byte order.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, when a part does not read, a placed object is missing from the library or
  /// does not build, the scene places a multi-use object with an impostor, is of draft quality, or a light does not
  /// build; or it holds details and `sources` name no textures root, or they do not export
  /// ([`LevelBuildDetails`]).
  pub fn export<T: ByteOrder>(
    scene: &EditableLevelScene,
    sources: &LevelBuildSources<'_>,
  ) -> XrfResult<LevelBuildExport> {
    let game_materials: &GameMtlFile = sources.game_materials;
    let guid: EditableLevelGuid = scene.level.get_guid()?;
    let options: EditableLevelOptions = scene.level.get_level_options()?;
    let params: EditableBuildParams = scene.level.get_build_params()?;

    if params.light_quality == Self::DRAFT_QUALITY {
      return Err(XrfError::new_not_implemented_error(
        "A scene of draft quality is not exported: the build would swap its shaders",
      ));
    }

    let placed: EditableObjectPart<EditableSceneObject> = LevelSceneExporter::get_part(scene, guid)?;
    let mut library: LevelBuildLibrary = LevelBuildLibrary::new(sources.objects);

    library.load::<T>(placed.objects.iter().map(|object| object.reference.as_str()))?;

    let by_name: HashMap<&str, &EditableSceneObject> = placed
      .objects
      .iter()
      .map(|object| (object.object.name.as_str(), object))
      .collect();
    let sector_part: EditableObjectPart<EditableSectorObject> = LevelSceneExporter::get_part(scene, guid)?;
    let sectors: LevelBuildSectors = LevelBuildSectors::new(&sector_part.objects, |object, mesh| {
      let placed: &EditableSceneObject = by_name.get(object)?;
      let bounds: Aabb = library
        .get(&placed.reference)?
        .meshes
        .iter()
        .flatten()
        .find(|it| it.name == mesh)?
        .bounds;

      Some(bounds.transform(&placed.object.transform.to_matrix()))
    });
    let mut tables: LevelBuildTables = LevelBuildTables::default();
    let mut geometry: LevelBuildGeometry = LevelBuildGeometry::default();
    let mut lights: LevelBuildLights = LevelBuildLights::default();
    let mut census: LevelBuildCensus = LevelBuildCensus::default();
    let light_part: EditableObjectPart<EditableLightObject> = LevelSceneExporter::get_part(scene, guid)?;

    lights.add_hemisphere(options.light_hemi_quality);
    lights.add_sun(
      options.light_sun_quality,
      &EditableLightObject::read_sun_direction(&light_part.tool_data)?
        .unwrap_or_else(EditableLightObject::get_default_sun_direction),
    )?;

    let glows: Vec<BuildPrjGlow> = LevelSceneExporter::get_part::<EditableGlowObject>(scene, guid)?
      .objects
      .iter()
      .map(|glow| {
        let position: &Vector3d = &glow.object.transform.position;

        Ok(BuildPrjGlow {
          position: *position,
          size: glow.radius,
          flags: u32::from(glow.flags & EditableGlowObject::FIXED_SIZE != 0),
          material: u32::from(tables.get_material(
            &glow.shader,
            None,
            &glow.texture,
            sectors.calculate(position, glow.radius),
          )?),
        })
      })
      .collect::<XrfResult<_>>()?;

    let mut occluders: LevelBuildOccluders = LevelBuildOccluders::default();
    let mut climbables: LevelBuildClimbables = LevelBuildClimbables::default();
    let mut mu_models: Vec<BuildPrjMuModel> = Vec::new();
    let mut mu_references: Vec<BuildPrjMuReference> = Vec::new();
    let mut placed_bounds: Option<Aabb> = None;

    for object in &placed.objects {
      let file: &EditableObjectFile = library
        .get(&object.reference)
        .ok_or_else(|| XrfError::new_invalid_error(format!("Library object '{}' was not read", object.reference)))?;

      occluders.add_object(object, file, game_materials)?;

      // `CSceneObject::GetBox`: the object's box, the union of its meshes', placed.
      if let Some(bounds) = file
        .meshes
        .iter()
        .flatten()
        .map(|mesh| mesh.bounds)
        .reduce(|mut held, bounds| {
          held.include_box(&bounds);
          held
        })
      {
        let bounds: Aabb = bounds.transform(&object.object.transform.to_matrix());

        match &mut placed_bounds {
          Some(held) => held.include_box(&bounds),
          None => placed_bounds = Some(bounds),
        }
      }

      if file.flags
        & (EditableObjectFile::DYNAMIC_FLAG | EditableObjectFile::HOM_FLAG | EditableObjectFile::SOUND_OCCLUDER_FLAG)
        != 0
      {
        census.other_objects += 1;
      } else if file.flags & EditableObjectFile::MULTIPLE_USAGE_FLAG != 0 {
        if file.flags & EditableObjectFile::USING_LOD_FLAG != 0 {
          return Err(XrfError::new_not_implemented_error(format!(
            "Scene object '{}' places '{}', a multi-use object with an impostor, whose LOD textures are not exported yet",
            object.object.name, object.reference
          )));
        }

        mu_references.push(Self::add_mu_reference(
          object,
          file,
          &sectors,
          &mut tables,
          game_materials,
          &mut mu_models,
        )?);
      } else {
        geometry.add_object(object, file, &sectors, &mut tables, game_materials)?;
        climbables.add_object(object, file, game_materials)?;
        census.static_objects += 1;
      }
    }

    let climbables: LevelBuildClimbableExport = climbables.into_export();

    let controls: HashMap<u32, String> = EditableLightObject::read_controls(&light_part.tool_data)?;

    for light in &light_part.objects {
      lights.add_light(light, &controls, &sectors)?;
    }

    let portals: Vec<BuildPrjPortal> = LevelSceneExporter::get_part::<EditablePortalObject>(scene, guid)?
      .objects
      .iter()
      .map(|portal| Self::get_portal(portal, &sectors))
      .collect::<XrfResult<_>>()?;

    if let Some(tool) = scene.get_tool(EditableWallmarkTool::TOOL_NAME) {
      geometry.add_wallmarks(
        &EditableWallmarkTool::read_from_scene_tool::<T>(tool)?,
        &sectors,
        &mut tables,
      )?;
    }

    let details: Option<LevelBuildDetails> = match scene.get_tool(EditableDetailObjectTool::TOOL_NAME) {
      Some(tool) => {
        let textures: &Path = sources.textures.ok_or_else(|| {
          XrfError::new_invalid_error(
            "The scene holds details, whose textures the Build merges into an atlas; name the textures root",
          )
        })?;

        Some(LevelBuildDetails::export::<T>(
          &EditableDetailObjectTool::read_from_scene_tool::<T>(tool)?,
          &mut library,
          textures,
        )?)
      }
      None => None,
    };
    let records: LevelBuildTableRecords = tables.into_records()?;
    // `BuildLTX` takes the first level-bound shape's box over the placed objects'.
    let map_bounds: Aabb = LevelSceneExporter::get_part::<EditableShapeObject>(scene, guid)?
      .objects
      .iter()
      .filter(|shape| shape.shape.usage == EditableShape::LEVEL_BOUND_USAGE)
      .find_map(|shape| Some(shape.shape.get_bounds()?.transform(&shape.object.transform.to_matrix())))
      .or(placed_bounds)
      .unwrap_or_else(|| Aabb::of_point(&Vector3d::ZERO));
    let (hom, som) = occluders.into_files();

    census.faces = geometry.faces.len();
    census.skipped_faces = geometry.skipped_faces;
    census.invalid_faces = geometry.invalid_faces;
    census.static_lights = lights.static_lights.len();
    census.dynamic_lights = lights.dynamic_lights.len();
    census.glows = glows.len();
    census.portals = portals.len();
    census.occluder_polygons = hom.as_ref().map_or(0, |file| file.polygons.len());
    census.sound_occluder_polygons = som.as_ref().map_or(0, |file| file.polygons.len());
    census.multi_use_models = mu_models.len();
    census.multi_use_references = mu_references.len();
    census.detail_objects = details.as_ref().map_or(0, |details| details.file.objects.len());
    census.detail_textures = details.as_ref().map_or(0, |details| details.textures);
    census.climbable_objects = climbables.entities.len();
    census.refused_parts = climbables.refused.len();

    Ok(LevelBuildExport {
      project: BuildPrjFile {
        version: BuildPrjFile::CURRENT_VERSION,
        params: BuildPrjParams {
          smooth_angle: params.smooth_angle,
          weld_distance: params.weld_distance,
          lightmap_pixels_per_meter: params.light_pixel_per_meter,
          lightmap_jitter_samples: params.light_jitter_samples,
          lightmap_rms_zero: params.light_rms_zero,
          lightmap_rms: params.light_rms,
          quality: params.light_quality,
          quality_reserved: params.light_quality_reserved,
          reserved: params.reserved,
        },
        light_controls: lights.get_controls()?,
        static_lights: lights.static_lights,
        dynamic_lights: lights.dynamic_lights,
        vertices: geometry.vertices,
        faces: geometry.faces,
        smoothing_groups: Some(geometry.smoothing_groups),
        materials: records.materials,
        render_shaders: records.render_shaders,
        compiler_shaders: records.compiler_shaders,
        textures: records.textures,
        glows,
        portals,
        lods: Vec::new(),
        mu_models,
        mu_references,
      },
      hom,
      som,
      map: LevelMapConversion::export(&options, &map_bounds, &sector_part.objects),
      details,
      entities: climbables.entities,
      refused: climbables
        .refused
        .into_iter()
        .map(|(object, reason)| LevelSceneRefusal {
          file: String::from(LevelEntityConversion::SPAWN_FILE_NAME),
          object,
          reason,
        })
        .collect(),
      census,
    })
  }

  /// `BuildMUObject` (`BuilderRemote.cpp:837`): the reference placing `object` by `placed`'s transform in the sector
  /// of its first mesh, the model of `object` built first where `models` holds none of its name, in its own space and
  /// that first placement's sector, with no impostor.
  pub(crate) fn add_mu_reference(
    placed: &EditableSceneObject,
    object: &EditableObjectFile,
    sectors: &LevelBuildSectors,
    tables: &mut LevelBuildTables,
    game_materials: &GameMtlFile,
    models: &mut Vec<BuildPrjMuModel>,
  ) -> XrfResult<BuildPrjMuReference> {
    let model: usize = match models.iter().position(|model| model.name == placed.reference) {
      Some(model) => model,
      None => {
        let mut geometry: LevelBuildGeometry = LevelBuildGeometry::default();

        geometry.add_model(placed, object, sectors, tables, game_materials)?;
        models.push(BuildPrjMuModel {
          name: placed.reference.clone(),
          vertices: geometry.vertices,
          faces: geometry.faces,
          lod: u16::MAX,
          smoothing_groups: Some(geometry.smoothing_groups),
        });
        models.len() - 1
      }
    };

    Ok(BuildPrjMuReference {
      model: u32::try_from(model)
        .map_err(|_| XrfError::new_invalid_error("A build holds more multi-use models than its u32 indices number"))?,
      transform: placed.object.transform.to_matrix(),
      flags: 0,
      sector: LevelBuildGeometry::get_model_sector(placed, object, sectors),
      sector_padding: 0,
      reserved: [0; 8],
    })
  }

  /// A portal between the sectors it names: its polygon as the scene holds it where it has no more corners than a
  /// compiled portal (a decompiled portal holds the compiled one), else simplified as the Build simplifies it
  /// ([`LevelBuildPortal`]).
  fn get_portal(portal: &EditablePortalObject, sectors: &LevelBuildSectors) -> XrfResult<BuildPrjPortal> {
    let sector = |name: &str| {
      sectors.get_number(name).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Portal '{}' joins sector '{name}', which the scene does not hold",
          portal.object.name
        ))
      })
    };
    let vertices: Vec<Vector3d> = if portal.vertices.len() > BuildPrjPortal::MAXIMUM_VERTICES {
      LevelBuildPortal::simplify(&portal.vertices).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Portal '{}' of {} corners simplifies to no polygon, which the editor holds invalid",
          portal.object.name,
          portal.vertices.len()
        ))
      })?
    } else {
      portal.vertices.clone()
    };
    let mut vertex_slots: [Vector3d; BuildPrjPortal::MAXIMUM_VERTICES] =
      [Vector3d::ZERO; BuildPrjPortal::MAXIMUM_VERTICES];

    vertex_slots[..vertices.len()].copy_from_slice(&vertices);

    Ok(BuildPrjPortal {
      sector_front: sector(&portal.sector_front)?,
      sector_back: sector(&portal.sector_back)?,
      vertex_slots,
      vertex_count: u32::try_from(vertices.len()).unwrap_or(u32::MAX),
    })
  }
}
