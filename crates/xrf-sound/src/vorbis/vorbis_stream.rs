use std::io::{Read, Seek, SeekFrom};

use ogg::Packet as OggPacket;
use ogg::reading::PacketReader;
use ogg::writing::{PacketWriteEndInfo, PacketWriter};
use symphonia::core::codecs::audio::well_known::CODEC_ID_VORBIS;
use symphonia::core::codecs::audio::{AudioCodecParameters, AudioDecoder, AudioDecoderOptions};
use symphonia::core::packet::Packet;
use symphonia::core::units::{Duration, Timestamp};
use symphonia::default::get_codecs;

use crate::vorbis::ogg_page_header::OggPageHeader;
use crate::vorbis::vorbis_comment::VorbisComment;
use crate::vorbis::vorbis_identification::VorbisIdentification;

/// The one logical Vorbis stream a sound carries, opened as `ov_open_callbacks` opens it: three header packets of one
/// Ogg stream, then its audio.
pub(crate) struct VorbisStream {
  pub(crate) identification: VorbisIdentification,
  pub(crate) comment: VorbisComment,
  identification_packet: Vec<u8>,
  setup_packet: Vec<u8>,
  serial: u32,
  /// Whether the setup packet closes the stream, which leaves it without audio.
  is_ended_by_setup: bool,
}

impl VorbisStream {
  const SETUP_MAGIC: &'static [u8] = b"\x05vorbis";

  /// The largest Ogg page: its 27-byte header, 255 lacing values and 255 segments of 255 bytes.
  const MAX_PAGE_SIZE: u64 = 27 + 255 + 255 * 255;

  /// Reads the three header packets, leaving `reader` at the first audio packet.
  pub(crate) fn read<R: Read + Seek>(reader: &mut PacketReader<R>) -> Result<Self, String> {
    let identification_packet: OggPacket = Self::read_header_packet(reader, "identification")?;
    let identification: VorbisIdentification = VorbisIdentification::read(&identification_packet.data)?;
    let serial: u32 = identification_packet.stream_serial();
    let comment: OggPacket = Self::read_header_packet(reader, "comment")?;
    let setup: OggPacket = Self::read_header_packet(reader, "setup")?;

    if comment.stream_serial() != serial || setup.stream_serial() != serial {
      return Err(String::from("Vorbis header packets must belong to the same Ogg stream"));
    }

    if !setup.data.starts_with(Self::SETUP_MAGIC) {
      return Err(String::from("Ogg stream does not contain a Vorbis setup packet"));
    }

    Ok(Self {
      identification,
      comment: VorbisComment::read(&comment.data)?,
      is_ended_by_setup: setup.last_in_stream(),
      identification_packet: identification_packet.data,
      setup_packet: setup.data,
      serial,
    })
  }

  /// Decodes every audio packet after the headers, which is what proves the payload playable.
  pub(crate) fn decode<R: Read + Seek>(&self, reader: &mut PacketReader<R>) -> Result<(), String> {
    let mut decoder: Box<dyn AudioDecoder> = self.make_decoder()?;

    while let Some(packet) = Self::read_audio_packet(reader)? {
      if packet.stream_serial() != self.serial {
        continue;
      }

      decoder
        .decode(&Packet::new(0, Timestamp::ZERO, Duration::ZERO, packet.data))
        .map_err(|error| format!("Could not decode Vorbis audio packet: {error}"))?;
    }

    Ok(())
  }

  /// The stream's audio codec, symphonia's Vorbis decoder set up from the identification and setup packets.
  ///
  /// It trims as libvorbis does: the first packet after it starts or resets renders nothing, so each later packet
  /// renders from where the one before it ended.
  pub(crate) fn make_decoder(&self) -> Result<Box<dyn AudioDecoder>, String> {
    let mut extra_data: Vec<u8> = Vec::with_capacity(self.identification_packet.len() + self.setup_packet.len());

    extra_data.extend_from_slice(&self.identification_packet);
    extra_data.extend_from_slice(&self.setup_packet);

    let mut codec_parameters: AudioCodecParameters = AudioCodecParameters::new();

    codec_parameters
      .for_codec(CODEC_ID_VORBIS)
      .with_extra_data(extra_data.into_boxed_slice());

    get_codecs()
      .make_audio_decoder(&codec_parameters, &AudioDecoderOptions::default().gapless(true))
      .map_err(|error| format!("Could not initialize Vorbis decoder: {error}"))
  }

  pub(crate) fn get_serial(&self) -> u32 {
    self.serial
  }

