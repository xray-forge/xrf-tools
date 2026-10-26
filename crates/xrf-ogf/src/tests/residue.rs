use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::chunks::skeleton::ogf_kinematics_chunk::OgfKinematicsChunk;
use crate::ogf_file::OgfFile;
use crate::residue::{OgfNormalization, OgfResidueCause};
use crate::survey::OgfChunksProcessor;
use crate::tests::fixtures;

#[test]
fn reads_a_visual_whose_fifth_reference_is_split() -> XrfResult {
  let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(fixtures::split_motion_ref()?)?;
  let kinematics: &OgfKinematicsChunk = file.kinematics.as_ref().expect("Expect a motion refs chunk");

  assert_eq!(
    kinematics.motion_refs,
    fixtures::DECLARED_REFS.map(String::from).to_vec(),
    "Expect the four references the count governs, which is what the engine loads"
  );
  assert_eq!(
    kinematics.trailing,
    b"actors\\st".to_vec(),
    "Expect the part of the fifth path the declared size covers to be captured, not refused"
  );

  let residue = file.residue.expect("Expect the trailing bytes to be accounted for");

  assert_eq!(
    residue.cause,
    OgfResidueCause::SplitMotionRef {
      path: String::from(fixtures::SPLIT_REF)
    },
    "Expect the two fragments to be reported as the one path they form"
  );
  assert_eq!(residue.bytes, b"alker_scenario_animation\0".to_vec());

  Ok(())
}

#[test]
fn reads_motion_refs_through_the_narrow_path() -> XrfResult {
  // The route `ogf patch-motion-refs` takes, which failed on these files even though it reads only chunk 24.
  let mut reader = xrf_chunk::ChunkReader::from_vec(fixtures::split_motion_ref()?)?;

  assert_eq!(
    OgfFile::read_motion_refs_from_chunk::<XRayByteOrder, _>(&mut reader)?,
    fixtures::DECLARED_REFS.map(String::from).to_vec()
  );

  Ok(())
}

#[test]
fn reads_a_visual_ending_in_a_fragment_too_short_for_a_header() -> XrfResult {
  let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(fixtures::trailing_fragment()?)?;
  let residue = file.residue.expect("Expect the stray bytes to be accounted for");

  assert_eq!(residue.cause, OgfResidueCause::TrailingFragment);
  assert_eq!(residue.bytes, b"\r\n".to_vec());

  Ok(())
}

#[test]
fn refuses_a_truncated_chunk() -> XrfResult {
  let error: String = match OgfFile::read_from_bytes::<XRayByteOrder>(fixtures::truncated_chunk()?) {
    Ok(_) => panic!("Expected a chunk declaring more than the file holds to be refused"),
    Err(error) => error.to_string(),
  };

  // The engine reads this chunk through `open_chunk`, which would build a reader over bytes that are not there, so
  // tolerating it would accept a file the engine itself cannot load.
  assert!(
    error.contains("declares 4343 bytes, 30 remain before source end"),
    "Unexpected error: {error}"
  );

  Ok(())
}

#[test]
fn refuses_trailing_bytes_that_belong_to_nothing() -> XrfResult {
  // The guard that keeps the rule narrow. These bytes are the same bytes the split-reference fixture ends with, but no
  // chunk was cut short to produce them, so nothing accounts for them and the strict error must survive.
  let strict: String = match OgfFile::read_from_bytes::<XRayByteOrder>(fixtures::unexplained_residue()?) {
    Ok(_) => panic!("Expected unaccountable trailing bytes to be refused"),
    Err(error) => error.to_string(),
  };

  assert!(
    strict.contains("beyond source end") || strict.contains("remain before source end"),
    "Expect the error a strict walk always raised, unchanged: {strict}"
  );

  Ok(())
}

#[test]
fn reads_a_counted_reference_the_declared_size_cuts_as_the_engine_does() -> XrfResult {
  // `CKinematicsAnimated::Load` finds the chunk and reads its strings off the visual's reader, so the reference runs on
  // past the declared size and the engine loads it whole: reading it short would name a bank that does not exist.
  let bytes: Vec<u8> = fixtures::split_counted_ref()?;
  let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(bytes.clone())?;
  let declared: u32 = (fixtures::kinematics_payload(&[fixtures::COUNTED_REF]).len() - 14) as u32;

  assert_eq!(
    file.kinematics.map(|it| it.motion_refs),
    Some(vec![String::from(fixtures::COUNTED_REF)])
  );
  assert_eq!(
    file.residue.map(|it| it.cause),
    Some(OgfResidueCause::UnderstatedMotionRefs { declared })
  );
  assert_eq!(
    OgfNormalization::normalize_bytes::<XRayByteOrder>(&bytes)?,
    fixtures::with_refs(&[fixtures::COUNTED_REF])?,
    "Expect normalizing to keep every byte and only give the chunk the size of what the engine reads"
  );

  Ok(())
}

#[test]
fn reads_counted_references_running_whole_past_the_declared_size() -> XrfResult {
  let refs: [&str; 3] = [
    "dynamics\\devices\\dev_hand_detector_1_hud_animation",
    "dynamics\\weapons\\wpn_hand_gp100_hud_animation",
    "dynamics\\weapons\\wpn_hand_m1891_hud_animation",
  ];
  let bytes: Vec<u8> = fixtures::spilled_refs(&refs, 30)?;

  assert_eq!(
    OgfFile::read_motion_refs_from_bytes::<XRayByteOrder>(bytes.clone())?,
    refs.map(String::from).to_vec()
  );
  assert_eq!(
    OgfNormalization::normalize_bytes::<XRayByteOrder>(&bytes)?,
    fixtures::with_refs(&refs)?
  );

  Ok(())
}

