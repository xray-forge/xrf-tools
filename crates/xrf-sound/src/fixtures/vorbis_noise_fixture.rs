use ogg::writing::{PacketWriteEndInfo, PacketWriter};

/// An Ogg/Vorbis sound of noise written without an encoder: a setup of one codebook of the values -1 and 1, a flat
/// floor and one residue, then audio packets of random bits, so the frames a decode renders do not repeat.
///
/// The packets mix short and long blocks and the pages hold one to six of them; one packet runs over three pages, and
/// the last page trims the stream by [`Self::TRIMMED_FRAMES`], so seeks meet every layout a decoder has to place.
pub struct VorbisNoiseFixture;

impl VorbisNoiseFixture {
  pub const SAMPLE_RATE: u32 = 48_000;

  /// Frames the last packet renders past the last page's granule position.
  pub const TRIMMED_FRAMES: u64 = 100;

  const SERIAL: u32 = 0x4E4F_4953;

  const SHORT_BLOCK_EXPONENT: u8 = 8;

  const LONG_BLOCK_EXPONENT: u8 = 11;

  /// Residue coefficients each packet codes per channel: four partitions of 32.
  const CODED_COEFFICIENTS: u64 = 128;

  const PARTITION_SIZE: u64 = 32;

  /// The length the packet in the middle is padded to, past two full pages.
  const SPANNING_LENGTH: usize = 150_000;

  /// `-1.0` and `2.0` in the packing of a Vorbis codebook's float.
  const MINIMUM_VALUE: u32 = 0xE010_0000;

  const DELTA_VALUE: u32 = 0x6030_0000;

  /// A sound of `packets_count` audio packets in `channels` channels, the same bytes for the same arguments.
  ///
  /// # Panics
  ///
  /// Panics when `packets_count` is under three, too few to trim.
  pub fn sound(channels: u16, packets_count: usize) -> Vec<u8> {
    assert!(packets_count >= 3, "a noise sound holds three packets or more");

    let mut random: NoiseRandom =
      NoiseRandom(0x9E37_79B9_7F4A_7C15 ^ (u64::from(channels) << 32) ^ packets_count as u64);
    let is_long: Vec<bool> = (0..packets_count).map(|_| !random.next().is_multiple_of(3)).collect();
    let mut writer: PacketWriter<'_, Vec<u8>> = PacketWriter::new(Vec::new());
    let mut granule: u64 = 0;
    let mut page_left: u64 = 0;

    Self::write(
      &mut writer,
      Self::identification(channels),
      PacketWriteEndInfo::EndPage,
      0,
    );
    Self::write(&mut writer, Self::comment(), PacketWriteEndInfo::NormalPacket, 0);
    Self::write(&mut writer, Self::setup(), PacketWriteEndInfo::EndPage, 0);

    for (index, &is_long_block) in is_long.iter().enumerate() {
      if index > 0 {
        granule += (Self::get_block_size(is_long[index - 1]) + Self::get_block_size(is_long_block)) / 4;
      }

      let mut packet: Vec<u8> = Self::audio(
        &mut random,
        channels,
        AudioBlocks {
          is_previous_long: index > 0 && is_long[index - 1],
          is_long: is_long_block,
          is_next_long: is_long.get(index + 1).copied().unwrap_or(false),
        },
      );

      if index == packets_count / 2 {
        packet.resize_with(Self::SPANNING_LENGTH, || random.next() as u8);
      }

      if page_left == 0 {
        page_left = 1 + random.next() % 6;
      }

      page_left -= 1;

      let (end, packet_granule) = if index + 1 == packets_count {
        (PacketWriteEndInfo::EndStream, granule - Self::TRIMMED_FRAMES)
      } else if page_left == 0 {
        (PacketWriteEndInfo::EndPage, granule)
      } else {
        (PacketWriteEndInfo::NormalPacket, granule)
      };

      Self::write(&mut writer, packet, end, packet_granule);
    }

    writer.into_inner()
  }

  fn get_block_size(is_long: bool) -> u64 {
    1 << if is_long {
      Self::LONG_BLOCK_EXPONENT
    } else {
      Self::SHORT_BLOCK_EXPONENT
    }
  }

  fn identification(channels: u16) -> Vec<u8> {
    let mut packet: Vec<u8> = b"\x01vorbis".to_vec();

    packet.extend_from_slice(&0u32.to_le_bytes());
    packet.push(u8::try_from(channels).expect("a noise sound has few channels"));
    packet.extend_from_slice(&Self::SAMPLE_RATE.to_le_bytes());
    packet.extend_from_slice(&[0; 12]);
    packet.push((Self::LONG_BLOCK_EXPONENT << 4) | Self::SHORT_BLOCK_EXPONENT);
    packet.push(1);
    packet
  }

