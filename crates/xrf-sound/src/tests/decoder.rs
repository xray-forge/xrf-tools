use std::io::Cursor;

use ogg::reading::PacketReader;
use ogg::writing::{PacketWriteEndInfo, PacketWriter};
use xrf_error::XrfResult;

use crate::fixtures::VorbisNoiseFixture;
use crate::vorbis::ogg_page_header::OggPageHeader;
use crate::{SoundFile, VorbisDecoder};

fn decode_whole(bytes: &[u8]) -> XrfResult<Vec<f32>> {
  let mut decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(bytes)?;
  let channels: usize = usize::from(decoder.get_channels());
  let mut samples: Vec<f32> = Vec::new();
  let mut packet: Vec<f32> = Vec::new();

  while let Some(first_frame) = decoder.decode_next(&mut packet)? {
    assert_eq!(
      first_frame as usize,
      samples.len() / channels,
      "packets follow one another"
    );
    samples.extend_from_slice(&packet);
  }

  Ok(samples)
}

#[test]
fn decodes_a_sound_to_the_length_its_last_page_places() -> XrfResult {
  for channels in [1, 2] {
    let bytes: Vec<u8> = VorbisNoiseFixture::sound(channels, 400);
    let file: SoundFile = SoundFile::read_strictly_from_bytes(&bytes)?;
    let decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(bytes.as_slice())?;
    let samples: Vec<f32> = decode_whole(&bytes)?;

    assert_eq!(decoder.get_channels(), channels);
    assert_eq!(decoder.get_sample_rate(), VorbisNoiseFixture::SAMPLE_RATE);
    assert_eq!(decoder.get_samples_count(), file.samples_count);
    assert_eq!(samples.len() as u64, file.samples_count * u64::from(channels));
    assert!(samples.iter().filter(|sample| **sample != 0.0).count() > samples.len() / 2);
  }

  Ok(())
}

#[test]
fn seeks_forward_and_back_to_the_frames_a_straight_decode_gives() -> XrfResult {
  let bytes: Vec<u8> = VorbisNoiseFixture::sound(2, 600);
  let whole: Vec<f32> = decode_whole(&bytes)?;
  let mut decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(bytes.as_slice())?;
  let samples_count: u64 = decoder.get_samples_count();
  let mut packet: Vec<f32> = Vec::new();
  let mut frames: Vec<u64> = vec![
    0,
    1,
    127,
    128,
    129,
    samples_count / 2,
    samples_count - 1,
    5000,
    3,
    samples_count - 2,
  ];
  let mut random: u64 = 0x2545_F491_4F6C_DD1D;

  for _ in 0..300 {
    random ^= random << 13;
    random ^= random >> 7;
    random ^= random << 17;
    frames.push(random % samples_count);
  }

  for frame in frames {
    decoder.seek(frame)?;

    let first_frame: u64 = decoder
      .decode_next(&mut packet)?
      .expect("a frame inside the sound decodes");
    let mut decoded: Vec<f32> = packet.clone();

    // A fixture page ends at most six packets of 1024 frames after the one before it.
    assert!(
      first_frame <= frame,
      "a seek to {frame} lands at {first_frame}, past it"
    );
    assert!(
      frame - first_frame < 2 * 6 * 1024,
      "a seek to {frame} lands far before it, at {first_frame}"
    );

    while decoded.len() < 2 * (frame - first_frame + 4096) as usize
      && let Some(next) = decoder.decode_next(&mut packet)?
    {
      assert_eq!(
        next,
        first_frame + decoded.len() as u64 / 2,
        "packets follow one another after a seek"
      );
      decoded.extend_from_slice(&packet);
    }

    let start: usize = 2 * first_frame as usize;

    assert_eq!(
      decoded,
      whole[start..(start + decoded.len()).min(whole.len())],
      "a seek to {frame}"
    );
  }

  Ok(())
}

#[test]
fn a_seek_to_the_end_leaves_nothing_to_decode() -> XrfResult {
  let bytes: Vec<u8> = VorbisNoiseFixture::sound(1, 50);
  let mut decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(bytes.as_slice())?;
  let mut packet: Vec<f32> = vec![1.0];

  for frame in [decoder.get_samples_count(), decoder.get_samples_count() + 10_000] {
    decoder.seek(frame)?;

    while let Some(first_frame) = decoder.decode_next(&mut packet)? {
      assert!(
        first_frame + (packet.len() as u64) <= frame,
        "nothing at or past {frame} decodes"
      );
    }

    assert!(packet.is_empty());
  }

  Ok(())
}

const OTHER_SERIAL: u32 = 0x4F54_4852;

