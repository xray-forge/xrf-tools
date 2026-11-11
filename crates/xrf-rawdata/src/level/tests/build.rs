//! The pieces a `build.prj` export is made of: its tables, its generated sky and sun, its sectors, its multi-use
//! models, the occluders, ladders and the map's settings written beside it.

use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{
  EditableBuildParams, EditableCustomObject, EditableLevelOptions, EditableObjectTransform, EditableSceneObject,
  EditableSectorItem, EditableSectorObject,
};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::LevelDynamicLight;
use xrf_math::{Aabb, Affine3d, Vector2d, Vector3d};
use xrf_ogf::{OgfFile, OgfModelType};
use xrf_spawn::{AlifeObjectInherited, Shape};
use xrf_test_utils::GeneratedResources;

use crate::LevelBuildExport;
use crate::level::build::level_build_breakables::LevelBuildBreakables;
use crate::level::build::level_build_climbables::LevelBuildClimbables;
use crate::level::build::level_build_geometry::LevelBuildGeometry;
use crate::level::build::level_build_hemisphere::LevelBuildHemisphere;
use crate::level::build::level_build_lighting::LevelBuildLighting;
use crate::level::build::level_build_lights::LevelBuildLights;
use crate::level::build::level_build_models::LevelBuildModels;
use crate::level::build::level_build_occluders::LevelBuildOccluders;
use crate::level::build::level_build_portal::LevelBuildPortal;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::LevelBuildTables;
use crate::level::build::level_build_textures::LevelBuildTextures;
use crate::level::level_breakable_conversion::LevelBreakableConversion;
use crate::level::level_map_conversion::LevelMapConversion;
use crate::level::level_occluder_conversion::LevelOccluderConversion;
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

fn sector(name: &str, is_default: bool, object: &str) -> EditableSectorObject {
  EditableSectorObject {
    object: EditableCustomObject::new(name.to_owned(), EditableObjectTransform::default()),
    color: [255; 4],
    is_default,
    items: vec![EditableSectorItem {
      object: object.to_owned(),
      mesh: String::from("mesh"),
    }],
    map_index: 0,
  }
}

/// Two sectors, `room` holding the unit box at the origin and `hall` the one 10 m along x, and the default after them.
fn sectors() -> LevelBuildSectors {
  LevelBuildSectors::new(
    &[sector("room", false, "room_shell"), sector("hall", false, "hall_shell")],
    |object, _| {
      let offset: f32 = if object == "hall_shell" { 10.0 } else { 0.0 };

      Some(Aabb {
        min: Vector3d::new(offset - 1.0, -1.0, -1.0),
        max: Vector3d::new(offset + 1.0, 1.0, 1.0),
      })
    },
  )
}

#[test]
fn keeps_each_material_once_per_texture_shaders_and_sector() -> XrfResult {
  let mut tables: LevelBuildTables = LevelBuildTables::default();
  let first: u16 = tables.get_material("models\\model", Some("default"), "brick\\wall", 0)?;

  assert_eq!(
    tables.get_material("models\\model", Some("default"), "BRICK\\WALL", 0)?,
    first,
    "textures compare without case"
  );
  assert_eq!(
    tables.get_material("models\\model", Some("default"), "brick\\wall", 1)?,
    1
  );
  assert_eq!(tables.get_material("models\\model", None, "brick\\wall", 0)?, 2);
  assert!(tables.get_material("models\\model", None, "", 0).is_err());

  let records = tables.into_records()?;

  assert_eq!(records.textures.len(), 1);
  assert_eq!(records.compiler_shaders.len(), 1);
  assert_eq!(
    records.materials[2].compiler_shader,
    LevelBuildTables::NO_COMPILER_SHADER
  );

  Ok(())
}

#[test]
fn spreads_the_skys_energy_over_each_qualitys_directions_downwards() {
  for (quality, count) in [(1, 26), (2, 91), (3, 196)] {
    let lights: Vec<(Vector3d, f32)> = LevelBuildHemisphere::get_lights(quality, 2.0);

    assert_eq!(lights.len(), count);
    assert!((lights.iter().map(|(_, energy)| energy).sum::<f32>() - 2.0).abs() < 1e-4);
    assert!(
      lights
        .iter()
        .all(|(direction, _)| direction.y <= 0.0 && (direction.length() - 1.0).abs() < 1e-5)
    );
  }
}

