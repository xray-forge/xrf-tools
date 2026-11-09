use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_editable_level::{EditableObjectPart, EditableShapeElement, EditableSpawnPointKind, EditableSpawnPointObject};
use xrf_error::XrfResult;
use xrf_math::{Sphere, Vector3d};
use xrf_spawn::{
  AlifeGraphPoint, AlifeObjectSpaceRestrictor, LevelSpawnFile, LevelSpawnObject, Shape, SpawnPacket, SpawnPacketHeader,
};

use crate::level::tests::GUID;
use crate::level::tests::spawn_data::restrictor;
use crate::{LevelDecompileContext, LevelEntityConversion, LevelEntityDecompile, LevelEntityExport};

/// A packet as a spawn list holds it, under chunk `id`.
fn listed<S: ChunkReadWrite>(packet: &S, id: u32) -> XrfResult<LevelSpawnObject> {
  let mut bytes: ChunkWriter = ChunkWriter::new();

  packet.write::<XRayByteOrder>(&mut bytes)?;

  LevelSpawnObject::read::<XRayByteOrder, _>(id, &mut ChunkReader::from_bytes(&bytes.buffer)?)
}

fn restrictor_named(name: &str) -> SpawnPacket<AlifeObjectSpaceRestrictor> {
  let mut packet: SpawnPacket<AlifeObjectSpaceRestrictor> = restrictor();

  packet.header.name = String::from(name);

  packet
}

/// A graph point, a class of no volume, named as a generated shape name would be.
fn graph_point() -> SpawnPacket<AlifeGraphPoint> {
  SpawnPacket {
    header: SpawnPacketHeader {
      section: String::from("graph_point"),
      name: String::from("shape_0000"),
      position: Vector3d::new(1.5, -2.0, 4.25),
      ..restrictor().header
    },
    state: AlifeGraphPoint {
      connection_point_name: String::new(),
      connection_level_name: String::new(),
      location0: 1,
      location1: 0,
      location2: 0,
      location3: 0,
    },
  }
}

/// `level.spawn` with a restrictor and a graph point, and `level_rs.spawn` with a restrictor marked to respawn.
fn lists() -> XrfResult<(LevelSpawnFile, LevelSpawnFile)> {
  Ok((
    LevelSpawnFile {
      objects: vec![listed(&restrictor(), 0)?, listed(&graph_point(), 1)?],
    },
    LevelSpawnFile {
      objects: vec![listed(&restrictor_named("test_respawned"), 0)?],
    },
  ))
}

fn decompile(spawn: &LevelSpawnFile, respawn: &LevelSpawnFile) -> XrfResult<LevelEntityDecompile> {
  LevelEntityConversion::decompile::<XRayByteOrder>(Some(spawn), Some(respawn), &mut LevelDecompileContext::new(GUID))
}

fn bytes_of(list: Option<&LevelSpawnFile>) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  list.expect("a spawn list").write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

/// Each entity stands at its packet's place under its name, a respawning one marked so, a restrictor's volume attached
/// as a shape at the entity; no generated name takes an entity's.
#[test]
fn decompiles_entities_at_their_place_with_their_shapes_attached() -> XrfResult {
  let (spawn, respawn) = lists()?;
  let decompiled: LevelEntityDecompile = decompile(&spawn, &respawn)?;
  let [first, point, respawned] = decompiled.entities.as_slice() else {
    panic!("three entities");
  };

  assert!(decompiled.refused.is_empty());
  assert_eq!(
    (
      first.object.name.as_str(),
      point.object.name.as_str(),
      respawned.object.name.as_str()
    ),
    ("test_restrictor", "shape_0000", "test_respawned")
  );
  assert_eq!(first.object.transform.position, restrictor().header.position);
  assert_eq!(first.object.transform.rotation, restrictor().header.direction);
  assert!(point.attached.is_none(), "a graph point has no volume");

  let flags = |entity: &EditableSpawnPointObject| match &entity.kind {
    EditableSpawnPointKind::Entity { section, flags, data } => {
      assert_eq!(data.get("000002"), Some(format!("\"{section}\"").as_str()));
      *flags
    }
    _ => panic!("an entity"),
  };

  assert_eq!([flags(first), flags(point), flags(respawned)], [0, 0, 1]);

  let attached = first.attached.as_ref().expect("a restrictor's shape");

  assert_eq!(attached.object.name, "shape_0001", "the graph point keeps shape_0000");
  assert_eq!(attached.object.transform.position, first.object.transform.position);
  assert_eq!(
    attached.shape.elements,
    [EditableShapeElement::Sphere {
      center: Vector3d::ZERO,
      radius: 1.0
    }]
  );

  Ok(())
}

