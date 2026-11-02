use crate::vorbis::vorbis_packet_reader::VorbisPacketReader;

/// The identification header packet, held to the checks libvorbis's `_vorbis_unpack_info` makes: a stream failing one
/// is one `ov_open_callbacks` refuses, and the engine with it.
pub(crate) struct VorbisIdentification {
  pub(crate) channels: u16,
  pub(crate) sample_rate: u32,
}

impl VorbisIdentification {
  pub(crate) const MAGIC: &'static [u8] = b"\x01vorbis";

  /// The smallest and largest block size exponents libvorbis decodes, blocks of 64 and 8192 samples.
  const BLOCK_SIZE_EXPONENTS: (u8, u8) = (6, 13);

  pub(crate) fn read(packet: &[u8]) -> Result<Self, String> {
    if !packet.starts_with(Self::MAGIC) {
      return Err(String::from(
        "Ogg stream does not contain a Vorbis identification packet",
      ));
    }

    let mut reader: VorbisPacketReader = VorbisPacketReader::new(&packet[Self::MAGIC.len()..]);
    let version: u32 = reader.read_u32("Vorbis version")?;

    if version != 0 {
      return Err(format!("Unsupported Vorbis version: {version}"));
    }

    let channels: u16 = u16::from(reader.read_u8("channels")?);
    let sample_rate: u32 = reader.read_u32("sample rate")?;

    reader.read_bytes(12, "bitrates")?;

    let block_sizes: u8 = reader.read_u8("block sizes")?;
    let framing: u8 = reader.read_u8("framing bit")?;

    if channels == 0 || sample_rate == 0 {
      return Err(String::from(
        "Vorbis identification packet must define non-zero channels and sample rate",
      ));
    }

    let (short, long): (u8, u8) = (block_sizes & 0x0F, block_sizes >> 4);
    let (smallest, largest): (u8, u8) = Self::BLOCK_SIZE_EXPONENTS;

    if short < smallest || long < short || long > largest {
      return Err(format!(
        "Vorbis block sizes 2^{short} and 2^{long} are not ones libvorbis decodes"
      ));
    }

    if framing & 1 == 0 {
      return Err(String::from("Vorbis identification packet lacks its framing bit"));
    }

    Ok(Self { channels, sample_rate })
  }
}
