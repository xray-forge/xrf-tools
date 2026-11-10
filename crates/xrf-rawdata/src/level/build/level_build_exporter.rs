use std::collections::HashMap;
use std::path::Path;

use byteorder::ByteOrder;
use serde::Serialize;
use xrf_editable_level::{
  EditableBuildParams, EditableGlowObject, EditableLevelGuid, EditableLevelOptions, EditableLevelScene,
  EditableLightObject, EditableObjectPart, EditablePortalObject, EditableSceneObject, EditableSectorObject,
  EditableWallmarkTool,
};
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_level::{BuildPrjFile, BuildPrjGlow, BuildPrjParams, BuildPrjPortal};
use xrf_math::{Aabb, Vector3d};

use crate::level::build::level_build_geometry::LevelBuildGeometry;
use crate::level::build::level_build_library::LevelBuildLibrary;
use crate::level::build::level_build_lights::LevelBuildLights;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::{LevelBuildTableRecords, LevelBuildTables};
use crate::level::level_scene_exporter::LevelSceneExporter;

/// Exports a scene into the `build.prj` the Level Editor's Build hands xrLC (`SceneBuilder::CompileStatic` and
/// `SaveBuild`, `BuilderRemote.cpp:1421`, `:357`): the sky and the sun generated from the level options, then each tool
/// in the editor's order of classes, glows, static objects, lights, portals and wall marks.
pub struct LevelBuildExporter;

/// A scene exported: the build project, and what went into it.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelBuildExport {
  pub project: BuildPrjFile,
  pub census: LevelBuildCensus,
}

/// What a build took from the scene and left out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelBuildCensus {
  /// Static objects whose meshes went into the geometry.
  pub static_objects: usize,
  /// Objects of no static geometry: dynamic ones, and occluders, which `level.hom` and `level.som` take.
  pub other_objects: usize,
  pub faces: usize,
  /// Faces of occluder, breakable and climbable surfaces, which other exports take.
  pub skipped_faces: usize,
  /// Faces of no area, which the Build drops.
  pub invalid_faces: usize,
  pub static_lights: usize,
  pub dynamic_lights: usize,
  pub glows: usize,
  pub portals: usize,
}

impl LevelBuildExporter {
  /// `ebqDraft`, the quality whose materials the Build swaps for the plain vertex shaders, which this does not.
  const DRAFT_QUALITY: u16 = 0;

  /// The build project of `scene`, its library objects read under `objects` and their game materials named by
  /// `game_materials`, in `T`'s byte order.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, when a part does not read, a placed object is missing from the library or
  /// does not build, the scene places a multi-use object, is of draft quality, or holds a portal of more corners than
  /// a compiled one, which the Build simplifies and this does not yet; or a light does not build.
  pub fn export<T: ByteOrder>(
    scene: &EditableLevelScene,
    objects: &Path,
    game_materials: &GameMtlFile,
  ) -> XrfResult<LevelBuildExport> {
    let guid: EditableLevelGuid = scene.level.get_guid()?;
    let options: EditableLevelOptions = scene.level.get_level_options()?;
    let params: EditableBuildParams = scene.level.get_build_params()?;

    if params.light_quality == Self::DRAFT_QUALITY {
      return Err(XrfError::new_not_implemented_error(
        "A scene of draft quality is not exported: the build would swap its shaders",
      ));
    }

    let placed: EditableObjectPart<EditableSceneObject> = LevelSceneExporter::get_part(scene, guid)?;
    let mut library: LevelBuildLibrary = LevelBuildLibrary::new(objects);

    library.load::<T>(placed.objects.iter().map(|object| object.reference.as_str()))?;

    let by_name: HashMap<&str, &EditableSceneObject> = placed
      .objects
      .iter()
      .map(|object| (object.object.name.as_str(), object))
      .collect();
    let sectors: LevelBuildSectors = LevelBuildSectors::new(
      &LevelSceneExporter::get_part::<EditableSectorObject>(scene, guid)?.objects,
      |object, mesh| {
        let placed: &EditableSceneObject = by_name.get(object)?;
        let bounds: Aabb = library
          .get(&placed.reference)?
          .meshes
          .iter()
          .flatten()
          .find(|it| it.name == mesh)?
          .bounds;

        Some(bounds.transform(&placed.object.transform.to_matrix()))
      },
    );
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

    for object in &placed.objects {
      let file: &EditableObjectFile = library
        .get(&object.reference)
        .ok_or_else(|| XrfError::new_invalid_error(format!("Library object '{}' was not read", object.reference)))?;

      if file.flags
        & (EditableObjectFile::DYNAMIC_FLAG | EditableObjectFile::HOM_FLAG | EditableObjectFile::SOUND_OCCLUDER_FLAG)
        != 0
      {
        census.other_objects += 1;
      } else if file.flags & EditableObjectFile::MULTIPLE_USAGE_FLAG != 0 {
        return Err(XrfError::new_not_implemented_error(format!(
          "Scene object '{}' places the multi-use object '{}', which is not exported yet",
          object.object.name, object.reference
        )));
      } else {
        geometry.add_object(object, file, &sectors, &mut tables, game_materials)?;
        census.static_objects += 1;
      }
    }

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

    let records: LevelBuildTableRecords = tables.into_records()?;

    census.faces = geometry.faces.len();
    census.skipped_faces = geometry.skipped_faces;
    census.invalid_faces = geometry.invalid_faces;
    census.static_lights = lights.static_lights.len();
    census.dynamic_lights = lights.dynamic_lights.len();
    census.glows = glows.len();
    census.portals = portals.len();

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
          quality_reserved: 0,
          reserved: [0.0; EditableBuildParams::RESERVED_FLOATS],
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
        mu_models: Vec::new(),
        mu_references: Vec::new(),
      },
      census,
    })
  }

  /// A portal between the sectors it names, its polygon as the scene holds it: a decompiled portal holds the compiled
  /// one, which the Build's simplification gives back unchanged.
  fn get_portal(portal: &EditablePortalObject, sectors: &LevelBuildSectors) -> XrfResult<BuildPrjPortal> {
    let sector = |name: &str| {
      sectors.get_number(name).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Portal '{}' joins sector '{name}', which the scene does not hold",
          portal.object.name
        ))
      })
    };

    if portal.vertices.len() > BuildPrjPortal::MAXIMUM_VERTICES {
      return Err(XrfError::new_not_implemented_error(format!(
        "Portal '{}' holds {} corners; the build simplifies such a polygon, which is not exported yet",
        portal.object.name,
        portal.vertices.len()
      )));
    }

    let mut vertex_slots: [Vector3d; BuildPrjPortal::MAXIMUM_VERTICES] =
      [Vector3d::ZERO; BuildPrjPortal::MAXIMUM_VERTICES];

    vertex_slots[..portal.vertices.len()].copy_from_slice(&portal.vertices);

    Ok(BuildPrjPortal {
      sector_front: sector(&portal.sector_front)?,
      sector_back: sector(&portal.sector_back)?,
      vertex_slots,
      vertex_count: u32::try_from(portal.vertices.len()).unwrap_or(u32::MAX),
    })
  }
}
