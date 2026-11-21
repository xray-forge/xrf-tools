use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_editable_level::{EditableObjectPart, EditableShapeElement, EditableSpawnPointKind, EditableSpawnPointObject};
use xrf_error::XrfResult;
use xrf_math::{Sphere, Vector3d};
use xrf_spawn::{
  AlifeGraphPoint, AlifeObject, AlifeObjectSpaceRestrictor, LevelSpawnFile, LevelSpawnObject, Shape, SpawnPacket,
  SpawnPacketHeader,
};

use crate::level::entities::level_entity_conversion::LevelEntityConversion;
use crate::level::entities::level_entity_decompile::LevelEntityDecompile;
use crate::level::entities::level_entity_export::LevelEntityExport;
use crate::level::entities::level_entity_sources::LevelEntitySources;
use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::tests::fixtures::GUID;
use crate::level::tests::spawn_data::restrictor;

pub(super) fn listed<S: ChunkReadWrite>(packet: &S, id: u32) -> XrfResult<LevelSpawnObject> {
  let mut bytes: ChunkWriter = ChunkWriter::new();

  packet.write(&mut bytes)?;

  LevelSpawnObject::read(id, &mut ChunkReader::from_bytes(&bytes.buffer)?)
}

fn restrictor_named(name: &str) -> SpawnPacket<AlifeObjectSpaceRestrictor> {
  let mut packet: SpawnPacket<AlifeObjectSpaceRestrictor> = restrictor();

  packet.header.name = String::from(name);

  packet
}

// Named as a generated shape name would be.
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
  decompile_with_root(spawn, respawn, &[])
}

fn decompile_with_root(
  spawn: &LevelSpawnFile,
  respawn: &LevelSpawnFile,
  root: &[AlifeObject],
) -> XrfResult<LevelEntityDecompile> {
  LevelEntityConversion::decompile(
    &LevelEntitySources {
      spawn: Some(spawn),
      respawn: Some(respawn),
      root,
      folded: &[],
    },
    &mut LevelDecompileContext::new(GUID),
  )
}

// As xrAI builds it into a root spawn: its spawn id, vertices and distance filled in.
fn built(packet: &SpawnPacket<AlifeObjectSpaceRestrictor>) -> XrfResult<AlifeObject> {
  let mut object: AlifeObject = listed(packet, 0)?.read_alife_object()?;
  let base = object.inherited.get_abstract_mut().expect("an abstract object");

  base.game_vertex_id = 17;
  base.distance = 2.5;
  base.level_vertex_id = 99;
  object.header.spawn_id = 5;

  Ok(object)
}

fn bytes_of(list: Option<&LevelSpawnFile>) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  list.expect("a spawn list").write(&mut writer)?;

  Ok(writer.buffer)
}

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

#[test]
fn exports_entities_back_to_their_spawn_lists() -> XrfResult {
  let (spawn, respawn) = lists()?;
  let exported: LevelEntityExport =
    LevelEntityConversion::export(&EditableObjectPart::new(GUID, decompile(&spawn, &respawn)?.entities))?;

  assert_eq!(bytes_of(exported.spawn.as_ref())?, bytes_of(Some(&spawn))?);
  assert_eq!(bytes_of(exported.respawn.as_ref())?, bytes_of(Some(&respawn))?);

  Ok(())
}

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

  let exported: LevelEntityExport = LevelEntityConversion::export(&EditableObjectPart::new(GUID, entities))?;
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

// The build refuses it otherwise.
#[test]
fn refuses_to_export_an_entity_of_a_volume_without_a_shape() -> XrfResult {
  let (spawn, respawn) = lists()?;
  let mut entities: Vec<EditableSpawnPointObject> = decompile(&spawn, &respawn)?.entities;

  entities[0].attached = None;

  let error: String = LevelEntityConversion::export(&EditableObjectPart::new(GUID, entities))
    .unwrap_err()
    .to_string();

  assert!(
    error.contains("'test_restrictor'") && error.contains("must hold a shape attached"),
    "{error}"
  );

  Ok(())
}

