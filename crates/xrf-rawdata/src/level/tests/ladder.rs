use std::path::PathBuf;

use xrf_chunk::XRayByteOrder;
use xrf_editable_level::EditableObjectTransform;
use xrf_editable_object::{EditableMesh, EditableObjectFile};
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_math::{Affine3d, Vector3d};
use xrf_spawn::{
  AlifeObject, AlifeObjectAbstract, AlifeObjectBreakable, AlifeObjectClimable, AlifeObjectDynamicVisual,
  AlifeObjectInherited, AlifeObjectShape, ClsId, Shape, SpawnPacketHeader,
};
use xrf_test_utils::GeneratedResources;

use crate::level::geometry::level_visual_geometry::LevelVisualGeometry;
use crate::level::instance::level_instance_cutouts::LevelInstanceCutouts;
use crate::level::instance::level_instance_placement::LevelInstancePlacement;
use crate::level::parts::level_decompiled_ladder::LevelDecompiledLadder;
use crate::level::parts::level_ladder_conversion::LevelLadderConversion;
use crate::level::tests::entity::listed;
use crate::level::tests::spawn_data::restrictor;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A ladder as the Build exports one: a box of half sizes `half` in its frame, standing at `position` and turned by
/// `direction`, of `material`.
fn ladder(position: Vector3d, direction: Vector3d, half: [f32; 3], material: &str) -> XrfResult<AlifeObject> {
  let base: AlifeObjectAbstract = restrictor().state.base;

  Ok(AlifeObject {
    header: SpawnPacketHeader {
      section: String::from("climable_object"),
      name: String::from("clmbl#0"),
      position,
      direction,
      ..restrictor().header
    },
    clsid: ClsId::from_section("climable_object")?,
    inherited: AlifeObjectInherited::CseAlifeObjectClimable(Box::new(AlifeObjectClimable {
      base: AlifeObjectShape {
        base: AlifeObjectAbstract {
          custom_data: String::new(),
          ..base
        },
        shape: vec![Shape::Box(Affine3d {
          x_axis: &Vector3d::X * half[0],
          y_axis: &Vector3d::Y * half[1],
          z_axis: &Vector3d::Z * half[2],
          translation: Vector3d::ZERO,
        })],
      },
      game_material: String::from(material),
    })),
  })
}

fn decompile(entity: &AlifeObject) -> XrfResult<LevelDecompiledLadder> {
  Ok(LevelLadderConversion::decompile("test", 3, entity)?.expect("a ladder"))
}

fn get_mesh(object: &EditableObjectFile) -> &EditableMesh {
  &object.meshes.as_deref().unwrap_or_default()[0]
}

/// Each face's normal and corners in the ladder's own frame, as the scene places it: the frame the packet's angles name,
/// at its position.
fn get_box_faces(ladder: &LevelDecompiledLadder, entity: &AlifeObject) -> Vec<(Vector3d, [Vector3d; 3])> {
  let mesh: &EditableMesh = get_mesh(&ladder.object.decompile.object);
  let transform = &ladder.object.transform;
  let placed: Affine3d = Affine3d::from_xyz(&transform.rotation).with_translation(transform.position);
  let frame: Affine3d = Affine3d::from_xyz(&entity.header.direction);
  let round = |value: f32| (value * 1000.0).round() / 1000.0 + 0.0;
  let to_box = |point: &Vector3d| {
    let world: Vector3d = &placed.transform_point(point) - &entity.header.position;

    Vector3d::new(
      round(world.dot(&frame.x_axis)),
      round(world.dot(&frame.y_axis)),
      round(world.dot(&frame.z_axis)),
    )
  };

  mesh
    .faces
    .iter()
    .map(|face| {
      let corners: [Vector3d; 3] = face.corners.map(|corner| to_box(&mesh.points[corner.point as usize]));
      let cross: Vector3d = (&corners[1] - &corners[0]).cross(&(&corners[2] - &corners[0]));
      let normal: Vector3d = &cross / cross.length();

      (
        Vector3d::new(round(normal.x), round(normal.y), round(normal.z)),
        corners,
      )
    })
    .collect()
}

