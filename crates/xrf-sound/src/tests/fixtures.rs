use std::io::Cursor;

use ogg::reading::PacketReader;
use ogg::writing::{PacketWriteEndInfo, PacketWriter};

/// The stream serial every fixture sound carries.
pub(crate) const SERIAL: u32 = 0x5852_4600;

/// Samples a channel of a fixture sound plays.
pub(crate) const SAMPLES_COUNT: u64 = 3000;

/// An identification packet of a stereo 44.1 kHz stream, with the block sizes `block_sizes` packs.
pub(crate) fn identification(block_sizes: u8) -> Vec<u8> {
  let mut packet: Vec<u8> = b"\x01vorbis".to_vec();

  packet.extend_from_slice(&0u32.to_le_bytes());
  packet.push(2);
  packet.extend_from_slice(&44_100u32.to_le_bytes());
  packet.extend_from_slice(&[0; 12]);
  packet.push(block_sizes);
  packet.push(1);
  packet
}

/// A comment packet holding `comments` behind a vendor string, closed by `framing`.
pub(crate) fn comment(comments: &[Vec<u8>], framing: u8) -> Vec<u8> {
  let vendor: &[u8] = b"Xiph.Org libVorbis I 20020717";
  let mut packet: Vec<u8> = b"\x03vorbis".to_vec();

  packet.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
  packet.extend_from_slice(vendor);
  packet.extend_from_slice(&(comments.len() as u32).to_le_bytes());

  for comment in comments {
    packet.extend_from_slice(&(comment.len() as u32).to_le_bytes());
    packet.extend_from_slice(comment);
  }

  packet.push(framing);
  packet
}

/// A version 3 X-Ray parameter block.
pub(crate) fn xray_comment(
  min_distance: f32,
  max_distance: f32,
  base_volume: f32,
  game_type: u32,
  max_ai: f32,
) -> Vec<u8> {
  [
    3u32.to_le_bytes(),
    min_distance.to_le_bytes(),
    max_distance.to_le_bytes(),
    base_volume.to_le_bytes(),
    game_type.to_le_bytes(),
    max_ai.to_le_bytes(),
  ]
  .concat()
}

/// A sound of the three headers given and three audio packets on two pages, the last closing the stream at
/// [`SAMPLES_COUNT`]. The audio packets are not Vorbis audio, so only a header read accepts the sound.
pub(crate) fn sound_of(identification: Vec<u8>, comment: Vec<u8>) -> Vec<u8> {
  let mut writer: PacketWriter<'_, Vec<u8>> = PacketWriter::new(Vec::new());
  let packets: [(Vec<u8>, PacketWriteEndInfo, u64); 6] = [
    (identification, PacketWriteEndInfo::EndPage, 0),
    (comment, PacketWriteEndInfo::NormalPacket, 0),
    (b"\x05vorbis setup".to_vec(), PacketWriteEndInfo::EndPage, 0),
    (vec![1; 300], PacketWriteEndInfo::EndPage, 1024),
    (vec![2; 40], PacketWriteEndInfo::NormalPacket, SAMPLES_COUNT),
    (vec![3; 600], PacketWriteEndInfo::EndStream, SAMPLES_COUNT),
  ];

  for (data, end, granule) in packets {
    writer
      .write_packet(data, SERIAL, end, granule)
      .expect("a fixture sound is written to memory");
  }

  writer.into_inner()
}

/// A sound whose comment header holds `comments`.
pub(crate) fn sound(comments: &[Vec<u8>]) -> Vec<u8> {
  sound_of(identification(0xB8), comment(comments, 1))
}

/// Every packet of a sound after its headers, with the granule position of the page it ends on.
pub(crate) fn audio_packets(bytes: &[u8]) -> Vec<(Vec<u8>, u64)> {
  let mut reader: PacketReader<Cursor<&[u8]>> = PacketReader::new(Cursor::new(bytes));
  let mut packets: Vec<(Vec<u8>, u64)> = Vec::new();

  while let Some(packet) = reader.read_packet().expect("a fixture sound reads back") {
    let granule: u64 = packet.absgp_page();

    packets.push((packet.data, granule));
  }

  packets.split_off(3)
}
