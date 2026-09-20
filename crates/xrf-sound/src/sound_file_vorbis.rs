use std::io::{Read, Seek};

use ogg::Packet as OggPacket;
use ogg::reading::PacketReader;
use symphonia::core::codecs::audio::well_known::CODEC_ID_VORBIS;
use symphonia::core::codecs::audio::{AudioCodecParameters, AudioDecoderOptions};
use symphonia::core::packet::Packet;
use symphonia::core::units::{Duration, Timestamp};
use symphonia::default::get_codecs;

pub struct VorbisHeaders {
  pub comment: Vec<u8>,
  pub identification: Vec<u8>,
  pub setup: Vec<u8>,
  pub stream_serial: u32,
}

pub fn read_vorbis_headers<R>(reader: &mut PacketReader<R>) -> Result<VorbisHeaders, String>
where
  R: Read + Seek,
{
  let identification: OggPacket = read_expected_packet(reader, "identification")?;
  let stream_serial: u32 = identification.stream_serial();
  let comment: OggPacket = read_expected_packet(reader, "comment")?;
  let setup: OggPacket = read_expected_packet(reader, "setup")?;

  if !identification.data.starts_with(b"\x01vorbis") {
    return Err(String::from(
      "Ogg stream does not contain a Vorbis identification packet",
    ));
  }

  if comment.stream_serial() != stream_serial || setup.stream_serial() != stream_serial {
    return Err(String::from("Vorbis header packets must belong to the same Ogg stream"));
  }

  if !comment.data.starts_with(b"\x03vorbis") {
    return Err(String::from("Ogg stream does not contain a Vorbis comment packet"));
  }

  if !setup.data.starts_with(b"\x05vorbis") {
    return Err(String::from("Ogg stream does not contain a Vorbis setup packet"));
  }

  Ok(VorbisHeaders {
    comment: comment.data,
    identification: identification.data,
    setup: setup.data,
    stream_serial,
  })
}

pub fn parse_identification_packet(packet: &[u8]) -> Result<(u16, u32), String> {
  if !packet.starts_with(b"\x01vorbis") {
    return Err(String::from(
      "Ogg stream does not contain a Vorbis identification packet",
    ));
  }

  let mut offset: usize = 7;
  let version: u32 = read_u32(packet, &mut offset, "Vorbis version")?;

  if version != 0 {
    return Err(format!("Unsupported Vorbis version: {version}"));
  }

  let channels: u16 = read_u8(packet, &mut offset, "channels")? as u16;
  let sample_rate: u32 = read_u32(packet, &mut offset, "sample rate")?;

  if channels == 0 || sample_rate == 0 {
    return Err(String::from(
      "Vorbis identification packet must define non-zero channels and sample rate",
    ));
  }

  Ok((channels, sample_rate))
}

pub fn decode_vorbis_stream<R>(reader: &mut PacketReader<R>, headers: &VorbisHeaders) -> Result<(), String>
where
  R: Read + Seek,
{
  let mut extra_data: Vec<u8> = Vec::with_capacity(headers.identification.len() + headers.setup.len());
  extra_data.extend_from_slice(&headers.identification);
  extra_data.extend_from_slice(&headers.setup);
  let mut codec_parameters: AudioCodecParameters = AudioCodecParameters::new();
  codec_parameters
    .for_codec(CODEC_ID_VORBIS)
    .with_extra_data(extra_data.into_boxed_slice());
  let mut decoder = get_codecs()
    .make_audio_decoder(&codec_parameters, &AudioDecoderOptions::default())
    .map_err(|error| format!("Could not initialize Vorbis decoder: {error}"))?;

  loop {
    let packet: OggPacket = match reader.read_packet() {
      Ok(Some(packet)) => packet,
      Ok(None) => break,
      Err(error) => return Err(format!("Could not read Ogg/Vorbis packet: {error}")),
    };

    if packet.stream_serial() != headers.stream_serial {
      continue;
    }

    let packet: Packet = Packet::new(0, Timestamp::ZERO, Duration::ZERO, packet.data);

    decoder
      .decode(&packet)
      .map_err(|error| format!("Could not decode Vorbis audio packet: {error}"))?;
  }

  Ok(())
}

fn read_expected_packet<R>(reader: &mut PacketReader<R>, packet_name: &str) -> Result<OggPacket, String>
where
  R: Read + Seek,
{
  reader
    .read_packet_expected()
    .map_err(|error| format!("Could not read Vorbis {packet_name} packet: {error}"))
}

fn read_u8(bytes: &[u8], offset: &mut usize, field: &str) -> Result<u8, String> {
  let value: &[u8] = read_bytes(bytes, offset, 1, field)?;
  Ok(value[0])
}

fn read_u32(bytes: &[u8], offset: &mut usize, field: &str) -> Result<u32, String> {
  let value: &[u8] = read_bytes(bytes, offset, 4, field)?;
  Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn read_bytes<'a>(bytes: &'a [u8], offset: &mut usize, length: usize, field: &str) -> Result<&'a [u8], String> {
  let end: usize = offset
    .checked_add(length)
    .ok_or_else(|| format!("Vorbis {field} length overflows the packet"))?;
  let value: &[u8] = bytes
    .get(*offset..end)
    .ok_or_else(|| format!("Vorbis packet ends before {field}"))?;
  *offset = end;
  Ok(value)
}
