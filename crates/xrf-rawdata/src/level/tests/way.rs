use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{EditableObjectPart, EditableTextPart, EditableWayObject};
use xrf_error::XrfResult;
use xrf_level::{LevelGameFile, LevelGameWay, LevelGameWayLink, LevelGameWayPoint};
use xrf_math::Vector3d;

use crate::level::tests::GUID;
use crate::{LevelDecompileContext, LevelWayConversion};

fn way(name: &str, points: usize, links: &[(u16, u16, f32)]) -> LevelGameWay {
  LevelGameWay {
    version: EditableWayObject::VERSION as u16,
    name: name.to_owned(),
    kind: None,
    points: (0..points)
      .map(|index| LevelGameWayPoint {
        position: Vector3d::new(index as f32 * 2.5, -1.25, 40.0 - index as f32),
        flags: if index == 0 { 0x8000_0001 } else { 0 },
        name: format!("wp{index:02}"),
      })
      .collect(),
    links: links
      .iter()
      .map(|&(from, to, probability)| LevelGameWayLink { from, to, probability })
      .collect(),
  }
}

fn bytes_of(ways: Vec<LevelGameWay>) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelGameFile {
    rpoints: Vec::new(),
    ways,
  }
  .write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

#[test]
fn patrol_paths_come_back_to_the_same_file_through_the_parts_text() -> XrfResult {
  let ways: Vec<LevelGameWay> = vec![
    way(
      "lx8_patrol_walk",
      3,
      &[(0, 1, 1.0), (0, 2, 0.5), (1, 0, 1.0), (2, 1, 0.25)],
    ),
    way("lx8_patrol_look", 1, &[]),
  ];
  let part = LevelWayConversion::decompile(&ways, &mut LevelDecompileContext::new(GUID))?;
  let text: Vec<u8> = part.to_text_part()?.write_to_bytes()?;
  let read: EditableObjectPart<EditableWayObject> =
    EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(&text)?)?;

  assert_eq!(read, part);
  assert_eq!(read.objects[0].object.name, "lx8_patrol_walk");
  assert_eq!(read.objects[0].points[0].links.len(), 2);
  assert!(
    read
      .objects
      .iter()
      .flat_map(|way| &way.points)
      .all(|point| !point.selected)
  );
  assert_eq!(bytes_of(LevelWayConversion::export(&read)?)?, bytes_of(ways)?);

  Ok(())
}

#[test]
fn exports_links_point_by_point_with_no_type() -> XrfResult {
  // The editor keeps a point's links on the point, so the file lists them by the point they leave; a type a path keeps
  // is the way's, though CoP's export writes none.
  let mut typed: LevelGameWay = way("walk", 3, &[(2, 0, 1.0), (0, 1, 1.0)]);

  typed.kind = Some(3);

  let part = LevelWayConversion::decompile(&[typed], &mut LevelDecompileContext::new(GUID))?;
  let exported: Vec<LevelGameWay> = LevelWayConversion::export(&part)?;

  assert_eq!(part.objects[0].kind, 3);
  assert_eq!(exported[0].kind, None);
  assert_eq!(
    exported[0]
      .links
      .iter()
      .map(|link| (link.from, link.to))
      .collect::<Vec<_>>(),
    [(0, 1), (2, 0)]
  );

  Ok(())
}

#[test]
fn refuses_a_link_to_a_point_the_path_does_not_hold() {
  let error: String =
    LevelWayConversion::decompile(&[way("walk", 2, &[(0, 2, 1.0)])], &mut LevelDecompileContext::new(GUID))
      .unwrap_err()
      .to_string();

  assert!(error.contains("Patrol path 'walk' links point 0 to 2 of 2"), "{error}");
}
