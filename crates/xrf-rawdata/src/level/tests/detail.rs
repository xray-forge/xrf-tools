use xrf_editable_level::EditableDetailObjectTool;
use xrf_level::{DetailModel, DetailVertex, LevelDetailsFile, LevelDetailsHeader};
use xrf_math::Vector3d;

use crate::level::level_detail_conversion::{LevelDetailConversion, LevelDetailDecompile};
use crate::level::level_detail_library::LevelDetailLibrary;

/// A model of one quad, `size` across, its coordinates `uv` gives each corner's.
fn quad(size: f32, uv: impl Fn(f32, f32) -> (f32, f32)) -> DetailModel {
  DetailModel {
    shader: String::from("details\\blend"),
    texture: String::from("detail\\detail_grass"),
    flags: 0,
    min_scale: 0.5,
    max_scale: 2.0,
    vertices: [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
      .into_iter()
      .map(|(x, y)| {
        let (u, v) = uv(x, y);

        DetailVertex {
          position: Vector3d::new(x * size, y * size, 0.0),
          u,
          v,
        }
      })
      .collect(),
    indices: vec![0, 1, 2, 0, 2, 3],
  }
}

/// A slot planting `corners`' models, its base and height set and its light full.
fn slot(corners: [u8; 4]) -> [u32; 4] {
  let objects: u64 = corners
    .iter()
    .enumerate()
    .fold(0, |word, (corner, id)| word | (u64::from(*id) << (20 + 6 * corner)));
  let word: u64 = 1000 | (12 << 12) | objects | (0xFFFFF << 44);

  [word as u32, (word >> 32) as u32, 0x1234, 0x5678]
}

fn details(objects: Vec<DetailModel>, slots: Vec<[u32; 4]>) -> LevelDetailsFile {
  LevelDetailsFile {
    header: LevelDetailsHeader {
      version: LevelDetailsFile::CURRENT_VERSION,
      object_count: objects.len() as u32,
      offset_x: 4,
      offset_z: 2,
      size_x: slots.len() as u32,
      size_z: 1,
    },
    objects,
    slots,
  }
}

fn library() -> LevelDetailLibrary {
  LevelDetailLibrary::new(vec![
    (String::from("detail\\small"), quad(1.0, |u, v| (u, v))),
    (String::from("detail\\grass"), quad(2.0, |u, v| (u, v))),
  ])
}

/// A model the library draws comes back as its reference at the level's scale range and flags, its coordinates the
/// library's moved into the atlas, turned or not; a model it lacks is refused, and the corners planting it plant
/// nothing. The slots keep their heights and densities, their light zeroed.
#[test]
fn decompiles_details_into_the_library_objects_drawing_them() {
  let mut atlas: DetailModel = quad(2.0, |u, v| (0.25 * u + 0.5, 0.5 * v));
  let turned: DetailModel = quad(1.0, |u, v| (0.125 * v, 0.25 * u + 0.75));
  let unknown: DetailModel = quad(3.0, |u, v| (u, v));

  atlas.flags = 1;
  atlas.min_scale = 0.8;

  let file: LevelDetailsFile = details(
    vec![atlas, unknown, turned],
    vec![slot([0, 1, 2, 63]), slot([63, 63, 63, 63])],
  );
  let LevelDetailDecompile { tool, refused } = LevelDetailConversion::decompile(&file, &library());
  let tool: EditableDetailObjectTool = tool.expect("two models found");

  assert_eq!(
    tool
      .objects
      .iter()
      .map(|object| (object.reference.as_str(), object.flags, object.min_scale))
      .collect::<Vec<_>>(),
    [("detail\\grass", 1, 0.8), ("detail\\small", 0, 0.5)]
  );
  assert_eq!(refused.len(), 1);
  assert_eq!(refused[0].object, "detail object 1");
  assert_eq!(
    (0..4)
      .map(|corner| EditableDetailObjectTool::get_slot_object(&tool.slots[0], corner))
      .collect::<Vec<_>>(),
    [Some(0), None, Some(1), None]
  );
  assert_eq!(tool.slots[0][1] >> 12, 0, "the light zeroed");
  assert_eq!((tool.slots[0][0] & 0xF_FFFF), 1000 | (12 << 12), "the height kept");
  assert_eq!(
    [tool.slots[0][2], tool.slots[0][3]],
    [0x1234, 0x5678],
    "the densities kept"
  );
  assert_eq!(
    (tool.header.object_count, tool.header.offset_x, tool.header.size_x),
    (2, 4, 2)
  );
}

/// Where the library draws none of the level's models, there is no tool: the Build would refuse one with nothing to
/// merge into its atlas.
#[test]
fn leaves_the_tool_out_where_no_model_is_found() {
  let file: LevelDetailsFile = details(vec![quad(3.0, |u, v| (u, v))], vec![slot([0, 63, 63, 63])]);
  let decompile: LevelDetailDecompile = LevelDetailConversion::decompile(&file, &LevelDetailLibrary::default());

  assert!(decompile.tool.is_none());
  assert_eq!(decompile.refused.len(), 1);
}

/// A model of the library's geometry whose coordinates no single remap reaches is another object's.
#[test]
fn matches_no_model_whose_coordinates_are_no_remap() {
  let bent: DetailModel = quad(2.0, |u, v| (u + 0.3 * v, v));

  assert_eq!(library().find(&bent), None);
  assert_eq!(library().find(&quad(2.0, |u, v| (u, v))), Some("detail\\grass"));
}
