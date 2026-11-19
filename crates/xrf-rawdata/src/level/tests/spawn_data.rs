use xrf_chunk::{
  ChunkField, ChunkFieldReadWrite, ChunkFieldRecorder, ChunkFieldReplay, ChunkReadWrite, ChunkWriter, XRayByteOrder,
};
use xrf_error::XrfResult;
use xrf_ini::{IniFile, IniFormat, IniSection};
use xrf_math::{Sphere, Vector3d};
use xrf_spawn::{
  AlifeObject, AlifeObjectAbstract, AlifeObjectBreakable, AlifeObjectDynamicVisual, AlifeObjectInherited,
  AlifeObjectSpaceRestrictor, ClsId, Shape, SpawnPacket, SpawnPacketHeader,
};

use crate::level::entities::level_spawn_data_fields::LevelSpawnDataFields;

const SECTION: &str = "object_1_spawndata";

pub(super) fn restrictor() -> SpawnPacket<AlifeObjectSpaceRestrictor> {
  SpawnPacket {
    header: SpawnPacketHeader {
      id: u16::MAX,
      net_action: 1,
      section: String::from("space_restrictor"),
      name: String::from("test_restrictor"),
      script_game_id: 0,
      script_rp: 254,
      position: Vector3d::new(-55.345_585, -22.013_443, 93.112_25),
      direction: Vector3d::new(0.0, -0.0, 0.0),
      respawn_time: 0,
      parent_id: u16::MAX,
      phantom_id: u16::MAX,
      script_flags: 33,
      version: 128,
      game_type: u16::MAX,
      script_version: 12,
      client_data: Vec::new(),
      spawn_id: u16::MAX,
    },
    state: AlifeObjectSpaceRestrictor {
      base: AlifeObjectAbstract {
        game_vertex_id: u16::MAX,
        distance: 0.0,
        direct_control: 1,
        level_vertex_id: u32::MAX,
        flags: 0xFFFF_FF3E,
        custom_data: String::from("[logic]\r\ncfg = scripts\\test_restrictor.ltx"),
        story_id: u32::MAX,
        spawn_story_id: u32::MAX,
      },
      shape: vec![Shape::Sphere(Sphere {
        center: Vector3d::ZERO,
        radius: 1.0,
      })],
      restrictor_type: 3,
    },
  }
}

fn write_text(packet: &SpawnPacket<AlifeObjectSpaceRestrictor>) -> XrfResult<String> {
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();
  let mut ini: IniFile = IniFile::default();

  packet.write_fields::<XRayByteOrder, _>(&mut recorder)?;
  LevelSpawnDataFields::write_section(&recorder.into_fields(), ini.section_mut(SECTION))?;

  ini.write_to_string_as(&IniFormat::XRAY_SAVE_AS)
}

fn read_text(text: &str) -> XrfResult<SpawnPacket<AlifeObjectSpaceRestrictor>> {
  let ini: IniFile = IniFile::read_from_str_as(text, &IniFormat::XRAY_SAVE_AS)?;
  let section: &IniSection = ini.section_required(SECTION)?;
  let mut fields: LevelSpawnDataFields<'_> = LevelSpawnDataFields::new(section);
  let packet: SpawnPacket<AlifeObjectSpaceRestrictor> =
    SpawnPacket::read_fields::<XRayByteOrder, _>(&mut ChunkFieldReplay::new(&mut fields))?;

  fields.assert_read()?;

  Ok(packet)
}

#[test]
fn writes_spawn_data_as_the_editor_prints_it() -> XrfResult {
  let text: String = write_text(&restrictor())?;
  let lines: Vec<&str> = text.lines().map(str::trim_start).collect();

  for expected in [
    "000001                           = 1",
    "000002                           = \"space_restrictor\"",
    "000005                           = 254",
    "000006                           = -55.345585, -22.013443, 93.112251",
    "000007                           = 0.000000, -0.000000, 0.000000",
    "000018                           = 0",
    "000020                           = 0.000000",
    "000022                           = -1",
    "000023                           = -194",
    "000024                           = \"[logic]",
    "cfg = scripts\\test_restrictor.ltx\"",
    "000029                           = 0.000000, 0.000000, 0.000000",
    "000031                           = 3",
  ] {
    assert!(lines.contains(&expected), "Expect line '{expected}' in\n{text}");
  }

  assert!(!text.contains("000032"), "Expect 31 fields in\n{text}");

  Ok(())
}