#[test]
fn refuses_counted_references_followed_by_bytes_nothing_reads() -> XrfResult {
  let mut bytes: Vec<u8> = fixtures::split_counted_ref()?;

  bytes.extend_from_slice(b"unexplained bytes");

  assert!(OgfFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

#[test]
fn reads_a_found_last_chunk_declaring_more_than_the_file_holds() -> XrfResult {
  let (overstated, exact) = fixtures::overstated_last_chunk()?;
  let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(overstated.clone())?;

  assert_eq!(
    file.kinematics.map(|it| it.motion_refs),
    Some(vec![String::from(
      "anomaly_weapons\\hud_animation\\wpn_axe_hud_animation"
    )])
  );
  assert_eq!(
    file.residue.map(|it| it.cause),
    Some(OgfResidueCause::OverstatedMotionRefs { declared: 56 })
  );
  assert_eq!(OgfNormalization::normalize_bytes::<XRayByteOrder>(&overstated)?, exact);

  Ok(())
}

#[test]
fn refuses_any_other_chunk_the_file_cuts_short() -> XrfResult {
  // A description cut mid-way is a truncated file: its payload does not read whole, so no size is merely wrong.
  let mut bytes: Vec<u8> = fixtures::header_bytes()?;

  bytes.extend_from_slice(&18u32.to_le_bytes());
  bytes.extend_from_slice(&141u32.to_le_bytes());
  bytes.extend_from_slice(b"x:\\rawdata\\objects\\cut");

  assert!(OgfFile::read_from_bytes::<XRayByteOrder>(bytes).is_err());

  Ok(())
}

#[test]
fn reads_a_compression_mark_the_engine_ignores_and_normalizes_it_away() -> XrfResult {
  let marked: Vec<u8> = fixtures::marked_header()?;
  let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(marked.clone())?;

  assert_eq!(file.header.version, 4);
  assert!(
    file.residue.is_none(),
    "Expect a mark on a found chunk to be no departure at all"
  );
  assert_eq!(
    OgfNormalization::normalize_bytes::<XRayByteOrder>(&marked)?,
    fixtures::well_formed()?
  );

  Ok(())
}

#[test]
fn refuses_a_compressed_chunk_the_engine_would_decompress() -> XrfResult {
  let error: String = OgfFile::read_from_bytes::<XRayByteOrder>(fixtures::marked_opened_chunk()?)
    .expect_err("expect a compressed user data chunk to be refused")
    .to_string();

  assert!(error.contains("Compressed ogf chunk 17"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn normalizes_only_what_the_engine_ignores() -> XrfResult {
  let well_formed: Vec<u8> = fixtures::well_formed()?;

  assert_eq!(
    OgfNormalization::normalize_bytes::<XRayByteOrder>(&well_formed)?,
    well_formed,
    "Expect a well-formed visual to normalize to itself byte for byte, which is what keeps the patch guard unchanged"
  );

  let split: Vec<u8> = fixtures::split_motion_ref()?;
  let normalized: Vec<u8> = OgfNormalization::normalize_bytes::<XRayByteOrder>(&split)?;

  assert_eq!(
    normalized.len(),
    split.len() - 34,
    "Expect both fragments of the fifth path to go, and nothing else"
  );
  assert_eq!(
    normalized, well_formed,
    "Expect normalizing to land on exactly the well-formed visual carrying the same four references"
  );

  let fragment: Vec<u8> = fixtures::trailing_fragment()?;

  assert_eq!(
    OgfNormalization::normalize_bytes::<XRayByteOrder>(&fragment)?,
    well_formed
  );

  Ok(())
}

#[test]
fn normalized_bytes_still_read_as_the_same_visual() -> XrfResult {
  let normalized: Vec<u8> = OgfNormalization::normalize_bytes::<XRayByteOrder>(&fixtures::split_motion_ref()?)?;
  let file: OgfFile = OgfFile::read_from_bytes::<XRayByteOrder>(normalized)?;

  assert_eq!(
    file.kinematics.map(|it| it.motion_refs),
    Some(fixtures::DECLARED_REFS.map(String::from).to_vec()),
    "Expect the engine to load exactly what it loaded before normalizing"
  );
  assert!(file.residue.is_none(), "Expect a normalized visual to carry no residue");

  Ok(())
}

#[test]
fn refuses_to_normalize_what_it_cannot_account_for() -> XrfResult {
  assert!(
    OgfNormalization::normalize_bytes::<XRayByteOrder>(&fixtures::unexplained_residue()?).is_err(),
    "Expect normalization to refuse a file the reader refuses, rather than truncate it to the last good chunk"
  );

  Ok(())
}

#[test]
fn surveys_report_residue_rather_than_tolerating_it() -> XrfResult {
  // The survey exists to answer whether parsing is complete, so a silent pass here would be the one wrong answer.
  let path: std::path::PathBuf = GeneratedResources::write(
    &GeneratedResources::sample_name(file!(), "survey.ogf"),
    fixtures::split_motion_ref()?,
  )?;
  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(std::fs::read(path)?)?;

  // Surveyed after the visual was read through the same reader, as `ogf info` does to read the file once.
  OgfFile::read_from_chunk::<XRayByteOrder, _>(&mut reader)?;

  let survey = OgfChunksProcessor::collect_chunks::<XRayByteOrder, _>(&mut reader)?;

  assert_eq!(
    survey.entries.iter().map(|it| it.id).collect::<Vec<u32>>(),
    vec![1, OgfKinematicsChunk::CHUNK_ID]
  );
  assert!(
    survey.residue.is_some(),
    "Expect the survey to carry the bytes it could not walk"
  );

  Ok(())
}