  /// Samples a channel plays, `ov_pcm_total`: the granule position of the stream's last page that places one, found
  /// from the end of `source` as libvorbisfile finds it, so the audio between is not read.
  ///
  /// The count runs from granule zero; the rare stream cut to start at a later granule counts the samples it cut too.
  pub(crate) fn read_samples_count<R: Read + Seek>(&self, source: &mut R) -> Result<u64, String> {
    let length: u64 = source
      .seek(SeekFrom::End(0))
      .map_err(|error| format!("Could not seek to the end of the Ogg stream: {error}"))?;
    let mut tail: Vec<u8> = Vec::new();

    source
      .seek(SeekFrom::Start(length.saturating_sub(Self::MAX_PAGE_SIZE)))
      .and_then(|_| source.read_to_end(&mut tail))
      .map_err(|error| format!("Could not read the end of the Ogg stream: {error}"))?;

    self.find_samples_count(&tail)
  }

  /// [`Self::read_samples_count`] of a stream held whole in `bytes`.
  pub(crate) fn find_samples_count(&self, bytes: &[u8]) -> Result<u64, String> {
    let tail: &[u8] = &bytes[bytes.len().saturating_sub(Self::MAX_PAGE_SIZE as usize)..];

    (0..tail.len())
      .rev()
      .find_map(|offset| {
        OggPageHeader::read(tail, offset)
          .filter(|header| header.serial == self.serial)
          .and_then(|header| header.granule)
      })
      .ok_or_else(|| String::from("Ogg stream has no page placing its last sample"))
  }

  /// The stream written again, `reader` standing at its first audio packet as [`Self::read`] leaves it: the comment
  /// header as [`Self::comment`] holds it now and every other packet as it was, the identification packet on a page of
  /// its own, the comment and setup closing the next, and each audio packet ending the page and carrying the granule
  /// position it did.
  ///
  /// # Errors
  ///
  /// Returns an error when a packet of another Ogg stream follows the headers, since the rewrite carries one stream.
  pub(crate) fn rewrite<R: Read + Seek>(&self, reader: &mut PacketReader<R>) -> Result<Vec<u8>, String> {
    let mut writer: PacketWriter<'_, Vec<u8>> = PacketWriter::new(Vec::new());
    let setup_end: PacketWriteEndInfo = if self.is_ended_by_setup {
      PacketWriteEndInfo::EndStream
    } else {
      PacketWriteEndInfo::EndPage
    };

    Self::write_packet(
      &mut writer,
      self.identification_packet.clone(),
      self.serial,
      PacketWriteEndInfo::EndPage,
      0,
    )?;
    Self::write_packet(
      &mut writer,
      self.comment.write()?,
      self.serial,
      PacketWriteEndInfo::NormalPacket,
      0,
    )?;
    Self::write_packet(&mut writer, self.setup_packet.clone(), self.serial, setup_end, 0)?;

    while let Some(packet) = Self::read_audio_packet(reader)? {
      if packet.stream_serial() != self.serial {
        return Err(String::from(
          "Ogg file carries a second stream beside the sound's, which a rewrite would drop",
        ));
      }

      let end: PacketWriteEndInfo = if packet.last_in_stream() {
        PacketWriteEndInfo::EndStream
      } else if packet.last_in_page() {
        PacketWriteEndInfo::EndPage
      } else {
        PacketWriteEndInfo::NormalPacket
      };
      let granule: u64 = packet.absgp_page();

      Self::write_packet(&mut writer, packet.data, self.serial, end, granule)?;
    }

    Ok(writer.into_inner())
  }

  fn read_header_packet<R: Read + Seek>(reader: &mut PacketReader<R>, name: &str) -> Result<OggPacket, String> {
    reader
      .read_packet_expected()
      .map_err(|error| format!("Could not read Vorbis {name} packet: {error}"))
  }

  fn read_audio_packet<R: Read + Seek>(reader: &mut PacketReader<R>) -> Result<Option<OggPacket>, String> {
    reader
      .read_packet()
      .map_err(|error| format!("Could not read Ogg/Vorbis packet: {error}"))
  }

  fn write_packet(
    writer: &mut PacketWriter<'_, Vec<u8>>,
    data: Vec<u8>,
    serial: u32,
    end: PacketWriteEndInfo,
    granule: u64,
  ) -> Result<(), String> {
    writer
      .write_packet(data, serial, end, granule)
      .map_err(|error| format!("Could not write Ogg/Vorbis packet: {error}"))
  }
}
