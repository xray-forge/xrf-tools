//! The pieces a `build.prj` export is made of: its tables, its generated sky and sun, its sectors, the occluders and
//! the map's settings written beside it.

use xrf_editable_level::{
  EditableCustomObject, EditableLevelOptions, EditableObjectTransform, EditableSceneObject, EditableSectorItem,
  EditableSectorObject,
};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::LevelDynamicLight;
use xrf_math::{Aabb, Vector2d, Vector3d};

use crate::level::build::level_build_hemisphere::LevelBuildHemisphere;
use crate::level::build::level_build_lights::LevelBuildLights;
use crate::level::build::level_build_occluders::LevelBuildOccluders;
use crate::level::build::level_build_portal::LevelBuildPortal;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::LevelBuildTables;
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
  let geometry: Vec<LevelVisualGeometry> = LevelVisualGeometry::of_triangles(
    "floor\\floor_tile",
    "default",
    &SurfaceDress {
      compiler_shader: String::from("default"),
      game_material: material.to_owned(),
    },
    &[
      [corner(0.0, 0.0), corner(0.0, 1.0), corner(1.0, 1.0)],
      [corner(0.0, 0.0), corner(1.0, 1.0), corner(1.0, 0.0)],
    ],
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
