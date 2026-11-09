use byteorder::ByteOrder;
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;

use crate::data::alife::spawn_packet_header::SpawnPacketHeader;

/// One entity's spawn packet, `CSE_Abstract::Spawn_Write`: its header, then its class's own fields, `state`, after the
/// size the header patches in. `level.spawn` holds one per entity as bytes, and the Level Editor saves one as
/// `[object_N_spawndata]` fields.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnPacket<S> {
  pub header: SpawnPacketHeader,
  /// The class's own fields, `STATE_Write`.
  pub state: S,
}

impl<S: ChunkFieldReadWrite> ChunkFieldReadWrite for SpawnPacket<S> {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    let header: SpawnPacketHeader = reader.read_xr::<T, _>()?;
    let state: S = header.read_state::<T, _, _>(reader, |reader| reader.read_xr::<T, _>())?;

    Ok(Self { header, state })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_xr::<T, _>(&self.header)?;
    SpawnPacketHeader::write_state::<T, _>(writer, |writer| writer.write_xr::<T, _>(&self.state))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::VecDeque;

  use xrf_chunk::{
    ChunkField, ChunkFieldKind, ChunkFieldReadWrite, ChunkFieldRecorder, ChunkFieldReplay, ChunkFieldSource,
    ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder,
  };
  use xrf_error::{XrfError, XrfResult};
  use xrf_math::{Affine3d, Sphere, Vector3d};

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_space_restrictor::AlifeObjectSpaceRestrictor;
  use crate::data::alife::spawn_packet::SpawnPacket;
  use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
  use crate::data::generic::shape::Shape;

  type Restrictor = SpawnPacket<AlifeObjectSpaceRestrictor>;

  /// Fields handed out in order, as an entity's saved spawn data would be.
  struct QueuedFields(VecDeque<ChunkField>);

  impl ChunkFieldSource for QueuedFields {
    fn read_field(&mut self, _: ChunkFieldKind) -> XrfResult<ChunkField> {
      self
        .0
        .pop_front()
        .ok_or_else(|| XrfError::new_parsing_error("No field left"))
    }

    fn is_ended(&self) -> bool {
      self.0.is_empty()
    }
  }

  /// A restrictor as a level designer places one: the ids the editor leaves unset, two lines of logic.
  fn new_restrictor(shape: Vec<Shape>) -> Restrictor {
    SpawnPacket {
      header: SpawnPacketHeader {
        id: u16::MAX,
        net_action: 1,
        section: String::from("space_restrictor"),
        name: String::from("test_restrictor"),
        script_game_id: 0,
        script_rp: 254,
        position: Vector3d::new(-55.5, -22.25, 93.0),
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
        shape,
        restrictor_type: 3,
      },
    }
  }

  fn new_unit_sphere() -> Vec<Shape> {
    vec![Shape::Sphere(Sphere {
      center: Vector3d::ZERO,
      radius: 1.0,
    })]
  }

  fn record(packet: &Restrictor) -> XrfResult<Vec<ChunkField>> {
    let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();

    packet.write_fields::<XRayByteOrder, _>(&mut recorder)?;

    Ok(recorder.into_fields())
  }

  fn write_bytes(packet: &Restrictor) -> XrfResult<Vec<u8>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    packet.write::<XRayByteOrder>(&mut writer)?;

