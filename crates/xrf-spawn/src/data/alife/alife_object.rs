use std::io::Write;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{
  ChunkDataSource, ChunkFieldReader, ChunkFieldReplay, ChunkFieldWriter, ChunkReadWrite, ChunkReader, ChunkWriter,
};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxWrite};
use xrf_math::Vector3d;
use xrf_utils::{assert_equal, assert_not_equal, narrow_format_size};

use crate::constants::NET_ACTION_UPDATE;
use crate::data::alife::alife_built_fields::AlifeBuiltFields;
use crate::data::alife::alife_object_inherited::AlifeObjectInherited;
use crate::data::alife::script_dialect::ScriptDialect;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
use crate::data::alife::zeroed_update_fields::ZeroedUpdateFields;
use crate::data::meta::alife_class::AlifeClass;
use crate::data::meta::cls_id::ClsId;

/// An object of a spawn set (`all.spawn`): its spawn packet, the spawn packet header's fields then the class's spawn
/// fields, and its update packet, the class's update fields, each in a chunk of its own behind a `u16` length.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObject {
  /// What the spawn packet opens with, `CSE_Abstract::Spawn_Write`'s own fields; they sit beside the object's in
  /// every serialized form.
  #[serde(flatten)]
  pub header: SpawnPacketHeader,
  pub clsid: ClsId,
  /// The server class payload, its fields from both packets. The engine applies the update packet over the spawn
  /// packet when it creates the object, so where both carry a field the update packet's is the one the game uses.
  pub inherited: AlifeObjectInherited,
}

impl AlifeObject {
  pub const DATA_SPAWN_CHUNK_ID: u32 = 0;
  pub const DATA_UPDATE_CHUNK_ID: u32 = 1;

  /// One framed object chunk: its own header, plus the header and payload of the index chunk it must carry.
  pub const MIN_SERIALIZED_SIZE: u64 = 8 + 8 + 2;

  /// Move the object to `position`, facing `direction` as the spawn packet's angles give it, in every packet that
  /// stores where it stands. `CALifeSimulatorBase::create` reads the update packet over the spawn packet, so an object
  /// moved in the spawn packet alone would start where its update packet still says.
  pub fn move_to(&mut self, position: Vector3d, direction: Vector3d) {
    self.inherited.move_to(&position, &direction);
    self.header.position = position;
    self.header.direction = direction;
  }

  /// The fields xrAI fills in when it builds the object into a root spawn ([`AlifeBuiltFields`]).
  pub fn get_built_fields(&self) -> AlifeBuiltFields {
    self.inherited.get_built_fields(self.header.spawn_id)
  }

  /// Sets the fields xrAI fills in, such as another packet of the object holds them or as
  /// [`AlifeBuiltFields::UNBUILT`] before any build; the update packet is left as it is.
  pub fn set_built_fields(&mut self, fields: &AlifeBuiltFields) {
    self.header.spawn_id = fields.spawn_id;
    self.inherited.set_built_fields(fields);
  }

  /// Write the object's update packet alone, its class's update fields, as a root spawn stores it after its length and
  /// `M_UPDATE`.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload is not its section's class, or the writer refuses a value.
  pub fn write_update_packet<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    self.write_packets::<T, _, _>(&mut ChunkWriter::new(), writer)
  }

  /// The object as a fresh server entity holds it, read from its own spawn packet ([`Self::read_spawn_packet`]): its
  /// update fields derived from the spawn packet, or zero where the engine leaves them uninitialised. A root spawn xrAI
  /// builds anew stores this object's update packet.
  ///
  /// # Errors
  ///
  /// Returns an error when the object's spawn packet does not write, or does not read back as its class.
  pub fn get_fresh_entity<T: ByteOrder>(&self) -> XrfResult<Self> {
    let mut spawn: ChunkWriter = ChunkWriter::new();

    self.write_spawn_packet::<T, _>(&mut spawn)?;

    Self::read_spawn_packet::<T, _>(&mut ChunkReader::from_bytes(&spawn.buffer)?)
  }

  /// Refuse an object whose payload is shaped for another class than its section's, which the engine would read
  /// with the section's class in the game whose scripts its spawn version names: a Clear Sky script class's payload
  /// under a Call of Pripyat header among them, and the reverse.
  fn assert_class_matches_section(&self) -> XrfResult {
    let dialect: ScriptDialect = SpawnLayout::from(&self.header).script_dialect();
    let clsid: ClsId = ClsId::from_dialect_section(&self.header.section, dialect)?;
    let class: AlifeClass = AlifeClass::from_dialect_cls_id(&clsid, dialect);

    if clsid != self.clsid || class != self.inherited.get_class() {
      return Err(XrfError::new_invalid_error(format!(
        "ALife object '{}' of section '{}' ({clsid:?}, {class}) carries {:?} and a {} payload",
        self.header.name,
        self.header.section,
        self.clsid,
        self.inherited.get_class()
      )));
    }

    Ok(())
  }

  /// Read an object from its spawn packet alone, as `level.spawn` holds it and the Level Editor saves it, bytes or
  /// fields: the update fields as xrAI's freshly read server entity holds them when it writes a root spawn
  /// (`AlifeObjectInherited::set_spawned_update`), the members the engine leaves uninitialised zero.
  ///
  /// # Errors
  ///
  /// Returns an error when the packet is not a spawn packet of a known class, or its state does not read whole.
  pub fn read_spawn_packet<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    let mut object: Self = Self::read_packets::<T, _, _>(reader, &mut ChunkFieldReplay::new(&mut ZeroedUpdateFields))?;

    object.inherited.set_spawned_update(&object.header);

    Ok(object)
  }

  /// Write the object's spawn packet alone, as `level.spawn` holds it and the Level Editor saves it.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload is not its section's class, or the writer refuses a value.
  pub fn write_spawn_packet<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    self.write_packets::<T, _, _>(writer, &mut ChunkWriter::new())
  }

  /// The spawn packet from `spawn`, its class's update fields from `update`.
  fn read_packets<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
  ) -> XrfResult<Self> {
    let header: SpawnPacketHeader = spawn.read_xr::<T, _>()?;
    let layout: SpawnLayout = SpawnLayout::from(&header);
    let dialect: ScriptDialect = layout.script_dialect();
    let clsid: ClsId = ClsId::from_dialect_section(&header.section, dialect)?;
    let class: AlifeClass = AlifeClass::from_dialect_cls_id(&clsid, dialect);

    assert_not_equal(&class, &AlifeClass::Unknown, "Expect known ALife object clsid")?;

    let inherited: AlifeObjectInherited = header.read_state::<T, _, _>(spawn, |spawn| {
      AlifeObjectInherited::read::<T, _, _>(spawn, update, &class, layout)
    })?;

    Ok(Self {
      header,
      clsid,
      inherited,
    })
  }

  /// The spawn packet into `spawn`, its class's update fields into `update`.
  fn write_packets<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
  ) -> XrfResult {
    self.assert_class_matches_section()?;
    spawn.write_xr::<T, _>(&self.header)?;
    SpawnPacketHeader::write_state::<T, _>(spawn, |spawn| {
      self
        .inherited
        .write::<T, _, _>(spawn, update, SpawnLayout::from(&self.header))
    })
  }
}