#[test]
fn generates_the_sun_as_the_shipped_editor_does() -> XrfResult {
  let mut lights: LevelBuildLights = LevelBuildLights::default();
  let direction: Vector2d = Vector2d::new(-0.5, 1.7);
  let step: f32 = 3.0_f32.to_radians() / 7.0;
  let (pitch_from, heading_from) = (-0.5 - 3.0_f32.to_radians() / 2.0, 1.7 - 3.0_f32.to_radians() / 2.0);

  lights.add_sun(3, &direction)?;

  assert_eq!(lights.static_lights.len(), 49);
  assert_eq!(lights.dynamic_lights.len(), 1);
  assert!((lights.static_lights[0].diffuse.r - 1.0 / 49.0).abs() < 1e-7);

  // Each sample from the one before: the second heading is the first's taken through the step once more.
  let heading = |light: &LevelDynamicLight| (-light.direction.x).atan2(light.direction.z);

  assert!((heading(&lights.static_lights[0]) - heading_from).abs() < 1e-5);
  assert!((heading(&lights.static_lights[1]) - (heading_from + heading_from * step)).abs() < 1e-5);
  assert!((lights.static_lights[0].direction.y.asin() - pitch_from).abs() < 1e-5);
  assert!(lights.add_sun(4, &direction).is_err());

  Ok(())
}

#[test]
fn places_items_and_spheres_in_the_sectors_holding_them() {
  let sectors: LevelBuildSectors = sectors();

  assert_eq!(sectors.get_number("hall"), Some(1));
  assert_eq!(sectors.find("hall_shell", "mesh"), 1);
  assert_eq!(sectors.find("lamp", "mesh"), 2, "an unlisted mesh goes to the default");
  assert_eq!(sectors.calculate(&Vector3d::new(10.0, 0.0, 0.0), 0.1), 1);
  assert_eq!(sectors.calculate(&Vector3d::new(5.0, 0.0, 0.0), 0.1), 2);
  assert_eq!(
    sectors.get_light_sectors(&Vector3d::new(0.5, 0.0, 0.0), 9.0),
    vec![0, 1]
  );
  assert!(
    sectors
      .get_light_sectors(&Vector3d::new(50.0, 0.0, 0.0), 1.0)
      .is_empty()
  );
}

/// An object of one quad on the ground, two triangles of the game material `material`, flagged `flags`.
fn quad(material: &str, flags: u32) -> XrfResult<EditableObjectFile> {
  let corner = |x: f32, z: f32| Vector3d::new(x, 0.0, z);

  object_of(
    &[
      [corner(0.0, 0.0), corner(0.0, 1.0), corner(1.0, 1.0)],
      [corner(0.0, 0.0), corner(1.0, 1.0), corner(1.0, 0.0)],
    ],
    material,
    flags,
  )
}

/// An object of `triangles`, one surface of the game material `material`, flagged `flags`.
fn object_of(triangles: &[[Vector3d; 3]], material: &str, flags: u32) -> XrfResult<EditableObjectFile> {
  let geometry: Vec<LevelVisualGeometry> = LevelVisualGeometry::of_triangles(
    "floor\\floor_tile",
    "default",
    &SurfaceDress {
      compiler_shader: String::from("default"),
      game_material: material.to_owned(),
    },
    triangles,
  );
  let mut object: EditableObjectFile = ObjectDecompiler::decompile_level_geometry(&geometry)?.object;

  object.flags = flags;

  Ok(object)
}

fn placed_at(x: f32) -> EditableSceneObject {
  EditableSceneObject {
    object: EditableCustomObject::new(
      String::from("placed"),
      EditableObjectTransform {
        position: Vector3d::new(x, 0.0, 0.0),
        ..EditableObjectTransform::default()
      },
    ),
    reference: String::from("placed"),
    flags: 0,
  }
}

