use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::level::level_file::LevelFile;
use crate::level::level_header_chunk::LevelHeaderChunk;
use crate::level::portals::level_portal::LevelPortal;
use crate::level::portals::level_portals_chunk::LevelPortalsChunk;
use crate::level::sectors::level_sector::LevelSector;
use crate::level::sectors::level_sectors_chunk::LevelSectorsChunk;
use crate::tests::fixtures::{chunk, floats};

/// One sector as the file stores it: a portal id run, then the root visual.
fn sector(portals: &[u16], root: u32) -> XrfResult<Vec<u8>> {
  let ids: Vec<u8> = portals.iter().flat_map(|id| id.to_le_bytes()).collect();
  let mut bytes: Vec<u8> = chunk(LevelSector::PORTALS_CHUNK_ID, &ids)?;

  bytes.extend(chunk(LevelSector::ROOT_CHUNK_ID, &root.to_le_bytes())?);

  Ok(bytes)
}

/// One portal record: two sector ids, six vectors, and how many of them are the polygon.
fn portal(front: u16, back: u16, count: u32) -> Vec<u8> {
  let mut bytes: Vec<u8> = front.to_le_bytes().to_vec();

  bytes.extend_from_slice(&back.to_le_bytes());
  bytes.extend(floats(&[
    0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
  ]));
  bytes.extend_from_slice(&count.to_le_bytes());

  bytes
}

/// A whole `level` carrying a header, the sectors and the portals.
fn level(sectors: &[Vec<u8>], portals: &[Vec<u8>]) -> XrfResult<Vec<u8>> {
  let mut body: Vec<u8> = Vec::new();

  for (index, sector) in sectors.iter().enumerate() {
    body.extend(chunk(index as u32, sector)?);
  }

  let mut bytes: Vec<u8> = chunk(LevelHeaderChunk::CHUNK_ID, &[14, 0, 1, 0])?;

  bytes.extend(chunk(LevelSectorsChunk::CHUNK_ID, &body)?);
  bytes.extend(chunk(
    LevelPortalsChunk::CHUNK_ID,
    &portals.iter().flatten().copied().collect::<Vec<u8>>(),
  )?);

  Ok(bytes)
}

#[test]
fn test_reads_sectors_and_the_portals_they_name() -> XrfResult {
  let read: LevelFile = LevelFile::read_from_bytes::<XRayByteOrder>(level(
    &[sector(&[0, 1], 7)?, sector(&[1], 9)?],
    &[portal(0, 1, 4), portal(1, 0, 3)],
  )?)?;

  let sectors: &LevelSectorsChunk = read.sectors.as_ref().expect("a level with sectors");

  assert_eq!(sectors.sectors.len(), 2);
  assert_eq!(sectors.sectors[0].portals, vec![0, 1]);
  assert_eq!(sectors.sectors[0].root, 7);
  assert_eq!(sectors.sectors[1].portals, vec![1]);
  assert_eq!(sectors.sectors[1].root, 9);

  let portals: &LevelPortalsChunk = read.portals.as_ref().expect("a level with portals");

  assert_eq!(portals.portals.len(), 2);
  assert_eq!(portals.portals[0].sector_front, 0);
  assert_eq!(portals.portals[0].sector_back, 1);
  assert_eq!(portals.portals[0].vertex_count, 4);

  Ok(())
}

#[test]
fn test_a_portal_keeps_the_vectors_past_its_count_and_hands_back_only_the_polygon() -> XrfResult {
  let read: LevelFile = LevelFile::read_from_bytes::<XRayByteOrder>(level(&[sector(&[0], 0)?], &[portal(0, 0, 3)])?)?;
  let portals: &LevelPortalsChunk = read.portals.as_ref().expect("a level with portals");
  let portal: &LevelPortal = &portals.portals[0];

  assert_eq!(
    portal.vertices.len(),
    LevelPortal::MAXIMUM_VERTICES,
    "the record always holds six, whatever the count says"
  );
  assert_eq!(portal.get_polygon().expect("a polygon").len(), 3);
  assert_eq!(
    portal.get_polygon().expect("a polygon")[1],
    Vector3d { x: 1.0, y: 0.0, z: 0.0 }
  );
  assert!(portal.is_polygon());

  Ok(())
}

#[test]
fn test_a_count_no_polygon_could_have_is_told_apart() -> XrfResult {
  for count in [0, 1, 2, 7, u32::MAX] {
    let read: LevelFile =
      LevelFile::read_from_bytes::<XRayByteOrder>(level(&[sector(&[0], 0)?], &[portal(0, 0, count)])?)?;
    let portal: &LevelPortal = &read.portals.as_ref().expect("a level with portals").portals[0];

    assert!(!portal.is_polygon(), "a count of {count} is not a polygon");
  }

  // A count past the room hands back nothing rather than stale vectors.
  let read: LevelFile = LevelFile::read_from_bytes::<XRayByteOrder>(level(&[sector(&[0], 0)?], &[portal(0, 0, 7)])?)?;

  assert_eq!(read.portals.expect("portals").portals[0].get_polygon(), None);

  Ok(())
}

#[test]
fn test_refuses_a_portals_chunk_that_does_not_divide_into_records() -> XrfResult {
  let mut short: Vec<u8> = portal(0, 0, 3);

  short.truncate(short.len() - 1);

  assert!(
    LevelFile::read_from_bytes::<XRayByteOrder>(level(&[sector(&[0], 0)?], &[short])?).is_err(),
    "a partial record is refused rather than read as a whole one"
  );

  Ok(())
}

#[test]
fn test_refuses_a_sector_missing_either_of_its_chunks() -> XrfResult {
  let mut body: Vec<u8> = chunk(LevelSector::PORTALS_CHUNK_ID, &0u16.to_le_bytes())?;
  let mut bytes: Vec<u8> = chunk(LevelHeaderChunk::CHUNK_ID, &[14, 0, 1, 0])?;

  bytes.extend(chunk(LevelSectorsChunk::CHUNK_ID, &chunk(0, &body)?)?);

  assert!(
    LevelFile::read_from_bytes::<XRayByteOrder>(bytes).is_err(),
    "a sector without a root is refused, as the engine asserts on it"
  );

  body = chunk(LevelSector::ROOT_CHUNK_ID, &0u32.to_le_bytes())?;

  let mut bytes: Vec<u8> = chunk(LevelHeaderChunk::CHUNK_ID, &[14, 0, 1, 0])?;

  bytes.extend(chunk(LevelSectorsChunk::CHUNK_ID, &chunk(0, &body)?)?);

  assert!(
    LevelFile::read_from_bytes::<XRayByteOrder>(bytes).is_err(),
    "a sector without a portal list is refused too"
  );

  Ok(())
}