impl ChunkReadWrite for AlifeObject {
  /// Read the object's two chunks: the spawn packet behind its length, then the update packet behind its length and
  /// `M_UPDATE`, each read whole.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut spawn_reader: ChunkReader<D> = reader.read_child_by_index(Self::DATA_SPAWN_CHUNK_ID)?;
    let data_length: u16 = spawn_reader.read_u16::<T>()?;

    assert_equal(
      u64::from(data_length) + 2,
      spawn_reader.size,
      "Expect correct data size for ALife object",
    )?;

    let mut update_reader: ChunkReader<D> = reader.read_child_by_index(Self::DATA_UPDATE_CHUNK_ID)?;
    let update_data_length: u16 = update_reader.read_u16::<T>()?;
    let update_action: u16 = update_reader.read_u16::<T>()?;

    assert_equal(
      u64::from(update_data_length) + 2,
      update_reader.size,
      "Expect correct size of ALife object update data",
    )?;
    assert_equal(
      update_action,
      NET_ACTION_UPDATE,
      "Expect only net_update actions in ALife object update data",
    )?;

    let object: Self = Self::read_packets::<T, _, _>(&mut spawn_reader, &mut update_reader)?;

    spawn_reader.assert_read("Expect ALife object spawn data to be read")?;
    update_reader.assert_read("Expect ALife object update data to hold only its class's update fields")?;
    reader.assert_read("Expect ALife object chunk to be read")?;

    Ok(object)
  }

  /// Write the object's two chunks: the spawn packet, `Spawn_Write`'s header and state, and the update packet, each
  /// behind its length.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut spawn: ChunkWriter = ChunkWriter::new();
    let mut update: ChunkWriter = ChunkWriter::new();

    self.write_packets::<T, _, _>(&mut spawn, &mut update)?;

    let mut spawn_chunk: ChunkWriter = ChunkWriter::new();
    let mut update_chunk: ChunkWriter = ChunkWriter::new();

    spawn_chunk.write_u16::<T>(narrow_format_size(spawn.bytes_written(), "alife object spawn data")?)?;
    spawn_chunk.write_all(&spawn.buffer)?;
    update_chunk.write_u16::<T>(narrow_format_size(
      update.bytes_written() + size_of::<u16>(),
      "alife object update data",
    )?)?;
    update_chunk.write_u16::<T>(NET_ACTION_UPDATE)?;
    update_chunk.write_all(&update.buffer)?;

    writer.write_all(&spawn_chunk.flush_chunk_into_buffer::<T>(Self::DATA_SPAWN_CHUNK_ID)?)?;
    writer.write_all(&update_chunk.flush_chunk_into_buffer::<T>(Self::DATA_UPDATE_CHUNK_ID)?)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObject {
  /// Import ALife object data from ltx file section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L) -> XrfResult<Self> {
    let header: SpawnPacketHeader = SpawnPacketHeader::import(section_name, ltx)?;
    let layout: SpawnLayout = SpawnLayout::from(&header);
    let dialect: ScriptDialect = layout.script_dialect();
    let clsid: ClsId = ClsId::from_dialect_section(&header.section, dialect)?;
    let inherited: AlifeObjectInherited = AlifeObjectInherited::import(
      section_name,
      ltx,
      &AlifeClass::from_dialect_cls_id(&clsid, dialect),
      layout,
    )?;

    Ok(Self {
      header,
      clsid,
      inherited,
    })
  }

  /// Export ALife object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite) -> XrfResult {
    self.header.export(section_name, ltx)?;
    self
      .inherited
      .export(section_name, ltx, SpawnLayout::from(&self.header))
  }
}

#[cfg(test)]
mod tests {
  use std::fs::{self, File};
  use std::io::{Seek, SeekFrom, Write, read_to_string};
  use std::path::PathBuf;

  use byteorder::{ByteOrder, WriteBytesExt};
  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::{XrfError, XrfResult};
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::constants::NET_ACTION_UPDATE;
  use crate::data::alife::alife_built_fields::AlifeBuiltFields;
  use crate::data::alife::alife_object::AlifeObject;
  use crate::data::alife::alife_object_inherited::AlifeObjectInherited;
  use crate::data::alife::inherited::alife_actor::AlifeActor;
  use crate::data::alife::inherited::alife_graph_point::AlifeGraphPoint;
  use crate::data::alife::inherited::alife_monster::AlifeMonster;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
  use crate::data::alife::inherited::alife_object_item_custom_outfit::AlifeObjectItemCustomOutfit;
  use crate::data::alife::inherited::alife_object_item_document::AlifeObjectItemDocument;
  use crate::data::alife::inherited::alife_object_item_torch::AlifeObjectItemTorch;
  use crate::data::alife::inherited::alife_object_item_weapon::AlifeObjectItemWeapon;
  use crate::data::alife::inherited::alife_object_online_offline_group::AlifeObjectOnlineOfflineGroup;
  use crate::data::alife::inherited::alife_object_ph_skeleton_object::AlifeObjectPhSkeletonObject;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
  use crate::data::alife::inherited::alife_object_stationary_mgun::AlifeObjectStationaryMgun;
  use crate::data::alife::inherited::alife_squad::AlifeSquad;
  use crate::data::alife::inherited::alife_stalker::AlifeStalker;
  use crate::data::alife::spawn_layout::SpawnLayout;
  use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
  use crate::data::meta::alife_class::AlifeClass;
  use crate::data::meta::cls_id::ClsId;
  use crate::data::physics::physics_net_state::PhysicsNetState;
  use crate::data::physics::raw_float::RawFloat;
  use crate::fixtures::{ClearSkyStates, GameWrittenPackets};