#[test]
fn takes_occluders_from_flagged_objects_and_occluder_surfaces_and_sound_occluders_with_their_factor() -> XrfResult {
  let mut stone: GameMtlMaterial = GameMtlMaterial::new(3, "materials\\stone");

  stone.sound_occlusion_factor = 0.25;

  let library: GameMtlFile = GameMtlFile::new(vec![stone], Vec::new());
  let mut occluders: LevelBuildOccluders = LevelBuildOccluders::default();

  occluders.add_object(&placed_at(0.0), &quad("materials\\stone", 0)?, &library)?;
  occluders.add_object(
    &placed_at(10.0),
    &quad(LevelOccluderConversion::OCCLUDER_GAME_MATERIAL, 0)?,
    &library,
  )?;
  occluders.add_object(
    &placed_at(20.0),
    &quad("materials\\stone", EditableObjectFile::HOM_FLAG)?,
    &library,
  )?;
  occluders.add_object(
    &placed_at(30.0),
    &quad("materials\\stone", EditableObjectFile::SOUND_OCCLUDER_FLAG)?,
    &library,
  )?;

  let (hom, som) = occluders.into_files();
  let hom = hom.expect("an occluder surface and a flagged object occlude");
  let som = som.expect("a sound occluder occludes sound");

  assert_eq!(
    hom.polygons.len(),
    4,
    "two faces of each occluder, none of the plain object"
  );
  assert!(hom.polygons.iter().all(|polygon| polygon.vertices[0].x >= 10.0));
  assert_eq!(
    som
      .polygons
      .iter()
      .map(|polygon| (polygon.vertices[0].x >= 30.0, polygon.occlusion))
      .collect::<Vec<_>>(),
    [(true, 0.25), (true, 0.25)],
    "placed, with the factor of their game material"
  );

  let mut occluders: LevelBuildOccluders = LevelBuildOccluders::default();

  assert!(
    occluders
      .add_object(
        &placed_at(0.0),
        &quad("materials\\unknown", EditableObjectFile::SOUND_OCCLUDER_FLAG)?,
        &library
      )
      .is_err(),
    "a sound occluder of a game material the library lacks"
  );
  assert_eq!(LevelBuildOccluders::default().into_files(), (None, None));

  Ok(())
}

#[test]
fn writes_the_maps_settings_as_the_build_writes_them() {
  let mut options: EditableLevelOptions = EditableLevelOptions::new("test_level", (1 << 1) | (1 << 3));
  let mut first: EditableSectorObject = sector("room", false, "room_shell");
  let second: EditableSectorObject = sector("hall", false, "hall_shell");
  let bounds: Aabb = Aabb {
    min: Vector3d::new(-1.5, -10.0, 2.0),
    max: Vector3d::new(3.25, 10.0, 4.0),
  };

  first.map_index = EditableSectorObject::NO_MAP;

  assert_eq!(
    String::from_utf8_lossy(&LevelMapConversion::export(
      &options,
      &bounds,
      &[first.clone(), second.clone()]
    )),
    "[map_usage]\r\nver=1.0\r\ndeathmatch\r\nartefacthunt\r\n;\r\n[level_map]\r\n\
     bound_rect = -1.500000,2.000000,3.250000,4.000000\r\ntexture = map\\map_test_level\r\n;\r\n;\r\n\
     [sub_level_map]\r\n0 = 255\r\n1 = 0\r\n"
  );

  options.build_options = String::from("[extra]\r\nkey = value");

  let written: Vec<u8> = LevelMapConversion::export(&options, &bounds, &[first, second]);

  assert!(
    written.windows(14).any(|window| window == b"key = value\0;\r"),
    "the build options' text ended by a zero byte, as `w_stringZ` writes it"
  );
}

