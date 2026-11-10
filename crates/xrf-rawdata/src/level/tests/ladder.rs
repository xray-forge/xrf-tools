use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableMesh, EditableObjectFile};
use xrf_error::XrfResult;
use xrf_math::{Affine3d, Vector3d};
use xrf_spawn::{
  AlifeObject, AlifeObjectAbstract, AlifeObjectClimable, AlifeObjectShape, Shape, SpawnPacket, SpawnPacketHeader,
};

use crate::level::tests::entity::listed;
use crate::level::tests::spawn_data::restrictor;
use crate::{LevelDecompiledLadder, LevelLadderConversion};

/// A ladder as the Build exports one: a box of half sizes `half` in its frame, standing at `position` and turned by
/// `direction`, of `material`.
fn ladder(position: Vector3d, direction: Vector3d, half: [f32; 3], material: &str) -> XrfResult<AlifeObject> {
  let base: AlifeObjectAbstract = restrictor().state.base;
  let packet: SpawnPacket<AlifeObjectClimable> = SpawnPacket {
    header: SpawnPacketHeader {
      section: String::from("climable_object"),
      name: String::from("clmbl#0"),
      position,
      direction,
      ..restrictor().header
    },
    state: AlifeObjectClimable {
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
    },
  };

  listed(&packet, 0)?.read_alife_object::<XRayByteOrder>()
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

/// A flat ladder comes back as its front alone, facing the way its box does, its pivot the middle of its lowest end.
#[test]
fn decompiles_a_flat_ladder_into_its_front_standing_at_its_base() -> XrfResult {
  let entity: AlifeObject = ladder(Vector3d::new(10.0, 5.0, 3.0), Vector3d::ZERO, [0.4, 2.0, 0.0], "")?;
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

/// An entity of another class is no ladder.
#[test]
fn leaves_other_entities() -> XrfResult {
  let entity: AlifeObject = listed(&restrictor(), 0)?.read_alife_object::<XRayByteOrder>()?;

  assert!(LevelLadderConversion::decompile("test", 0, &entity)?.is_none());

  Ok(())
}