  /// A whole spawn packet header naming a section no class reads.
  fn new_unknown_section_header() -> SpawnPacketHeader {
    let mut header: SpawnPacketHeader = new_graph_point_object().header;

    header.section = String::from("unknown_section");

    header
  }

  #[test]
  fn binary_read_returns_error_for_unknown_section() -> XrfResult {
    let mut object_data_writer: ChunkWriter = ChunkWriter::new();

    new_unknown_section_header().write::<XRayByteOrder>(&mut object_data_writer)?;

    let mut spawn_data_writer: ChunkWriter = ChunkWriter::new();
    spawn_data_writer.write_u16::<XRayByteOrder>(object_data_writer.bytes_written() as u16)?;
    spawn_data_writer.write_all(&object_data_writer.flush_raw_into_buffer()?)?;

    // An empty update packet: its length, then `M_UPDATE`.
    let mut update_data_writer: ChunkWriter = ChunkWriter::new();
    update_data_writer.write_u16::<XRayByteOrder>(2)?;
    update_data_writer.write_u16::<XRayByteOrder>(NET_ACTION_UPDATE)?;

    let mut bytes: Vec<u8> = spawn_data_writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?;

    bytes.extend(update_data_writer.flush_chunk_into_buffer::<XRayByteOrder>(1)?);

    let path: PathBuf = GeneratedResources::write("alife_object/unknown_clsid.chunk", bytes)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_file(File::open(&path)?)?;
    let result: XrfResult<AlifeObject> = AlifeObject::read::<XRayByteOrder, _>(&mut reader);

    drop(reader);
    fs::remove_file(path)?;

    let error: XrfError = result.unwrap_err();

    assert_eq!(
      error.to_string(),
      "Parsing error: Unknown ALife object section 'unknown_section', no CLSID mapping exists"
    );

    Ok(())
  }