#[test]
fn simplifies_a_portal_of_more_corners_to_six_on_its_plane_wound_as_it_was() {
  // A regular octagon of radius 2 on the plane x + z = 4, centred at (2, 1, 2).
  let corner = |index: usize| {
    let angle: f32 = std::f32::consts::TAU * index as f32 / 8.0;
    let (along, height) = (2.0 * angle.cos(), 2.0 * angle.sin());

    Vector3d::new(
      2.0 + along * std::f32::consts::FRAC_1_SQRT_2,
      1.0 + height,
      2.0 - along * std::f32::consts::FRAC_1_SQRT_2,
    )
  };
  let octagon: Vec<Vector3d> = (0..8).map(corner).collect();
  let simplified: Vec<Vector3d> = LevelBuildPortal::simplify(&octagon).expect("a polygon");
  let winding = |vertices: &[Vector3d]| (&vertices[1] - &vertices[0]).cross(&(&vertices[2] - &vertices[0]));

  assert_eq!(simplified.len(), 6);
  assert!(
    simplified.iter().all(|vertex| (vertex.x + vertex.z - 4.0).abs() < 1e-4),
    "every corner on the portal's plane: {simplified:?}"
  );
  assert!(
    simplified
      .iter()
      .all(|vertex| vertex.distance(&Vector3d::new(2.0, 1.0, 2.0)) < 2.7),
    "a collapsed edge's corner where its neighbours meet, 2.61 from the centre: {simplified:?}"
  );
  assert!(
    winding(&simplified).dot(&winding(&octagon)) > 0.0,
    "wound as the portal was"
  );

  let mut reversed: Vec<Vector3d> = octagon;

  reversed.reverse();

  let simplified: Vec<Vector3d> = LevelBuildPortal::simplify(&reversed).expect("a polygon");

  assert!(
    winding(&simplified).dot(&winding(&reversed)) > 0.0,
    "the other way round too"
  );
  assert_eq!(
    LevelBuildPortal::simplify(&[Vector3d::ZERO; 7]),
    None,
    "corners holding no plane"
  );
}

#[test]
fn registers_a_climbable_surfaces_material_though_its_faces_are_left_out() -> XrfResult {
  let mut ladder: GameMtlMaterial = GameMtlMaterial::new(7, "materials\\fake_ladders");

  ladder.flags |= GameMtlMaterial::FLAG_CLIMABLE;

  let library: GameMtlFile = GameMtlFile::new(vec![ladder], Vec::new());
  let mut tables: LevelBuildTables = LevelBuildTables::default();
  let mut geometry: LevelBuildGeometry = LevelBuildGeometry::default();

  geometry.add_object(
    &placed_at(0.0),
    &quad("materials\\fake_ladders", 0)?,
    &sectors(),
    &mut tables,
    &library,
  )?;

  assert_eq!((geometry.faces.len(), geometry.skipped_faces), (0, 2));
  assert_eq!(
    tables.into_records()?.materials.len(),
    1,
    "the material registered before the game material is looked at, as `BuildMesh` does"
  );

  Ok(())
}

/// A game material library of one climbable material, `materials\fake_ladders`, and `materials\stone`.
fn ladders() -> GameMtlFile {
  let mut ladder: GameMtlMaterial = GameMtlMaterial::new(7, "materials\\fake_ladders");

  ladder.flags |= GameMtlMaterial::FLAG_CLIMABLE;

  GameMtlFile::new(vec![ladder, GameMtlMaterial::new(3, "materials\\stone")], Vec::new())
}

#[test]
fn makes_a_ladder_of_a_climbable_part_at_its_box_turned_to_face_its_faces() -> XrfResult {
  let mut climbables: LevelBuildClimbables = LevelBuildClimbables::default();

  climbables.add_object(&placed_at(10.0), &quad("materials\\fake_ladders", 0)?, &ladders())?;
  climbables.add_object(&placed_at(20.0), &quad("materials\\stone", 0)?, &ladders())?;

  let export = climbables.into_export();
  let [entity] = export.entities.as_slice() else {
    panic!("one ladder of the climbable quad alone: {:?}", export.entities);
  };
  let AlifeObjectInherited::CseAlifeObjectClimable(climbable) = &entity.inherited else {
    panic!("a climable_object");
  };
  let [Shape::Box(shape)] = climbable.base.shape.as_slice() else {
    panic!("one box");
  };
  let frame: Affine3d = Affine3d::from_xyz(&entity.header.direction);
  let half: [f32; 3] = [shape.x_axis.length(), shape.y_axis.length(), shape.z_axis.length()];
  let thin: usize = (0..3).min_by(|a, b| half[*a].total_cmp(&half[*b])).unwrap_or(0);
  let facing: Vector3d = frame.transform_direction(&[Vector3d::X, Vector3d::Y, Vector3d::Z][thin]);

  assert_eq!(
    (entity.header.section.as_str(), entity.header.name.as_str()),
    ("climable_object", "clmbl#0")
  );
  assert_eq!(climbable.game_material, "materials\\fake_ladders");
  assert!(
    entity.header.position.distance(&Vector3d::new(10.5, 0.0, 0.5)) < 1e-5,
    "{:?}",
    entity.header.position
  );
  assert!(half[thin] < 1e-6, "a flat part's box is flat: {half:?}");
  assert!(facing.y > 0.999, "its thin axis faces up, as the quad does: {facing:?}");
  assert!(export.refused.is_empty());

  Ok(())
}