// Pages end where the original's did; `between` becomes another stream's page after each page from the setup on.
fn rewrite(bytes: &[u8], mut change: impl FnMut(usize, Vec<u8>) -> Vec<u8>, between: Option<&[u8]>) -> Vec<u8> {
  let mut reader: PacketReader<Cursor<&[u8]>> = PacketReader::new(Cursor::new(bytes));
  let mut writer: PacketWriter<'_, Vec<u8>> = PacketWriter::new(Vec::new());
  let mut index: usize = 0;

  while let Some(packet) = reader.read_packet().expect("the fixture reads") {
    let end: PacketWriteEndInfo = if packet.last_in_stream() {
      PacketWriteEndInfo::EndStream
    } else if packet.last_in_page() {
      PacketWriteEndInfo::EndPage
    } else {
      PacketWriteEndInfo::NormalPacket
    };
    let (serial, granule): (u32, u64) = (packet.stream_serial(), packet.absgp_page());

    writer
      .write_packet(change(index, packet.data), serial, end, granule)
      .expect("a sound is written to memory");

    if let Some(other) = between
      && index >= 2
      && matches!(end, PacketWriteEndInfo::EndPage)
    {
      writer
        .write_packet(other.to_vec(), OTHER_SERIAL, PacketWriteEndInfo::EndPage, 0)
        .expect("a sound is written to memory");
    }

    index += 1;
  }

  writer.into_inner()
}

// Two full lacing values, claiming 510 bytes of body.
fn claiming_header(serial: u32) -> Vec<u8> {
  let mut header: Vec<u8> = OggPageHeader::CAPTURE.to_vec();

  header.extend_from_slice(&[0, 0]);
  header.extend_from_slice(&0u64.to_le_bytes());
  header.extend_from_slice(&serial.to_le_bytes());
  header.extend_from_slice(&[0; 8]);
  header.extend_from_slice(&[2, 255, 255]);
  header
}

#[test]
fn skips_a_packet_the_codec_refuses_and_places_the_next_page_by_its_granule() -> XrfResult {
  let clean: Vec<u8> = VorbisNoiseFixture::sound(1, 200);
  let whole: Vec<f32> = decode_whole(&clean)?;
  // The first bit set marks no audio packet, which the codec refuses as corrupt.
  let damaged: Vec<u8> = rewrite(&clean, |index, data| if index == 40 { vec![1] } else { data }, None);
  let mut decoder: VorbisDecoder<&[u8]> = VorbisDecoder::open(damaged.as_slice())?;
  let mut packet: Vec<f32> = Vec::new();
  let mut placed: Vec<(u64, Vec<f32>)> = Vec::new();

  while let Some(first_frame) = decoder.decode_next(&mut packet)? {
    placed.push((first_frame, packet.clone()));
  }

  let decoded: usize = placed.iter().map(|(_, samples)| samples.len()).sum();
  let (last_frame, last) = placed.last().expect("the sound decodes past the damage");

  assert!(
    placed
      .windows(2)
      .all(|pair| pair[0].0 + pair[0].1.len() as u64 <= pair[1].0),
    "packets keep their order and never overlap"
  );
  assert!(decoded < whole.len(), "the refused packet's frames are missing");
  assert_eq!(*last_frame + last.len() as u64, decoder.get_samples_count());
  assert_eq!(
    &last[..],
    &whole[*last_frame as usize..],
    "the sound plays on as decoded whole"
  );

  Ok(())
}

#[test]
fn decodes_and_seeks_one_stream_among_others_as_alone() -> XrfResult {
  let clean: Vec<u8> = VorbisNoiseFixture::sound(2, 300);
  // Another stream's pages between the sound's, each holding what reads as a page header claiming the bytes after it.
  let other: Vec<u8> = [&claiming_header(OTHER_SERIAL)[..], &[7; 64]].concat();
  let muxed: Vec<u8> = rewrite(&clean, |_, data| data, Some(&other));
  let mut alone: VorbisDecoder<&[u8]> = VorbisDecoder::open(clean.as_slice())?;
  let mut among: VorbisDecoder<&[u8]> = VorbisDecoder::open(muxed.as_slice())?;
  let (mut expected, mut packet): (Vec<f32>, Vec<f32>) = (Vec::new(), Vec::new());

  assert!(muxed.len() > clean.len() + 10 * other.len());
  assert_eq!(decode_whole(&muxed)?, decode_whole(&clean)?);
  assert_eq!(among.get_samples_count(), alone.get_samples_count());

  for frame in [0, 1000, alone.get_samples_count() / 2, alone.get_samples_count() - 1] {
    alone.seek(frame)?;
    among.seek(frame)?;

    assert_eq!(
      among.decode_next(&mut packet)?,
      alone.decode_next(&mut expected)?,
      "a seek to {frame}"
    );
    assert_eq!(packet, expected, "a seek to {frame}");
  }

  Ok(())
}

#[test]
fn finds_a_page_past_a_capture_pattern_whose_claimed_length_runs_over_it() {
  let page: Vec<u8> = VorbisNoiseFixture::refused(1, 3);
  let fake: Vec<u8> = claiming_header(OTHER_SERIAL);
  let bytes: Vec<u8> = [&fake[..], &page, &[0; 600]].concat();
  let real: OggPageHeader = OggPageHeader::read(&page, 0).expect("the fixture starts with a page");

  assert!(
    OggPageHeader::find(&bytes, 0, OTHER_SERIAL).is_some_and(|it| it.end > fake.len() + real.end),
    "the fake header claims the page after it"
  );
  assert_eq!(
    OggPageHeader::find(&bytes, 0, real.serial).map(|it| it.start),
    Some(fake.len())
  );
}

#[test]
fn refuses_bytes_that_are_no_vorbis_sound() {
  for bytes in [&b"OggS this is no sound at all"[..], &[]] {
    assert!(VorbisDecoder::open(bytes).is_err());
  }
}