  #[test]
  fn ltx_import_returns_error_for_unknown_section() -> XrfResult {
    let mut ltx: Ltx = Ltx::new();

    new_unknown_section_header().export("object", &mut ltx)?;

    let error: XrfError = AlifeObject::import("object", &ltx).unwrap_err();

    assert_eq!(
      error.to_string(),
      "Parsing error: Unknown ALife object section 'unknown_section', no CLSID mapping exists"
    );

    Ok(())
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = GeneratedResources::sample_name(file!(), "read_write.chunk");

    let original: AlifeObject = AlifeObject {
      header: SpawnPacketHeader {
        id: 340,
        net_action: 1,
        section: String::from("dolg_heavy_outfit"),
        name: String::from("test-outfit-object"),
        script_game_id: 2,
        script_rp: 3,
        position: Vector3d::new(1.0, 2.0, 3.0),
        direction: Vector3d::new(3.0, 2.0, 1.0),
        respawn_time: 50000,
        parent_id: 2143,
        phantom_id: 0,
        script_flags: 33,
        version: 128,
        game_type: 1,
        script_version: 10,
        client_data: Vec::new(),
        spawn_id: 2354,
      },
      clsid: ClsId::EStlk,
      inherited: AlifeObjectInherited::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit {
        base: AlifeObjectItem {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 12434,
              distance: 124.33,
              direct_control: 624345,
              level_vertex_id: 48528,
              flags: 34,
              custom_data: String::from("custom-data"),
              story_id: 523,
              spawn_story_id: 2865268,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 0,
          },
          condition: 1.0,
          upgrades: Vec::new(),
          update_state: Some(PhysicsNetState::new_mock(Vector3d::new(1.0, 2.0, 3.0))),
        },
        update_condition: 200,
      })),
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    // The outfit's update packet is its physics state, then its quantised condition: 55 bytes past its action.
    assert_eq!(writer.bytes_written(), 223);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(&filename)?, 0)?;

    assert_eq!(bytes_written, 223);

    let file: FileSlice = GeneratedResources::open_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 223 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObject::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = GeneratedResources::sample_name(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObject = AlifeObject {
      header: SpawnPacketHeader {
        id: 345,
        net_action: 1,
        section: String::from("dolg_heavy_outfit"),
        name: String::from("test-outfit-object"),
        script_game_id: 3,
        script_rp: 3,
        position: Vector3d::new(1.0, 2.0, 3.0),
        direction: Vector3d::new(3.0, 2.0, 1.0),
        respawn_time: 50000,
        parent_id: 2143,
        phantom_id: 0,
        script_flags: 33,
        version: 128,
        game_type: 1,
        script_version: 10,
        client_data: Vec::new(),
        spawn_id: 2354,
      },
      clsid: ClsId::EStlk,
      inherited: AlifeObjectInherited::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit {
        base: AlifeObjectItem {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 12,
              distance: 100.0,
              direct_control: 52,
              level_vertex_id: 364,
              flags: 33,
              custom_data: String::from("custom-data"),
              story_id: 523,
              spawn_story_id: 2865268,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 0,
          },
          condition: 0.2,
          upgrades: Vec::new(),
          update_state: Some(PhysicsNetState::new_mock(Vector3d::new(1.0, 2.0, 3.0))),
        },
        update_condition: 200,
      })),
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut GeneratedResources::overwrite(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(GeneratedResources::path(&ltx_filename))?;

    assert_eq!(AlifeObject::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObject = AlifeObject {
      header: SpawnPacketHeader {
        id: 341,
        net_action: 1,
        section: String::from("dolg_heavy_outfit"),
        name: String::from("test-outfit-object"),
        script_game_id: 2,
        script_rp: 3,
        position: Vector3d::new_mock(),
        direction: Vector3d::new_mock(),
        respawn_time: 50000,
        parent_id: 2143,
        phantom_id: 0,
        script_flags: 33,
        version: 128,
        game_type: 1,
        script_version: 10,
        client_data: Vec::new(),
        spawn_id: 2354,
      },
      clsid: ClsId::EStlk,
      inherited: AlifeObjectInherited::CseAlifeItemCustomOutfit(Box::new(AlifeObjectItemCustomOutfit {
        base: AlifeObjectItem {
          base: AlifeObjectDynamicVisual {
            base: AlifeObjectAbstract {
              game_vertex_id: 145,
              distance: 52.33,
              direct_control: 256,
              level_vertex_id: 3634,
              flags: 24,
              custom_data: String::from("custom-data"),
              story_id: 414,
              spawn_story_id: 124,
            },
            visual_name: String::from("visual-name"),
            visual_flags: 2,
          },
          condition: 1.0,
          upgrades: Vec::new(),
          update_state: Some(PhysicsNetState::new_mock(Vector3d::new(1.0, 2.0, 3.0))),
        },
        update_condition: 200,
      })),
    };

    let mut file: File =
      GeneratedResources::overwrite(&GeneratedResources::sample_name(file!(), "serialize_deserialize.json"))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_to_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObject>(&serialized)?, original);

    Ok(())
  }
  /// The smallest object the reader accepts: a graph point carries two strings and four bytes.
  fn new_graph_point_object() -> AlifeObject {
    AlifeObject {
      header: SpawnPacketHeader {
        id: 342,
        net_action: 1,
        section: String::from("graph_point"),
        name: String::from("test-graph-point"),
        script_game_id: 2,
        script_rp: 3,
        position: Vector3d::new(1.0, 2.0, 3.0),
        direction: Vector3d::new(3.0, 2.0, 1.0),
        respawn_time: 0,
        parent_id: 2143,
        phantom_id: 0,
        script_flags: 33,
        version: 128,
        game_type: 1,
        script_version: 10,
        client_data: Vec::new(),
        spawn_id: 2355,
      },
      clsid: ClsId::AiGraph,
      inherited: AlifeObjectInherited::CseAlifeGraphPoint(Box::new(AlifeGraphPoint {
        connection_point_name: String::from("connection-point"),
        connection_level_name: String::from("connection-level"),
        location0: 1,
        location1: 2,
        location2: 3,
        location3: 4,
      })),
    }
  }

  #[test]
  fn rejects_inherited_data_past_its_size_field() {
    let mut original: AlifeObject = new_graph_point_object();

    original.inherited = AlifeObjectInherited::CseAlifeGraphPoint(Box::new(AlifeGraphPoint {
      connection_point_name: "x".repeat(70000),
      connection_level_name: String::from("connection-level"),
      location0: 1,
      location1: 2,
      location2: 3,
      location3: 4,
    }));

    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      original
        .write::<XRayByteOrder>(&mut writer)
        .expect_err("expect the inherited data to exceed its size field")
        .to_string(),
      "Invalid error: spawn packet state exceeds the u16 format limit"
    );
  }

  #[test]
  fn rejects_spawn_data_past_its_size_field() {
    let mut original: AlifeObject = new_graph_point_object();

    original.header.name = "x".repeat(70000);

    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      original
        .write::<XRayByteOrder>(&mut writer)
        .expect_err("expect the spawn data to exceed its size field")
        .to_string(),
      "Invalid error: alife object spawn data exceeds the u16 format limit"
    );
  }

  /// The actor starts where its update packet says, so a move that reached only the spawn packet would not move it.
  #[test]
  fn moves_an_actor_in_both_packets() -> XrfResult {
    let mut original: AlifeObject = new_graph_point_object();

    original.header.section = String::from("actor");
    original.clsid = ClsId::SActor;
    original.inherited = AlifeObjectInherited::SeActor(Box::new(AlifeActor::new_mock(Vector3d::new(1.0, 2.0, 3.0))));
    original.move_to(Vector3d::new(-40.0, 5.5, 120.0), Vector3d::new(-0.1, 1.25, 0.0));

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_vec(writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?)?.read_child_by_index(0)?;
    let read: AlifeObject = AlifeObject::read::<XRayByteOrder, _>(&mut reader)?;
    let AlifeObjectInherited::SeActor(actor) = &read.inherited else {
      panic!("Expected an actor payload");
    };

    assert_eq!(read.header.position, Vector3d::new(-40.0, 5.5, 120.0));
    assert_eq!(read.header.direction, Vector3d::new(-0.1, 1.25, 0.0));
    assert_eq!(actor.base.base.update_position, Vector3d::new(-40.0, 5.5, 120.0));
    assert_eq!(actor.base.base.update_torso_yaw, 1.25);
    assert_eq!(actor.base.base.update_torso_pitch, -0.1);

    Ok(())
  }

  /// The spawn packet alone reads back with the update fields a fresh server entity holds: a creature stands where its
  /// spawn packet places it, turned by its angles, with its own health and team, and the rest of the spawn packet whole.
  #[test]
  fn reads_a_spawn_packet_alone_as_a_fresh_entity() -> XrfResult {
    let mut original: AlifeObject = new_graph_point_object();

    original.header.section = String::from("stalker");
    original.clsid = ClsId::AiStlS;
    original.inherited =
      AlifeObjectInherited::SeStalker(Box::new(AlifeStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0))));
    original.header.position = Vector3d::new(-40.0, 5.5, 120.0);
    original.header.direction = Vector3d::new(-0.1, 1.25, 0.0);

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write_spawn_packet::<XRayByteOrder, _>(&mut writer)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: AlifeObject = AlifeObject::read_spawn_packet::<XRayByteOrder, _>(&mut reader)?;
    let AlifeObjectInherited::SeStalker(stalker) = &read.inherited else {
      panic!("Expected a stalker payload");
    };
    let creature: &AlifeObjectCreature = &stalker.base.base.base.base;

    assert!(reader.is_ended());
    assert_eq!(read.header, original.header);
    assert_eq!(original.get_fresh_entity::<XRayByteOrder>()?, read);
    assert_eq!(creature.update_position, Vector3d::new(-40.0, 5.5, 120.0));
    assert_eq!((creature.update_torso_yaw, creature.update_torso_pitch), (1.25, -0.1));
    assert_eq!(creature.update_health, creature.health);
    assert_eq!(creature.update_team, creature.team);

    let mut rewritten: ChunkWriter = ChunkWriter::new();

    read.write_spawn_packet::<XRayByteOrder, _>(&mut rewritten)?;

    assert_eq!(rewritten.buffer, writer.buffer);

    Ok(())
  }

  /// A stalker and a monster written under their sections read back whole, each moved in both packets.
  #[test]
  fn writes_and_moves_a_stalker_and_a_monster() -> XrfResult {
    for (section, clsid, inherited) in [
      (
        "stalker",
        ClsId::AiStlS,
        AlifeObjectInherited::SeStalker(Box::new(AlifeStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0)))),
      ),
      (
        "dog_weak",
        ClsId::SmDogS,
        AlifeObjectInherited::SeMonster(Box::new(AlifeMonster::new_mock(Vector3d::new(1.0, 2.0, 3.0)))),
      ),
    ] {
      let mut original: AlifeObject = new_graph_point_object();

      original.header.section = String::from(section);
      original.clsid = clsid;
      original.inherited = inherited;
      original.move_to(Vector3d::new(-40.0, 5.5, 120.0), Vector3d::new(-0.1, 1.25, 0.0));

      let mut writer: ChunkWriter = ChunkWriter::new();

      original.write::<XRayByteOrder>(&mut writer)?;

      let mut reader: ChunkReader<InMemoryChunkDataSource> =
        ChunkReader::from_vec(writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?)?.read_child_by_index(0)?;
      let read: AlifeObject = AlifeObject::read::<XRayByteOrder, _>(&mut reader)?;
      let creature: &AlifeObjectCreature = match &read.inherited {
        AlifeObjectInherited::SeStalker(stalker) => &stalker.base.base.base.base,
        AlifeObjectInherited::SeMonster(monster) => &monster.base.base.base,
        other => panic!("Expected a stalker or a monster payload, read {other:?}"),
      };

      assert_eq!(read, original);
      assert_eq!(creature.update_position, Vector3d::new(-40.0, 5.5, 120.0));
      assert_eq!(creature.update_torso_yaw, 1.25);
      assert_eq!(creature.update_torso_pitch, -0.1);
    }

    Ok(())
  }

  /// Each section of a class Call of Pripyat creates reads back as that class through the object's two packets.
  #[test]
  fn writes_and_reads_every_class_by_its_section() -> XrfResult {
    let position: Vector3d = Vector3d::new(1.0, 2.0, 3.0);
    let visual: AlifeObjectDynamicVisual = AlifeObjectCreature::new_mock(position).base;
    let item: AlifeObjectItem = AlifeObjectItem::new_mock(position);
    let skeleton: AlifeObjectSkeleton = AlifeObjectSkeleton {
      name: String::from("$editor"),
      flags: 0,
      source_id: u16::MAX,
    };

    for (section, clsid, inherited) in [
      (
        "bolt",
        ClsId::IIBolt,
        AlifeObjectInherited::CseAlifeItemBolt(Box::new(item.clone())),
      ),
      (
        "device_torch",
        ClsId::TorchS,
        AlifeObjectInherited::CseAlifeItemTorch(Box::new(AlifeObjectItemTorch {
          base: item.clone(),
          update_flags: AlifeObjectItemTorch::FLAG_ACTIVE,
        })),
      ),
      (
        "document",
        ClsId::IIDoc,
        AlifeObjectInherited::CseAlifeItemDocument(Box::new(AlifeObjectItemDocument {
          base: item.clone(),
          info_portion: String::from("info"),
        })),
      ),
      (
        "device_flare",
        ClsId::DFlare,
        AlifeObjectInherited::CseAlifeItem(Box::new(item)),
      ),
      (
        "m_crow",
        ClsId::AiCrow,
        AlifeObjectInherited::CseAlifeCreatureCrow(Box::new(AlifeObjectCreature::new_mock(position))),
      ),
      (
        "m_phantom",
        ClsId::AiPhant,
        AlifeObjectInherited::CseAlifeCreaturePhantom(Box::new(AlifeObjectCreature::new_mock(position))),
      ),
      (
        "script_object",
        ClsId::ScrptObj,
        AlifeObjectInherited::CseAlifeDynamicObjectVisual(Box::new(visual.clone())),
      ),
      (
        "search_light",
        ClsId::OSearch,
        AlifeObjectInherited::CseAlifeObjectProjector(Box::new(visual.clone())),
      ),
      (
        "ph_skeleton_object",
        ClsId::PSkelet,
        AlifeObjectInherited::CseAlifePhSkeletonObject(Box::new(AlifeObjectPhSkeletonObject {
          base: visual.clone(),
          skeleton,
        })),
      ),
      (
        "stationary_mgun",
        ClsId::WSTMGun,
        AlifeObjectInherited::CseAlifeStationaryMGun(Box::new(AlifeObjectStationaryMgun {
          base: visual,
          update_working: 0,
          update_enemy_direction: [0.0, 0.0, 1.0].map(RawFloat::from),
        })),
      ),
      (
        "online_offline_group",
        ClsId::OnOffS,
        AlifeObjectInherited::SimSquadScripted(Box::new(AlifeSquad {
          base: AlifeObjectOnlineOfflineGroup::new_mock(vec![]),
          current_target_id: None,
          respawn_point_id: None,
          respawn_point_section: None,
          assigned_terrain_id: None,
        })),
      ),
    ] {
      let mut original: AlifeObject = new_graph_point_object();

      original.header.section = String::from(section);
      original.clsid = clsid;
      original.inherited = inherited;

      let mut writer: ChunkWriter = ChunkWriter::new();

      original.write::<XRayByteOrder>(&mut writer)?;

      let mut reader: ChunkReader<InMemoryChunkDataSource> =
        ChunkReader::from_vec(writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?)?.read_child_by_index(0)?;

      assert_eq!(
        AlifeObject::read::<XRayByteOrder, _>(&mut reader)?,
        original,
        "{section}"
      );
    }

    Ok(())
  }

  /// An item's `UPDATE_Read` moves it to its physics state's position over its spawn packet's.
  #[test]
  fn moves_an_item_and_its_physics_state() {
    let mut original: AlifeObject = new_graph_point_object();

    original.header.section = String::from("medkit");
    original.clsid = ClsId::SFood;
    original.inherited =
      AlifeObjectInherited::CseAlifeItem(Box::new(AlifeObjectItem::new_mock(original.header.position)));
    original.move_to(Vector3d::new(7.0, 8.0, 9.0), Vector3d::new(0.0, 0.5, 0.0));

    let AlifeObjectInherited::CseAlifeItem(item) = &original.inherited else {
      panic!("Expected an item payload");
    };

    assert_eq!(original.header.position, Vector3d::new(7.0, 8.0, 9.0));
    assert_eq!(
      item.update_state.as_ref().map(PhysicsNetState::get_position),
      Some(Vector3d::new(7.0, 8.0, 9.0))
    );
  }

  /// The fields xrAI fills in go back to a fresh entity's, and back again from another packet, nothing else changing.
  #[test]
  fn sets_the_fields_a_root_spawn_build_fills() {
    let mut original: AlifeObject = new_graph_point_object();

    original.header.section = String::from("medkit");
    original.clsid = ClsId::SFood;
    original.inherited =
      AlifeObjectInherited::CseAlifeItem(Box::new(AlifeObjectItem::new_mock(original.header.position)));

    let mut unbuilt: AlifeObject = original.clone();

    unbuilt.set_built_fields(&AlifeBuiltFields::UNBUILT);

    let base: &AlifeObjectAbstract = unbuilt.inherited.get_abstract().expect("an abstract object");

    assert_eq!(unbuilt.header.spawn_id, u16::MAX);
    assert_eq!(
      (base.game_vertex_id, base.distance, base.level_vertex_id),
      (u16::MAX, 0.0, u32::MAX)
    );
    assert_eq!(
      (
        unbuilt.get_built_fields().destination,
        unbuilt.get_built_fields().artefact_position_offset
      ),
      (None, None),
      "an item has no destination or artefact offset"
    );

    unbuilt.set_built_fields(&original.get_built_fields());

    assert_eq!(unbuilt, original, "nothing else changed");
  }

  /// A class storing no position in its update packet moves in its spawn packet alone.
  #[test]
  fn moves_a_graph_point_in_its_spawn_packet() {
    let mut original: AlifeObject = new_graph_point_object();
    let inherited: AlifeObjectInherited = original.inherited.clone();

    original.move_to(Vector3d::new(7.0, 8.0, 9.0), Vector3d::new(0.0, 0.5, 0.0));

    assert_eq!(original.header.position, Vector3d::new(7.0, 8.0, 9.0));
    assert_eq!(original.header.direction, Vector3d::new(0.0, 0.5, 0.0));
    assert_eq!(original.inherited, inherited);
  }

  #[test]
  fn rejects_inherited_size_below_its_own_prefix() -> XrfResult {
    let original: AlifeObject = new_graph_point_object();

    let mut inherited_writer: ChunkWriter = ChunkWriter::new();

    original.inherited.write::<XRayByteOrder, _, _>(
      &mut inherited_writer,
      &mut ChunkWriter::new(),
      SpawnLayout::from(&original.header),
    )?;

    let inherited_length: usize = inherited_writer.bytes_written();

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    let bytes: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?;

    // The inherited payload ends the spawn chunk, so its size prefix is the two bytes before it. Eight bytes of outer
    // chunk header, then the spawn chunk's own header, then its payload.
    let spawn_end: usize = 16 + XRayByteOrder::read_u32(&bytes[12..16]) as usize;
    let prefix_at: usize = spawn_end - inherited_length - 2;

    assert_eq!(
      XRayByteOrder::read_u16(&bytes[prefix_at..prefix_at + 2]),
      inherited_length as u16 + 2,
      "Expect the inherited size prefix at the computed offset"
    );

    for size in [0u16, 1u16] {
      let mut corrupted: Vec<u8> = bytes.clone();

      XRayByteOrder::write_u16(&mut corrupted[prefix_at..prefix_at + 2], size);

      let mut reader: ChunkReader<InMemoryChunkDataSource> =
        ChunkReader::from_bytes(&corrupted)?.read_child_by_index(0)?;

      let error: XrfError = AlifeObject::read::<XRayByteOrder, _>(&mut reader)
        .expect_err("expect the inherited size prefix to be rejected");

      assert_eq!(
        error.to_string(),
        format!(
          "Invalid error: Spawn packet 'test-graph-point' state size {size} is less than its own two-byte prefix"
        )
      );
    }

    Ok(())
  }

  /// `CSE_Abstract::Spawn_Read` reads client data whatever its size, so a spawn carrying some reads and writes back.
  #[test]
  fn carries_client_data_through_a_round_trip() -> XrfResult {
    let mut original: AlifeObject = new_graph_point_object();

    original.header.client_data = vec![4, 8, 15, 16, 23, 42];

    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> =
      ChunkReader::from_vec(writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?)?.read_child_by_index(0)?;

    assert_eq!(AlifeObject::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  /// The engine picks the payload's reader by section, so a payload of another class would be misread.
  #[test]
  fn refuses_to_write_a_payload_its_section_does_not_read() {
    let mut original: AlifeObject = new_graph_point_object();

    original.header.section = String::from("medkit");

    let error: String = original
      .write::<XRayByteOrder>(&mut ChunkWriter::new())
      .expect_err("a graph point payload under an item section")
      .to_string();

    assert!(error.contains("of section 'medkit' (SFood, CseAlifeItem)"), "{error}");
  }

  /// A spawn packet of Clear Sky's layout: a header at spawn `version` and script version 8, then `state` as stored.
  fn new_clear_sky_packet(section: &str, name: &str, version: u16, state: &str) -> XrfResult<Vec<u8>> {
    let mut header: SpawnPacketHeader = new_graph_point_object().header;
    let mut writer: ChunkWriter = ChunkWriter::new();

    header.section = String::from(section);
    header.name = String::from(name);
    header.version = version;
    header.game_type = u16::MAX;
    header.script_version = 8;
    header.write::<XRayByteOrder>(&mut writer)?;
    writer.write_all(&GameWrittenPackets::bytes(state))?;

    Ok(writer.buffer)
  }

  /// The object a spawn packet holds, after checking it writes back byte for byte and survives its LTX text with
  /// `absent_keys` left out.
  fn read_clear_sky_packet(packet: &[u8], absent_keys: &[&str]) -> XrfResult<AlifeObject> {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(packet)?;
    let object: AlifeObject = AlifeObject::read_spawn_packet::<XRayByteOrder, _>(&mut reader)?;
    let mut writer: ChunkWriter = ChunkWriter::new();
    let mut ltx: Ltx = Ltx::new();

    assert!(reader.is_ended());
    object.write_spawn_packet::<XRayByteOrder, _>(&mut writer)?;
    assert_eq!(writer.buffer, packet);
    object.export("object", &mut ltx)?;

    for key in absent_keys {
      assert_eq!(ltx.get_from("object", key), None, "{key}");
    }

    assert_eq!(AlifeObject::import("object", &ltx)?, object);

    Ok(object)
  }

  /// Clear Sky's smart cover stores neither `can_fire` nor `se_smart_cover`'s fields: the engine copies the combat
  /// cover flag into `can_fire`, and the script holds its constructor's empty description and loopholes.
  #[test]
  fn reads_and_writes_a_clear_sky_smart_cover() -> XrfResult {
    assert_eq!(GameWrittenPackets::bytes(ClearSkyStates::SMART_COVER).len(), 77);

    let packet: Vec<u8> = new_clear_sky_packet("smart_cover", "agr_smart_cover", 124, ClearSkyStates::SMART_COVER)?;
    let object: AlifeObject = read_clear_sky_packet(&packet, &["can_fire", "last_description", "loopholes"])?;
    let AlifeObjectInherited::SeSmartCover(cover) = &object.inherited else {
      panic!("Expected a smart cover payload, read {:?}", object.inherited);
    };

    assert_eq!(cover.base.description, "cover_loophole_2");
    assert_eq!(
      (cover.base.enter_min_enemy_distance, cover.base.exit_min_enemy_distance),
      (15.0, 10.0)
    );
    assert_eq!((cover.base.is_combat_cover, cover.base.can_fire), (1, 1));
    assert_eq!((cover.last_description.as_str(), cover.loopholes.len()), ("", 0));

    Ok(())
  }

  /// Clear Sky's inventory box stores no access fields or tip, which hold `CSE_ALifeInventoryBox`'s constructor values.
  #[test]
  fn reads_and_writes_a_clear_sky_inventory_box() -> XrfResult {
    assert_eq!(GameWrittenPackets::bytes(ClearSkyStates::INVENTORY_BOX).len(), 77);

    let packet: Vec<u8> = new_clear_sky_packet(
      "inventory_box",
      "agr_smart_terrain_1_2_box",
      124,
      ClearSkyStates::INVENTORY_BOX,
    )?;
    let object: AlifeObject = read_clear_sky_packet(
      &packet,
      &["inventory_box.can_take", "inventory_box.is_closed", "inventory_box.tip"],
    )?;
    let AlifeObjectInherited::CseAlifeInventoryBox(inventory_box) = &object.inherited else {
      panic!("Expected an inventory box payload, read {:?}", object.inherited);
    };

    assert_eq!(
      inventory_box.base.visual_name,
      "dynamics\\equipment_cache\\equipment_box_02_case"
    );
    assert_eq!(
      (
        inventory_box.can_take,
        inventory_box.is_closed,
        inventory_box.tip.as_str()
      ),
      (1, 0, "inventory_box_use")
    );

    Ok(())
  }

  /// Clear Sky's climbable object stores no material, which holds `CSE_ALifeObjectClimable`'s constructor value.
  #[test]
  fn reads_and_writes_a_clear_sky_climable_object() -> XrfResult {
    assert_eq!(GameWrittenPackets::bytes(ClearSkyStates::CLIMABLE).len(), 79);

    let packet: Vec<u8> = new_clear_sky_packet("climable_object", "clmbl#0", 124, ClearSkyStates::CLIMABLE)?;
    let object: AlifeObject = read_clear_sky_packet(&packet, &["climbable.game_material"])?;
    let AlifeObjectInherited::CseAlifeObjectClimable(climable) = &object.inherited else {
      panic!("Expected a climable object payload, read {:?}", object.inherited);
    };

    assert_eq!(climable.base.shape.len(), 1);
    assert_eq!(climable.game_material, "materials\\fake_ladders");

    Ok(())
  }

  /// Clear Sky's item of spawn version 123 stores no upgrades, which hold `CSE_ALifeInventoryItem`'s constructor's none.
  #[test]
  fn reads_and_writes_a_clear_sky_item_without_upgrades() -> XrfResult {
    assert_eq!(GameWrittenPackets::bytes(ClearSkyStates::ITEM_123).len(), 81);

    let packet: Vec<u8> = new_clear_sky_packet("medkit", "esc_medkit", 123, ClearSkyStates::ITEM_123)?;
    let object: AlifeObject = read_clear_sky_packet(&packet, &["item.upgrades"])?;
    let AlifeObjectInherited::CseAlifeItem(item) = &object.inherited else {
      panic!("Expected an item payload, read {:?}", object.inherited);
    };

    assert_eq!((item.condition, item.upgrades.len()), (1.0, 0));

    Ok(())
  }

  /// Clear Sky's weapon of spawn version 123 stores its loaded grenades but no upgrades, one of 122 neither.
  #[test]
  fn reads_and_writes_clear_sky_weapons() -> XrfResult {
    for (section, name, version, state, size, absent_keys) in [
      (
        "wpn_ak74u",
        "esc_wpn_ak74u",
        123,
        ClearSkyStates::WEAPON_123,
        79,
        ["item.upgrades"].as_slice(),
      ),
      (
        "wpn_rg-6",
        "mp_garbage_mp_wpn_rg-6",
        122,
        ClearSkyStates::WEAPON_122,
        74,
        ["item.upgrades", "item_weapon.elapsed_grenades"].as_slice(),
      ),
    ] {
      assert_eq!(GameWrittenPackets::bytes(state).len(), size, "{name}");

      let packet: Vec<u8> = new_clear_sky_packet(section, name, version, state)?;
      let object: AlifeObject = read_clear_sky_packet(&packet, absent_keys)?;
      let weapon: &AlifeObjectItemWeapon = match &object.inherited {
        AlifeObjectInherited::CseAlifeItemWeaponMagazined(weapon) => &weapon.base,
        AlifeObjectInherited::CseAlifeItemWeaponMagazinedWGl(weapon) => &weapon.base.base,
        AlifeObjectInherited::CseAlifeItemWeaponShotgun(weapon) => &weapon.base.base,
        other => panic!("Expected a weapon payload, read {other:?}"),
      };

      assert_eq!(
        (weapon.ammo_current, weapon.elapsed_grenades, weapon.base.upgrades.len()),
        (90, 0, 0),
        "{name}"
      );
    }

    Ok(())
  }

  /// Each Clear Sky script class a level spawns reads as its Clear Sky type, writes back byte for byte and survives its
  /// text: a level's monster, stalker, faction, smart terrain and actor.
  #[test]
  fn reads_and_writes_clear_sky_script_classes() -> XrfResult {
    for (section, name, state, size, class) in [
      (
        "dog_weak",
        "esc_dog_weak",
        ClearSkyStates::MONSTER,
        96,
        AlifeClass::ClearSkySeMonster,
      ),
      (
        "stalker",
        "esc_flashman_1",
        ClearSkyStates::STALKER,
        171,
        AlifeClass::ClearSkySeStalker,
      ),
      (
        "sim_faction",
        "val_sim_faction_killer",
        ClearSkyStates::SIM_FACTION,
        119,
        AlifeClass::ClearSkySeSimFaction,
      ),
      (
        "smart_terrain",
        "mar_smart_terrain_csky_base",
        ClearSkyStates::SMART_TERRAIN,
        200,
        AlifeClass::ClearSkySeSmartTerrain,
      ),
      (
        "actor",
        "level_prefix_actor",
        ClearSkyStates::ACTOR,
        302,
        AlifeClass::ClearSkySeActor,
      ),
    ] {
      assert_eq!(GameWrittenPackets::bytes(state).len(), size, "{name}");

      let packet: Vec<u8> = new_clear_sky_packet(section, name, 123, state)?;
      let object: AlifeObject = read_clear_sky_packet(&packet, &[])?;

      assert_eq!(object.inherited.get_class(), class, "{name}");
    }

    Ok(())
  }

  /// Clear Sky's faction is `se_sim_faction`, its own class (`SFACTION`), where Call of Pripyat's section is a space
  /// restrictor.
  #[test]
  fn reads_a_clear_sky_faction_as_its_own_class() -> XrfResult {
    let packet: Vec<u8> = new_clear_sky_packet(
      "sim_faction",
      "val_sim_faction_killer",
      123,
      ClearSkyStates::SIM_FACTION,
    )?;
    let object: AlifeObject = read_clear_sky_packet(&packet, &[])?;
    let AlifeObjectInherited::ClearSkySeSimFaction(faction) = &object.inherited else {
      panic!("Expected a Clear Sky faction payload, read {:?}", object.inherited);
    };

    assert_eq!(object.clsid, ClsId::SFaction);
    assert_eq!(
      (
        faction.community_player,
        faction.start_position_filled,
        faction.current_expansion_level
      ),
      (0, 0, 0)
    );
    assert_eq!(faction.base.base.shape.len(), 1);

    Ok(())
  }

  /// A Clear Sky payload under a Call of Pripyat header, or the reverse, would be read by the engine with the other
  /// game's script class, so it is not written.
  #[test]
  fn refuses_a_payload_of_the_other_games_script_class() -> XrfResult {
    let packet: Vec<u8> = new_clear_sky_packet("dog_weak", "esc_dog_weak", 123, ClearSkyStates::MONSTER)?;
    let mut clear_sky: AlifeObject = read_clear_sky_packet(&packet, &[])?;

    clear_sky.header.version = 128;
    clear_sky.header.script_version = 12;

    let error: String = clear_sky
      .write_spawn_packet::<XRayByteOrder, _>(&mut ChunkWriter::new())
      .expect_err("a Clear Sky monster under a Call of Pripyat header")
      .to_string();

    assert!(
      error.contains("(SmDogS, SeMonster) carries SmDogS and a ClearSkySeMonster payload"),
      "{error}"
    );

    let mut call_of_pripyat: AlifeObject = new_graph_point_object();

    call_of_pripyat.header.section = String::from("dog_weak");
    call_of_pripyat.header.version = 123;
    call_of_pripyat.header.script_version = 8;
    call_of_pripyat.clsid = ClsId::SmDogS;
    call_of_pripyat.inherited = AlifeObjectInherited::SeMonster(Box::new(AlifeMonster::new_mock(Vector3d::ZERO)));

    let error: String = call_of_pripyat
      .write_spawn_packet::<XRayByteOrder, _>(&mut ChunkWriter::new())
      .expect_err("a Call of Pripyat monster under a Clear Sky header")
      .to_string();

    assert!(
      error.contains("(SmDogS, ClearSkySeMonster) carries SmDogS and a SeMonster payload"),
      "{error}"
    );

    Ok(())
  }

  /// Clear Sky's scripts read fewer fields below their own script version, a layout no shipped object holds.
  #[test]
  fn refuses_clear_sky_script_fields_of_another_script_version() -> XrfResult {
    let mut packet: Vec<u8> = new_clear_sky_packet("dog_weak", "esc_dog_weak", 123, ClearSkyStates::MONSTER)?;
    let header_size: usize = packet.len() - GameWrittenPackets::bytes(ClearSkyStates::MONSTER).len();

    // The script version is the header's last fixed field before the client data's size and the spawn id.
    packet[header_size - 6] = 7;

    let error: String = AlifeObject::read_spawn_packet::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&packet)?)
      .expect_err("script version 7")
      .to_string();

    assert!(error.contains("se_monster of script version 7 is not read"), "{error}");

    Ok(())
  }

  /// A class whose update packet is empty reads none of it, so bytes there would be dropped by a rewrite.
  #[test]
  fn refuses_update_bytes_its_class_does_not_read() -> XrfResult {
    let original: AlifeObject = new_graph_point_object();
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    let mut bytes: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(0)?;
    let update_at: usize = 16 + XRayByteOrder::read_u32(&bytes[12..16]) as usize;

    // One stray byte past the update packet's action: the outer chunk, the update chunk and its length each grow by one.
    bytes.push(7);

    for at in [4, update_at + 4] {
      let size: u32 = XRayByteOrder::read_u32(&bytes[at..at + 4]);

      XRayByteOrder::write_u32(&mut bytes[at..at + 4], size + 1);
    }

    let length: u16 = XRayByteOrder::read_u16(&bytes[update_at + 8..update_at + 10]);

    XRayByteOrder::write_u16(&mut bytes[update_at + 8..update_at + 10], length + 1);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?.read_child_by_index(0)?;
    let error: String = AlifeObject::read::<XRayByteOrder, _>(&mut reader)
      .expect_err("a stray update byte")
      .to_string();

    assert!(
      error.contains("update data to hold only its class's update fields"),
      "{error}"
    );

    Ok(())
  }
}