// The fields xrAI filled in are reset, so an object as built exports to the editor's packet.
#[test]
fn takes_a_root_spawns_objects_in_place_of_the_level_spawn_list() -> XrfResult {
  let (mut spawn, respawn) = lists()?;

  spawn.objects.push(listed(&restrictor_named("test_removed"), 2)?);

  let root: [AlifeObject; 2] = [built(&restrictor_named("test_added"))?, built(&restrictor())?];
  let decompiled: LevelEntityDecompile = decompile_with_root(&spawn, &respawn, &root)?;
  let names: Vec<&str> = decompiled
    .entities
    .iter()
    .map(|entity| entity.object.name.as_str())
    .collect();

  assert_eq!(names, ["test_restrictor", "shape_0000", "test_added", "test_respawned"]);
  assert_eq!(
    decompiled
      .disagreements
      .iter()
      .map(|disagreement| (
        disagreement.object.as_str(),
        disagreement.reason.split(',').next().unwrap_or_default()
      ))
      .collect::<Vec<_>>(),
    [
      ("test_removed", "only in level.spawn"),
      ("test_added", "only in the root spawn"),
    ]
  );
  assert!(
    decompiled.update_packets.is_empty(),
    "a restrictor's update packet is empty"
  );

  let exported: LevelEntityExport = LevelEntityConversion::export(&EditableObjectPart::new(GUID, decompiled.entities))?;
  let objects: Vec<LevelSpawnObject> = exported.spawn.expect("level.spawn").objects;

  assert_eq!(
    objects[..2],
    spawn.objects[..2],
    "the built restrictor exports as the editor wrote it"
  );
  assert_eq!(objects[2], listed(&restrictor_named("test_added"), 2)?);

  Ok(())
}

#[test]
fn leaves_out_entities_folded_into_objects() -> XrfResult {
  let (spawn, respawn) = lists()?;
  let root: [AlifeObject; 2] = [built(&restrictor())?, built(&restrictor_named("test_added"))?];
  let folded: [String; 2] = [String::from("test_restrictor"), String::from("test_respawned")];
  let decompile_folded = |root: &[AlifeObject]| {
    LevelEntityConversion::decompile(
      &LevelEntitySources {
        spawn: Some(&spawn),
        respawn: Some(&respawn),
        root,
        folded: &folded,
      },
      &mut LevelDecompileContext::new(GUID),
    )
  };
  let names = |decompiled: &LevelEntityDecompile| -> Vec<String> {
    decompiled
      .entities
      .iter()
      .map(|entity| entity.object.name.clone())
      .collect()
  };

  assert_eq!(names(&decompile_folded(&[])?), ["shape_0000"]);

  let from_root: LevelEntityDecompile = decompile_folded(&root)?;

  assert_eq!(names(&from_root), ["shape_0000", "test_added"]);
  assert_eq!(
    from_root
      .disagreements
      .iter()
      .map(|disagreement| disagreement.object.as_str())
      .collect::<Vec<_>>(),
    ["test_added"]
  );

  Ok(())
}

#[test]
fn records_an_object_the_root_spawn_changed_past_its_build() -> XrfResult {
  let (spawn, respawn) = lists()?;
  let mut edited: SpawnPacket<AlifeObjectSpaceRestrictor> = restrictor();

  edited.state.restrictor_type = 2;

  let decompiled: LevelEntityDecompile = decompile_with_root(&spawn, &respawn, &[built(&edited)?])?;

  assert_eq!(decompiled.entities[0].object.name, "test_restrictor");
  assert_eq!(
    decompiled
      .disagreements
      .iter()
      .map(|disagreement| disagreement.object.as_str())
      .collect::<Vec<_>>(),
    ["test_restrictor"]
  );
  assert!(decompiled.disagreements[0].reason.contains("spawn packet differs"));

  Ok(())
}