/// A breakable part comes out as a rigid skeleton in the frame of its box, a bone per fragment, placed by a
/// `breakable_object` named by its file; its points, placed as the engine places them, are the faces' own.
#[test]
fn makes_a_breakable_of_a_breakable_part_in_the_frame_of_its_box() -> XrfResult {
  let mut glass: GameMtlMaterial = GameMtlMaterial::new(9, "materials\\glass");

  glass.flags |= GameMtlMaterial::FLAG_BREAKABLE;

  let library: GameMtlFile = GameMtlFile::new(vec![glass, GameMtlMaterial::new(3, "materials\\stone")], Vec::new());
  // A pane of eight triangles, standing in x and y.
  let corner = |x: f32, y: f32| Vector3d::new(x, y, 0.0);
  let pane: Vec<[Vector3d; 3]> = (0..4)
    .flat_map(|step| {
      let x: f32 = step as f32 * 0.5;

      [
        [corner(x, 0.0), corner(x, 1.0), corner(x + 0.5, 1.0)],
        [corner(x, 0.0), corner(x + 0.5, 1.0), corner(x + 0.5, 0.0)],
      ]
    })
    .collect();
  let mut breakables: LevelBuildBreakables = LevelBuildBreakables::default();

  breakables.add_object(&placed_at(10.0), &object_of(&pane, "materials\\glass", 0)?, &library)?;
  breakables.add_object(&placed_at(20.0), &quad("materials\\stone", 0)?, &library)?;

  let export = breakables.into_export()?;
  let ([written], [entity]) = (export.parts.as_slice(), export.entities.as_slice()) else {
    panic!("one breakable of the pane alone: {:?}", export.entities);
  };
  let AlifeObjectInherited::CseAlifeObjectBreakable(breakable) = &entity.inherited else {
    panic!("a breakable_object");
  };
  let part: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(written.write::<XRayByteOrder>()?)?;
  let ik_data = part.ik_data.as_ref().expect("bones' physics");

  // The file goes where the level's other files are listed, the packet names it as the Build does.
  assert_eq!(
    (
      written.path.as_str(),
      entity.header.name.as_str(),
      breakable.base.visual_name.as_str()
    ),
    ("meshes/brkbl#0.ogf", "meshes\\brkbl#0.ogf", "meshes\\brkbl#0")
  );
  assert_eq!(
    OgfModelType::from_raw(part.header.model_type),
    Some(OgfModelType::SkeletonRigid)
  );
  assert!(
    (1..=4).contains(&ik_data.bones.len()),
    "eight faces in fragments of two or more"
  );
  assert!(
    ik_data
      .bones
      .iter()
      .all(|bone| bone.joint.ik_flags & 1 != 0 && bone.game_material == "materials\\glass")
  );

  let placement: Affine3d = Affine3d::from_xyz(&entity.header.direction).with_translation(entity.header.position);

  for point in ObjectDecompiler::decompile_breakable(&part)?
    .object
    .meshes
    .iter()
    .flatten()
    .flat_map(|mesh| &mesh.points)
  {
    let placed: Vector3d = placement.transform_point(point);

    assert!(
      pane
        .iter()
        .flatten()
        .any(|corner| placed.distance(&Vector3d::new(corner.x + 10.0, corner.y, corner.z)) < 1e-4),
      "{placed:?} is a corner of the pane"
    );
  }

  assert!(export.refused.is_empty());

  Ok(())
}

