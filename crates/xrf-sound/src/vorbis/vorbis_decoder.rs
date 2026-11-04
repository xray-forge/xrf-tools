use std::io::{Cursor, SeekFrom};

use ogg::Packet as OggPacket;
use ogg::reading::PacketReader;
use symphonia::core::codecs::audio::AudioDecoder;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::packet::Packet;
use symphonia::core::units::{Duration, Timestamp};
use xrf_error::{XrfError, XrfResult};

use crate::vorbis::ogg_page_header::OggPageHeader;
use crate::vorbis::vorbis_stream::VorbisStream;

/// An Ogg/Vorbis sound held in `B` decoded packet by packet, each packet placed at the frame it starts on as libvorbis
/// places it: from the stream's start, and after a seek from the granule position of the page it lands on.
///
/// The headers are checked as [`crate::SoundFile`] checks them; the comments, the engine's binary parameter block among
/// them, are not interpreted.
pub struct VorbisDecoder<B: AsRef<[u8]>> {
  reader: PacketReader<Cursor<B>>,
  decoder: Box<dyn AudioDecoder>,
  serial: u32,
  channels: u16,
  sample_rate: u32,
  samples_count: u64,
  /// Where the first audio page starts, so the decoder seeks back to the stream's start without a page to place it.
  audio_start: usize,
  /// The frame the next decoded packet starts on; unknown after a seek until a page's granule position places it.
  position: Option<u64>,
}

impl<B: AsRef<[u8]>> VorbisDecoder<B> {
  /// Pages before a seek's landing are walked header by header once the bisection has narrowed it to this many bytes.
  const WALKED_BYTES: usize = 64 * 1024;

  /// Whether `bytes` start as an Ogg stream, the container this decoder reads; a sound in any other is not its to open.
  pub fn is_ogg(bytes: &[u8]) -> bool {
    bytes.starts_with(OggPageHeader::CAPTURE)
  }

  /// Opens the sound in `bytes` at its first frame.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes do not hold the Vorbis headers libvorbis opens, a page placing the stream's last
  /// sample, or a setup symphonia's Vorbis codec accepts.
  pub fn open(bytes: B) -> XrfResult<Self> {
    Self::read(bytes).map_err(|error| XrfError::new_verify_error(format!("Failed to open sound: {error}")))
  }

  pub fn get_channels(&self) -> u16 {
    self.channels
  }

  pub fn get_sample_rate(&self) -> u32 {
    self.sample_rate
  }

  /// Samples each channel plays, `ov_pcm_total`: decoding stops there, trimming the last packet to it.
  pub fn get_samples_count(&self) -> u64 {
    self.samples_count
  }

  /// Moves to the last page placing a frame at or before `frame`, so the next packet decoded starts there or earlier
  /// and the caller drops what comes before `frame` by the frame [`Self::decode_next`] answers.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes cannot be read at the page found.
  pub fn seek(&mut self, frame: u64) -> XrfResult<()> {
    let landing: usize = self.find_landing(frame);

    self
      .reader
      .seek_bytes(SeekFrom::Start(landing as u64))
      .map_err(|error| XrfError::new_verify_error(format!("Failed to seek in sound: {error}")))?;
    self.decoder.reset();
    self.position = (landing == self.audio_start).then_some(0);

    Ok(())
  }