/// Both lists export byte for byte, each entity to the list it came from.
#[test]
fn exports_entities_back_to_their_spawn_lists() -> XrfResult {
  let (spawn, respawn) = lists()?;
  let exported: LevelEntityExport = LevelEntityConversion::export::<XRayByteOrder>(&EditableObjectPart::new(
    GUID,
    decompile(&spawn, &respawn)?.entities,
  ))?;

  assert_eq!(bytes_of(exported.spawn.as_ref())?, bytes_of(Some(&spawn))?);
  assert_eq!(bytes_of(exported.respawn.as_ref())?, bytes_of(Some(&respawn))?);

  Ok(())
}

/// The spawn point's name, place and attached shape, scaled, are what the build exports, over the packet's own.
#[test]
fn exports_the_spawn_point_name_place_and_shape_over_the_packet() -> XrfResult {
  let (spawn, respawn) = lists()?;
  let mut entities: Vec<EditableSpawnPointObject> = decompile(&spawn, &respawn)?.entities;
  let first: &mut EditableSpawnPointObject = &mut entities[0];
  let attached = first.attached.as_mut().expect("a shape");

  first.object.name = String::from("test_moved");
  first.object.transform.position = Vector3d::new(10.0, 20.0, 30.0);
  first.object.transform.rotation = Vector3d::new(0.0, 1.5, 0.0);
  attached.object.transform.scale = Vector3d::new(2.0, 2.0, 2.0);
  attached.shape.elements = vec![EditableShapeElement::Sphere {
    center: Vector3d::new(0.5, 0.0, -1.0),
    radius: 1.5,
  }];

  let exported: LevelEntityExport =
    LevelEntityConversion::export::<XRayByteOrder>(&EditableObjectPart::new(GUID, entities))?;
  let mut expected: SpawnPacket<AlifeObjectSpaceRestrictor> = restrictor_named("test_moved");

  expected.header.position = Vector3d::new(10.0, 20.0, 30.0);
  expected.header.direction = Vector3d::new(0.0, 1.5, 0.0);
  expected.state.shape = vec![Shape::Sphere(Sphere {
    center: Vector3d::new(1.0, 0.0, -2.0),
    radius: 3.0,
  })];

  assert_eq!(exported.spawn.expect("level.spawn").objects[0], listed(&expected, 0)?);

  Ok(())
}

/// A packet of no known class is left out and recorded with its list, name and section; the rest decompile.
#[test]
fn refuses_an_entity_of_no_known_class_alone() -> XrfResult {
  let mut unknown: SpawnPacket<AlifeObjectSpaceRestrictor> = restrictor_named("test_team_base");

  unknown.header.section = String::from("test_unknown_section");

  let (mut spawn, respawn) = lists()?;

  spawn.objects.insert(0, listed(&unknown, 0)?);

  let decompiled: LevelEntityDecompile = decompile(&spawn, &respawn)?;
  let [refusal] = decompiled.refused.as_slice() else {
    panic!("one refusal");
  };

  assert_eq!(decompiled.entities.len(), 3);
  assert_eq!(
    (refusal.file.as_str(), refusal.object.as_str()),
    ("level.spawn", "test_team_base")
  );
  assert!(
    refusal.reason.starts_with("section 'test_unknown_section': ") && refusal.reason.contains("no CLSID mapping"),
    "{}",
    refusal.reason
  );

  Ok(())
}

/// A class with a volume exports only with a shape attached, as the build refuses it otherwise.
#[test]
fn refuses_to_export_an_entity_of_a_volume_without_a_shape() -> XrfResult {
  let (spawn, respawn) = lists()?;
  let mut entities: Vec<EditableSpawnPointObject> = decompile(&spawn, &respawn)?.entities;

  entities[0].attached = None;

  let error: String = LevelEntityConversion::export::<XRayByteOrder>(&EditableObjectPart::new(GUID, entities))
    .unwrap_err()
    .to_string();

  assert!(
    error.contains("'test_restrictor'") && error.contains("must hold a shape attached"),
    "{error}"
  );

  Ok(())
}
