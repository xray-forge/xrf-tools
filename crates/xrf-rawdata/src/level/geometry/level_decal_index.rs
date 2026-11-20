use std::collections::{HashMap, HashSet};

use xrf_level::LevelWallmarksFile;
use xrf_math::Vector3d;

/// A level's wall marks, indexed to tell which static render triangles are their copies.
///
/// The Build adds every decal to the static faces as well as to `level.wallmarks` (`ESceneWallmarkTool::ExportStatic`),
/// so a scene keeping both `wallmark.part` and those copies would draw each decal twice.
pub struct LevelDecalIndex {
  /// Each decal triangle's centre, by its slot's texture lowercased, in cells of [`Self::REACH`].
  centres: HashMap<String, HashMap<[i32; 3], Vec<Vector3d>>>,
  textures: HashSet<String>,
}

impl LevelDecalIndex {
  /// How far, metres, a static triangle's centre may lie from a decal triangle's to be its copy. Every such triangle
  /// of vanilla CoP lies within it: 6,601 exactly, 5,021 within 1 mm, 146 beyond.
  pub const REACH: f32 = 0.01;

  pub fn new(wallmarks: &LevelWallmarksFile) -> Self {
    let mut index: Self = Self {
      centres: HashMap::new(),
      textures: HashSet::new(),
    };

    for slot in &wallmarks.slots {
      let texture: String = slot.texture.to_lowercase();

      for mark in &slot.marks {
        for [a, b, c] in mark.vertices.as_chunks::<3>().0 {
          let centre: Vector3d = Self::get_centre(&[a.position, b.position, c.position]);

          index
            .centres
            .entry(texture.clone())
            .or_default()
            .entry(Self::get_cell(&centre))
            .or_default()
            .push(centre);
        }
      }

      index.textures.insert(texture);
    }

    index
  }

  /// Whether a static triangle of `points`, drawn with `texture`, is a decal's copy: of a slot's texture, its centre
  /// within [`Self::REACH`] of one of that slot's decal triangles'.
  pub fn holds(&self, texture: &str, points: &[Vector3d; 3]) -> bool {
    let texture: String = texture.to_lowercase();
    let Some(cells) = self.centres.get(&texture) else {
      return false;
    };
    let centre: Vector3d = Self::get_centre(points);
    let [x, y, z] = Self::get_cell(&centre);

    (-1..=1).any(|dx| {
      (-1..=1).any(|dy| {
        (-1..=1).any(|dz| {
          cells
            .get(&[x + dx, y + dy, z + dz])
            .is_some_and(|centres| centres.iter().any(|it| (it - &centre).length() <= Self::REACH))
        })
      })
    })
  }

  /// Whether any slot draws with `texture`, so a triangle of another needs no look.
  pub fn is_decal_texture(&self, texture: &str) -> bool {
    self.textures.contains(&texture.to_lowercase())
  }

  fn get_centre(points: &[Vector3d; 3]) -> Vector3d {
    &(&(&points[0] + &points[1]) + &points[2]) / 3.0
  }

  fn get_cell(point: &Vector3d) -> [i32; 3] {
    [point.x, point.y, point.z].map(|axis| (axis / Self::REACH).floor() as i32)
  }
}