  /// Decodes the next packet that renders audio into `samples`, interleaved, replacing what they held, and answers the
  /// frame it starts on; none once the stream has no audio left, leaving `samples` empty.
  ///
  /// A packet the codec refuses as corrupt is skipped, as players do, and the packets after it wait for the next
  /// page's granule position to place them.
  ///
  /// # Errors
  ///
  /// Returns an error when the Ogg stream cannot be read further or the codec fails for good.
  pub fn decode_next(&mut self, samples: &mut Vec<f32>) -> XrfResult<Option<u64>> {
    let channels: usize = usize::from(self.channels);

    loop {
      let Some(packet) = self.read_packet()? else {
        samples.clear();
        return Ok(None);
      };
      // A page's granule position is where the last packet ending on it stops, so it places the packet after it
      // whatever came before; between pages each packet starts where the one before it stopped.
      let page_granule: Option<u64> = packet
        .last_in_page()
        .then(|| packet.absgp_page())
        .filter(|granule| *granule != u64::MAX);
      let first_frame: Option<u64> = self.position;

      match self
        .decoder
        .decode(&Packet::new(0, Timestamp::ZERO, Duration::ZERO, packet.data))
      {
        Ok(buffer) => {
          let frames: u64 = buffer.frames() as u64;

          buffer.copy_to_vec_interleaved(samples);
          self.position = page_granule.or(first_frame.map(|first| first + frames));

          if let Some(first_frame) = first_frame {
            let kept: u64 = frames.min(self.samples_count.saturating_sub(first_frame));

            samples.truncate(usize::try_from(kept).unwrap_or(usize::MAX) * channels);

            if kept > 0 {
              return Ok(Some(first_frame));
            }
          }
        }
        Err(SymphoniaError::DecodeError(_)) => self.position = page_granule,
        Err(error) => return Err(XrfError::new_verify_error(format!("Failed to decode sound: {error}"))),
      }
    }
  }

  fn read(bytes: B) -> Result<Self, String> {
    let mut reader: PacketReader<Cursor<B>> = PacketReader::new(Cursor::new(bytes));
    let stream: VorbisStream = VorbisStream::read(&mut reader)?;
    let audio_start: usize = usize::try_from(reader.get_ref().position()).unwrap_or(usize::MAX);
    let samples_count: u64 = stream.find_samples_count(reader.get_ref().get_ref().as_ref())?;

    Ok(Self {
      decoder: stream.make_decoder()?,
      serial: stream.get_serial(),
      channels: stream.identification.channels,
      sample_rate: stream.identification.sample_rate,
      samples_count,
      audio_start,
      position: Some(0),
      reader,
    })
  }

  /// The next audio packet of the sound's stream, skipping any other stream's.
  fn read_packet(&mut self) -> XrfResult<Option<OggPacket>> {
    loop {
      match self.reader.read_packet() {
        Ok(Some(packet)) if packet.stream_serial() != self.serial => continue,
        Ok(packet) => return Ok(packet),
        Err(error) => return Err(XrfError::new_verify_error(format!("Failed to read sound: {error}"))),
      }
    }
  }

  /// Where to read from to reach `frame`: the start of the last page placing a frame at or before it that hands a
  /// reader starting there a packet, or the first audio page. A page's granule position grows with the stream, so a
  /// bisection over the bytes narrows the search before the pages are walked.
  fn find_landing(&self, frame: u64) -> usize {
    let bytes: &[u8] = self.reader.get_ref().get_ref().as_ref();
    let mut landing: usize = self.audio_start;
    // Every page starting at or after `end` that places a frame places one past `frame`.
    let mut end: usize = bytes.len();

    while end - landing > Self::WALKED_BYTES {
      let middle: usize = landing + (end - landing) / 2;

      match self.find_placing_page(bytes, middle) {
        Some(header) if header.start < end && header.granule.is_some_and(|it| it <= frame) => landing = header.start,
        _ => end = middle,
      }
    }

    let mut offset: usize = landing;

    while let Some(header) = self.find_placing_page(bytes, offset) {
      if header.granule.is_none_or(|it| it > frame) {
        break;
      }

      landing = header.start;
      offset = header.end;
    }

    landing
  }

  /// The first page of the sound's stream at or after `offset` that places a frame and hands a reader starting on it
  /// a packet.
  fn find_placing_page(&self, bytes: &[u8], offset: usize) -> Option<OggPageHeader> {
    let mut start: usize = offset;

    loop {
      let header: OggPageHeader = OggPageHeader::find(bytes, start, self.serial)?;

      if header.granule.is_some() && header.has_whole_packet {
        return Some(header);
      }

      start = header.end;
    }
  }
}