/// A ladder of some thickness comes back as its box less the face behind it: each face wound to face out of the box,
/// none facing back, its corners the box's.
#[test]
fn decompiles_a_ladder_into_its_box_less_the_back_face() -> XrfResult {
  let entity: AlifeObject = ladder(
    Vector3d::new(10.0, 5.0, 3.0),
    Vector3d::new(0.3, 1.2, -0.4),
    [0.4, 2.0, 0.1],
    "materials\\fake_ladders",
  )?;
  let decompiled: LevelDecompiledLadder = decompile(&entity)?;
  let faces: Vec<(Vector3d, [Vector3d; 3])> = get_box_faces(&decompiled, &entity);

  assert_eq!(decompiled.object.reference, "levels\\test\\test_ladder_3");
  assert_eq!(decompiled.object.source, "level.spawn");
  assert_eq!(decompiled.object.transform.rotation, entity.header.direction);
  assert_eq!(faces.len(), 10);

  for normal in [Vector3d::Z, Vector3d::X, -&Vector3d::X, Vector3d::Y, -&Vector3d::Y] {
    assert_eq!(
      faces.iter().filter(|(face, _)| *face == normal).count(),
      2,
      "Expect two faces facing {normal:?} in {faces:?}"
    );
  }

  for corner in faces.iter().flat_map(|(_, corners)| corners) {
    assert_eq!(
      [corner.x.abs(), corner.y.abs(), corner.z.abs()],
      [0.4, 2.0, 0.1],
      "{corner:?} is no corner of the box"
    );
  }

  Ok(())
}

/// A flat ladder comes back as its front alone, facing the way its box does, its pivot the middle of its lowest end:
/// one whose box keeps the micrometres of thickness the Build's fit leaves too.
#[test]
fn decompiles_a_flat_ladder_into_its_front_standing_at_its_base() -> XrfResult {
  let entity: AlifeObject = ladder(Vector3d::new(10.0, 5.0, 3.0), Vector3d::ZERO, [0.4, 2.0, 0.000_002], "")?;
  let decompiled: LevelDecompiledLadder = decompile(&entity)?;
  let faces: Vec<(Vector3d, [Vector3d; 3])> = get_box_faces(&decompiled, &entity);

  assert_eq!(faces.len(), 2);
  assert!(faces.iter().all(|(normal, _)| *normal == Vector3d::Z), "{faces:?}");
  assert_eq!(decompiled.object.transform.position, Vector3d::new(10.0, 3.0, 3.0));
  assert!(
    get_mesh(&decompiled.object.decompile.object)
      .points
      .iter()
      .all(|point| point.y >= 0.0),
    "the ladder rises from its pivot"
  );

  Ok(())
}

/// A packet of no game material, as written before version 127, takes the ladders' own; one that keeps it is dressed
/// with it.
#[test]
fn dresses_a_ladder_with_its_game_material() -> XrfResult {
  let material = |name: &str| -> XrfResult<(String, bool)> {
    let decompiled: LevelDecompiledLadder = decompile(&ladder(Vector3d::ZERO, Vector3d::ZERO, [0.4, 2.0, 0.0], name)?)?;
    let surfaces = decompiled.object.decompile.object.surfaces.unwrap_or_default();

    Ok((surfaces[0].game_material.clone(), decompiled.is_material_inferred))
  };

  assert_eq!(
    material("")?,
    (String::from(LevelLadderConversion::DEFAULT_GAME_MATERIAL), true)
  );
  assert_eq!(material("materials\\wood")?, (String::from("materials\\wood"), false));

  Ok(())
}

/// A breakable as the Build exports one, standing at `position`.
fn breakable(position: Vector3d) -> AlifeObject {
  let ladder: AlifeObject = ladder(position, Vector3d::ZERO, [1.0; 3], "").expect("a ladder");

  AlifeObject {
    header: SpawnPacketHeader {
      section: String::from("breakable_object"),
      name: String::from("meshes\\brkbl#0.ogf"),
      ..ladder.header
    },
    clsid: ClsId::OBrkbl,
    inherited: AlifeObjectInherited::CseAlifeObjectBreakable(Box::new(AlifeObjectBreakable {
      base: AlifeObjectDynamicVisual {
        base: restrictor().state.base,
        visual_name: String::from("meshes\\brkbl#0"),
        visual_flags: 0,
      },
      health: 1.0,
    })),
  }
}