/// Proof lighting sets the lightmaps' density and samples and the sky's and sun's quality, and nothing else.
#[test]
fn lights_at_proof_quality_leaving_the_rest_of_the_scene() {
  let mut params: EditableBuildParams = EditableBuildParams::DEFAULT;
  let mut options: EditableLevelOptions = EditableLevelOptions::new("test_level", EditableLevelOptions::SINGLE_PLAYER);
  let (params_before, options_before) = (params, options.clone());

  LevelBuildLighting::PROOF.apply(&mut params, &mut options);

  assert_eq!(
    (
      params.light_pixel_per_meter,
      params.light_jitter_samples,
      options.light_hemi_quality,
      options.light_sun_quality
    ),
    (3.0, 1, 1, 1)
  );
  assert_eq!(
    EditableBuildParams {
      light_pixel_per_meter: params_before.light_pixel_per_meter,
      light_jitter_samples: params_before.light_jitter_samples,
      ..params
    },
    params_before
  );
  assert_eq!(
    EditableLevelOptions {
      light_hemi_quality: options_before.light_hemi_quality,
      light_sun_quality: options_before.light_sun_quality,
      ..options
    },
    options_before
  );
}

/// A rebuild finds the parts an earlier build wrote under the paths it writes them to, so that it replaces them rather
/// than refusing them as files of no build.
#[test]
fn lists_a_written_breakable_part_by_the_path_the_build_writes_it_to() -> XrfResult {
  let directory: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "listed_part"))?;
  let (_, path) = LevelBreakableConversion::get_part_names(0);

  std::fs::create_dir_all(directory.join(LevelBreakableConversion::DIRECTORY))?;
  std::fs::write(directory.join(&path), b"part")?;

  assert!(LevelBuildExport::list_file_names(&directory).contains(&path));

  Ok(())
}

/// A multi-use object's model is built once, in its own space and its first placement's sector, with no impostor;
/// each placement references it with its own transform, in the sector of its own first mesh.
#[test]
fn builds_a_multi_use_model_once_and_references_it_per_placement() -> XrfResult {
  let library: GameMtlFile = GameMtlFile::new(vec![GameMtlMaterial::new(3, "materials\\stone")], Vec::new());
  let mut tree: EditableObjectFile = quad("materials\\stone", EditableObjectFile::MULTIPLE_USAGE_FLAG)?;

  for mesh in tree.meshes.iter_mut().flatten() {
    mesh.name = String::from("mesh");
  }

  let place = |name: &str, x: f32| EditableSceneObject {
    object: EditableCustomObject::new(name.to_owned(), placed_at(x).object.transform),
    reference: String::from("levels\\test\\test_tree_0"),
    flags: 0,
  };
  let mut tables: LevelBuildTables = LevelBuildTables::default();
  let mut built: LevelBuildModels = LevelBuildModels::default();

  for placed in [place("hall_shell", 10.0), place("tree_0001", 30.0)] {
    built.add(
      &placed,
      &tree,
      &sectors(),
      &mut tables,
      &library,
      &LevelBuildTextures::default(),
    )?;
  }

  let LevelBuildModels { models, references, .. } = built;
  let [model] = models.as_slice() else {
    panic!("one model for both placements: {}", models.len());
  };

  assert_eq!(model.name, "levels\\test\\test_tree_0");
  assert_eq!((model.faces.len(), model.lod), (2, u16::MAX));
  assert!(
    model.vertices.iter().all(|vertex| (0.0..=1.0).contains(&vertex.x)),
    "the model stays in its own space: {:?}",
    model.vertices
  );
  assert_eq!(
    references
      .iter()
      .map(|reference| (
        reference.model,
        reference.transform.get_translation().x,
        reference.sector
      ))
      .collect::<Vec<_>>(),
    [(0, 10.0, 1), (0, 30.0, 2)],
    "each placement in the sector naming its first mesh, else the default"
  );

  Ok(())
}

#[test]
fn refuses_a_climbable_part_whose_faces_share_no_edge() -> XrfResult {
  let point = |x: f32, z: f32| Vector3d::new(x, 0.0, z);
  let mut climbables: LevelBuildClimbables = LevelBuildClimbables::default();

  climbables.add_object(
    &placed_at(0.0),
    &object_of(
      &[
        [point(0.0, 0.0), point(0.0, 1.0), point(1.0, 1.0)],
        [point(1.0, 1.0), point(2.0, 1.0), point(2.0, 2.0)],
      ],
      "materials\\fake_ladders",
      0,
    )?,
    &ladders(),
  )?;

  let export = climbables.into_export();

  assert!(export.entities.is_empty());
  assert_eq!(
    export.refused.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
    ["clmbl#0"],
    "the two triangles meet at a point alone, so the part holds faces of no edge shared"
  );

  Ok(())
}