#[test]
fn reads_spawn_data_back_into_the_same_packet() -> XrfResult {
  let packet: SpawnPacket<AlifeObjectSpaceRestrictor> = restrictor();
  let read: SpawnPacket<AlifeObjectSpaceRestrictor> = read_text(&write_text(&packet)?)?;
  let write_binary = |packet: &SpawnPacket<AlifeObjectSpaceRestrictor>| -> XrfResult<Vec<u8>> {
    let mut bytes: ChunkWriter = ChunkWriter::new();

    packet.write::<XRayByteOrder>(&mut bytes)?;

    Ok(bytes.buffer)
  };

  assert_eq!(read, packet);
  assert_eq!(write_binary(&read)?, write_binary(&packet)?);

  Ok(())
}

// A field the packet does not take belongs to another layout than the class read.
#[test]
fn refuses_spawn_data_past_the_packet() -> XrfResult {
  let mut ini: IniFile = IniFile::read_from_str_as(&write_text(&restrictor())?, &IniFormat::XRAY_SAVE_AS)?;

  ini.section_mut(SECTION).set(
    LevelSpawnDataFields::format_key(32),
    LevelSpawnDataFields::format_value(&ChunkField::U8(0)),
  );

  let error: String = read_text(&ini.write_to_string_as(&IniFormat::XRAY_SAVE_AS)?)
    .expect_err("a field past the packet's last")
    .to_string();

  assert!(error.contains("holds field 000032 past the packet's last"), "{error}");

  Ok(())
}

fn breakable(visual: &str) -> AlifeObject {
  AlifeObject {
    header: SpawnPacketHeader {
      section: String::from("breakable_object"),
      name: String::from("box_0000"),
      ..restrictor().header
    },
    clsid: ClsId::OBrkbl,
    inherited: AlifeObjectInherited::CseAlifeObjectBreakable(Box::new(AlifeObjectBreakable {
      base: AlifeObjectDynamicVisual {
        base: restrictor().state.base,
        visual_name: String::from(visual),
        visual_flags: 0,
      },
      health: 1.0,
    })),
  }
}

fn write_entity(entity: &AlifeObject) -> XrfResult<IniSection> {
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();
  let mut section: IniSection = IniSection::new("spawndata");

  entity.write_spawn_packet::<XRayByteOrder, _>(&mut recorder)?;
  LevelSpawnDataFields::write_section(&recorder.into_fields(), &mut section)?;

  Ok(section)
}

#[test]
fn reads_the_visual_a_packet_names() -> XrfResult {
  assert_eq!(
    LevelSpawnDataFields::read_visual::<XRayByteOrder>(&write_entity(&breakable("dynamics\\box\\box_wood_01"))?)?,
    Some(String::from("dynamics\\box\\box_wood_01"))
  );
  assert_eq!(
    LevelSpawnDataFields::read_visual::<XRayByteOrder>(&write_entity(&breakable(""))?)?,
    None,
    "a packet naming no visual"
  );

  let mut restrictor_data: IniSection = IniSection::new("spawndata");
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();

  restrictor().write_fields::<XRayByteOrder, _>(&mut recorder)?;
  LevelSpawnDataFields::write_section(&recorder.into_fields(), &mut restrictor_data)?;

  assert_eq!(
    LevelSpawnDataFields::read_visual::<XRayByteOrder>(&restrictor_data)?,
    None,
    "a restrictor is drawn as nothing"
  );

  Ok(())
}

// The build refuses it too.
#[test]
fn refuses_the_visual_of_a_packet_that_does_not_read() -> XrfResult {
  let mut data: IniSection = write_entity(&breakable("dynamics\\box\\box_wood_01"))?;

  data.set(
    LevelSpawnDataFields::format_key(data.fields().len() + 1),
    LevelSpawnDataFields::format_value(&ChunkField::U8(0)),
  );

  assert!(LevelSpawnDataFields::read_visual::<XRayByteOrder>(&data).is_err());
  assert!(LevelSpawnDataFields::read_visual::<XRayByteOrder>(&IniSection::new("spawndata")).is_err());

  Ok(())
}

// A double quote would end its quoted value early.
#[test]
fn refuses_a_string_holding_a_double_quote() {
  let mut packet: SpawnPacket<AlifeObjectSpaceRestrictor> = restrictor();

  packet.state.base.custom_data = String::from("[logic]\r\nactive = \"quoted\"");

  let error: String = write_text(&packet).expect_err("a quoted string").to_string();

  assert!(error.contains("holds a double quote"), "{error}");
}