    Ok(writer.buffer)
  }

  /// The fields in the order and kinds GSC's `labx8` scene saves a space restrictor's (`[object_50_spawndata]`): the
  /// size `Spawn_Write` patches in kept as the zero first written, the sphere as a vector and a float.
  #[test]
  fn records_a_restrictor_as_the_editor_numbers_its_fields() -> XrfResult {
    assert_eq!(
      record(&new_restrictor(new_unit_sphere()))?,
      vec![
        ChunkField::U16(1),
        ChunkField::String(String::from("space_restrictor")),
        ChunkField::String(String::from("test_restrictor")),
        ChunkField::U8(0),
        ChunkField::U8(254),
        ChunkField::Vector3([-55.5, -22.25, 93.0]),
        ChunkField::Vector3([0.0, -0.0, 0.0]),
        ChunkField::U16(0),
        ChunkField::U16(u16::MAX),
        ChunkField::U16(u16::MAX),
        ChunkField::U16(u16::MAX),
        ChunkField::U16(33),
        ChunkField::U16(128),
        ChunkField::U16(u16::MAX),
        ChunkField::U16(12),
        ChunkField::U16(0),
        ChunkField::U16(u16::MAX),
        ChunkField::U16(0),
        ChunkField::U16(u16::MAX),
        ChunkField::F32(0.0),
        ChunkField::U32(1),
        ChunkField::U32(u32::MAX),
        ChunkField::U32(0xFFFF_FF3E),
        ChunkField::String(String::from("[logic]\r\ncfg = scripts\\test_restrictor.ltx")),
        ChunkField::U32(u32::MAX),
        ChunkField::U32(u32::MAX),
        ChunkField::U8(1),
        ChunkField::U8(Shape::SPHERE_TYPE),
        ChunkField::Vector3([0.0, 0.0, 0.0]),
        ChunkField::F32(1.0),
        ChunkField::U8(3),
      ]
    );

    Ok(())
  }

  /// Bytes and fields read back into the same packet, the bytes with the state's size patched in.
  #[test]
  fn reads_bytes_and_fields_back_into_the_same_packet() -> XrfResult {
    for shape in [
      new_unit_sphere(),
      vec![
        Shape::Box(Affine3d {
          x_axis: Vector3d::new(20.0, 0.0, 0.0),
          y_axis: Vector3d::new(0.0, 16.5, 0.0),
          z_axis: Vector3d::new(0.0, 0.0, 22.5),
          translation: Vector3d::new(4.75, 0.0, -11.5),
        }),
        Shape::Sphere(Sphere {
          center: Vector3d::new(1.0, 2.0, 3.0),
          radius: 4.5,
        }),
      ],
    ] {
      let packet: Restrictor = new_restrictor(shape);
      let bytes: Vec<u8> = write_bytes(&packet)?;
      let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?;
      let mut source: QueuedFields = QueuedFields(record(&packet)?.into());

      assert_eq!(Restrictor::read::<XRayByteOrder, _>(&mut reader)?, packet);
      assert!(reader.is_ended());
      assert_eq!(
        Restrictor::read_fields::<XRayByteOrder, _>(&mut ChunkFieldReplay::new(&mut source))?,
        packet
      );
      assert!(source.0.is_empty());
    }

    Ok(())
  }

  /// A size that counts other than the state's fields is refused.
  #[test]
  fn refuses_bytes_whose_state_size_counts_other_fields() -> XrfResult {
    let packet: Restrictor = new_restrictor(new_unit_sphere());
    let mut bytes: Vec<u8> = write_bytes(&packet)?;
    let mut state: ChunkWriter = ChunkWriter::new();

    packet.state.write::<XRayByteOrder>(&mut state)?;

    let size_at: usize = bytes.len() - state.bytes_written() - 2;
    let short: u16 = u16::try_from(state.bytes_written()).expect("a small state") + 1;

    bytes[size_at..size_at + 2].copy_from_slice(&short.to_le_bytes());

    let error: String = Restrictor::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&bytes)?)
      .expect_err("a state size one short of its fields")
      .to_string();

    assert!(
      error.contains(&format!("counts {short} bytes with its size")),
      "{error}"
    );

    Ok(())
  }

  /// Client data is raw bytes, which fields cannot hold; the bytes carry it.
  #[test]
  fn refuses_client_data_in_fields() -> XrfResult {
    let mut packet: Restrictor = new_restrictor(new_unit_sphere());

    packet.header.client_data = vec![4, 8, 15];

    let error: String = record(&packet).expect_err("client data").to_string();

    assert!(error.contains("Cannot record 3 raw bytes"), "{error}");
    assert_eq!(
      Restrictor::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&write_bytes(&packet)?)?)?,
      packet
    );

    Ok(())
  }
}