  fn comment() -> Vec<u8> {
    let vendor: &[u8] = b"xrf noise";
    let mut packet: Vec<u8> = b"\x03vorbis".to_vec();

    packet.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    packet.extend_from_slice(vendor);
    packet.extend_from_slice(&0u32.to_le_bytes());
    packet.push(1);
    packet
  }

  /// Codebooks of two one-bit entries valued -1 and 1, one flat floor 1, one residue 1 coding the first
  /// [`Self::CODED_COEFFICIENTS`] with them, one mapping, and a short and a long mode.
  fn setup() -> Vec<u8> {
    let mut bits: NoiseBits = NoiseBits::default();

    for &byte in b"\x05vorbis" {
      bits.write(u64::from(byte), 8);
    }

    // Two such codebooks, since symphonia takes no residue values from the first: each its sync, one dimension, two
    // entries of one bit each, and a lookup of the values -1 and 1.
    bits.write(1, 8);

    for _ in 0..2 {
      bits.write(0x56_4342, 24);
      bits.write(1, 16);
      bits.write(2, 24);
      bits.write(0, 1 + 1 + 5 + 5);
      bits.write(1, 4);
      bits.write(u64::from(Self::MINIMUM_VALUE), 32);
      bits.write(u64::from(Self::DELTA_VALUE), 32);
      bits.write(0, 4 + 1);
      bits.write(0b10, 2);
    }

    // One time domain transform, then floor 1 with no partitions over a range of 256.
    bits.write(0, 6 + 16);
    bits.write(0, 6);
    bits.write(1, 16);
    bits.write(0, 5 + 2);
    bits.write(8, 4);

    // Residue 1 over the coded coefficients, classified by the first codebook into one class the second decodes in the
    // first pass.
    bits.write(0, 6);
    bits.write(1, 16);
    bits.write(0, 24);
    bits.write(Self::CODED_COEFFICIENTS, 24);
    bits.write(Self::PARTITION_SIZE - 1, 24);
    bits.write(0, 6 + 8);
    bits.write(1, 3 + 1);
    bits.write(1, 8);

    // One mapping of one submap to the floor and the residue, no coupling.
    bits.write(0, 6 + 16 + 1 + 1 + 2 + 8 + 8 + 8);

    // A short and a long mode, both of that mapping, then the framing bit.
    bits.write(1, 6);
    bits.write(0, 1 + 16 + 16 + 8);
    bits.write(1, 1);
    bits.write(0, 16 + 16 + 8);
    bits.write(1, 1);

    bits.bytes
  }

  /// A packet of `blocks`: each channel's floor at a random level, then random bits for every residue codeword.
  fn audio(random: &mut NoiseRandom, channels: u16, blocks: AudioBlocks) -> Vec<u8> {
    let mut bits: NoiseBits = NoiseBits::default();
    let codewords: u64 =
      u64::from(channels) * Self::CODED_COEFFICIENTS / Self::PARTITION_SIZE * (1 + Self::PARTITION_SIZE);

    bits.write(0, 1);
    bits.write(u64::from(blocks.is_long), 1);

    if blocks.is_long {
      bits.write(u64::from(blocks.is_previous_long), 1);
      bits.write(u64::from(blocks.is_next_long), 1);
    }

    for _ in 0..channels {
      bits.write(1, 1);
      bits.write(200 + random.next() % 56, 8);
      bits.write(200 + random.next() % 56, 8);
    }

    for _ in 0..codewords {
      bits.write(random.next() & 1, 1);
    }

    bits.bytes
  }

  fn write(writer: &mut PacketWriter<'_, Vec<u8>>, packet: Vec<u8>, end: PacketWriteEndInfo, granule: u64) {
    writer
      .write_packet(packet, Self::SERIAL, end, granule)
      .expect("a noise sound is written to memory");
  }
}

/// The block sizes around one audio packet, which its window flags name.
struct AudioBlocks {
  is_previous_long: bool,
  is_long: bool,
  is_next_long: bool,
}

/// A Vorbis bitstream being packed, least significant bit first.
#[derive(Default)]
struct NoiseBits {
  bytes: Vec<u8>,
  bits_count: usize,
}

impl NoiseBits {
  fn write(&mut self, value: u64, count: u32) {
    for bit in 0..count {
      if self.bits_count.is_multiple_of(8) {
        self.bytes.push(0);
      }

      if (value >> bit) & 1 != 0 {
        *self.bytes.last_mut().expect("a byte was pushed for this bit") |= 1 << (self.bits_count % 8);
      }

      self.bits_count += 1;
    }
  }
}

/// A xorshift sequence, so a fixture is the same on every run.
struct NoiseRandom(u64);

impl NoiseRandom {
  fn next(&mut self) -> u64 {
    self.0 ^= self.0 << 13;
    self.0 ^= self.0 >> 7;
    self.0 ^= self.0 << 17;
    self.0
  }
}