/// A library object of a ladder, an upright panel 0.8 m wide and 2 m high rising from its pivot, and a pane of glass
/// 2 m beside it, written where a placement reads it from.
fn library_ladder(path: &PathBuf) -> XrfResult {
  let dress = |material: &str| SurfaceDress {
    compiler_shader: String::from("default"),
    game_material: material.to_owned(),
  };
  let panel = |x: f32, width: f32, height: f32| -> Vec<[Vector3d; 3]> {
    let corner = |along: f32, up: f32| Vector3d::new(x + along, up, 0.0);

    vec![
      [corner(-width, 0.0), corner(width, 0.0), corner(width, height)],
      [corner(-width, 0.0), corner(width, height), corner(-width, height)],
    ]
  };
  let mut geometry: Vec<LevelVisualGeometry> = LevelVisualGeometry::of_triangles(
    "prop\\prop_ladder",
    "default",
    &dress("materials\\fake_ladders"),
    &panel(0.0, 0.4, 2.0),
  );

  geometry.extend(LevelVisualGeometry::of_triangles(
    "glass\\glass_window",
    "default",
    &dress("materials\\glass"),
    &panel(2.0, 0.5, 1.0),
  ));

  ObjectDecompiler::decompile_level_geometry(&geometry)?
    .object
    .write_to_path::<XRayByteOrder, _>(path)
}

/// A ladder and a breakable whose packets a placed library object's own faces make again are told from those it does
/// not make: the object brings them back, and the Build cuts them out of it again.
#[test]
fn tells_the_ladders_and_breakables_a_placed_library_object_makes_again() -> XrfResult {
  let path: PathBuf =
    GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "cutouts"))?.join("ladder.object");
  let mut ladders: GameMtlMaterial = GameMtlMaterial::new(7, "materials\\fake_ladders");
  let mut glass: GameMtlMaterial = GameMtlMaterial::new(9, "materials\\glass");

  ladders.flags |= GameMtlMaterial::FLAG_CLIMABLE;
  glass.flags |= GameMtlMaterial::FLAG_BREAKABLE;
  library_ladder(&path)?;

  let placement: LevelInstancePlacement = LevelInstancePlacement {
    reference: String::from("statics\\ladder"),
    path,
    object: String::from("ladder_0000"),
    meshes: Vec::new(),
    transform: EditableObjectTransform {
      position: Vector3d::new(10.0, 5.0, 3.0),
      ..EditableObjectTransform::default()
    },
    coverage: 1.0,
    error: 0.0,
    triangles: 4,
  };
  let cutouts: LevelInstanceCutouts =
    LevelInstanceCutouts::of::<XRayByteOrder>(&[placement], &GameMtlFile::new(vec![ladders, glass], Vec::new()))?;
  // The panel's middle, a metre above the placement.
  let made: Vector3d = Vector3d::new(10.0, 6.0, 3.0);

  assert!(cutouts.is_made(&ladder(made, Vector3d::new(1.2, 0.3, 0.0), [1.0, 0.0, 0.4], "")?));
  assert!(
    !cutouts.is_made(&ladder(
      Vector3d::new(10.0, 6.0, 5.0),
      Vector3d::ZERO,
      [1.0, 0.0, 0.4],
      ""
    )?),
    "a ladder standing elsewhere"
  );
  assert!(
    cutouts.is_made(&ladder(made, Vector3d::ZERO, [1.0, 0.06, 0.4], "")?),
    "a ladder GSC's object built thicker"
  );
  assert!(
    !cutouts.is_made(&ladder(made, Vector3d::ZERO, [2.0, 0.0, 0.4], "")?),
    "a ladder of another size standing there"
  );
  assert!(cutouts.is_made(&breakable(Vector3d::new(12.0, 5.5, 3.0))), "the glass");
  assert!(!cutouts.is_made(&breakable(made)), "the ladder is no breakable");
  assert!(
    !LevelInstanceCutouts::default().is_made(&ladder(made, Vector3d::ZERO, [1.0, 0.0, 0.4], "")?),
    "nothing is made where no game material library tells the faces apart"
  );

  Ok(())
}

/// An entity of another class is no ladder.
#[test]
fn leaves_other_entities() -> XrfResult {
  let entity: AlifeObject = listed(&restrictor(), 0)?.read_alife_object::<XRayByteOrder>()?;

  assert!(LevelLadderConversion::decompile("test", 0, &entity)?.is_none());

  Ok(())
}
