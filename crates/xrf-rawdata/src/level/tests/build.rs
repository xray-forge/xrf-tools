//! The pieces a `build.prj` export is made of: its tables, its generated sky and sun, and its sectors.

use xrf_editable_level::{EditableCustomObject, EditableObjectTransform, EditableSectorItem, EditableSectorObject};
use xrf_error::XrfResult;
use xrf_level::LevelDynamicLight;
use xrf_math::{Aabb, Vector2d, Vector3d};

use crate::level::build::level_build_hemisphere::LevelBuildHemisphere;
use crate::level::build::level_build_lights::LevelBuildLights;
use crate::level::build::level_build_sectors::LevelBuildSectors;
use crate::level::build::level_build_tables::LevelBuildTables;

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
