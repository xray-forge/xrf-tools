use xrf_math::{EPS_S, Vector3d};

use crate::compiler_shaders::CompilerShaders;
use crate::geometry::geometry_mesh::GeometryMesh;
use crate::geometry::geometry_split_box::GeometrySplitBox;

/// A level's rendered faces grouped into the splits that each become a visual, as xrLC groups them: by material
/// (`xrPhase_ResolveMaterials`), cut in space (`xrPhase_Subdivide`), then joined again where small
/// (`xrPhase_MergeGeometry`).
///
/// - **By material.** Materials are taken in the order faces first name them, every face counted; a split each holds
///   that material's faces whose compiler shader renders, in face order, and a split left empty goes.
/// - **Cut.** A split of at least [`Self::LOW_FACES`] twice over whose box is wider than [`Self::MAXIMUM_SIZE`] on an
///   axis, or which holds more than [`Self::HIGH_FACES`], is halved on its box's longest axis, each face going by its
///   centre to the lower half's closed box or else the upper. Where a half would hold under [`Self::LOW_FACES`], the
///   cut moves by half the larger half's width towards it, up to ten times while the split holds more than four times
///   the least, then turns to the next axis, three axes in all. A split cut goes, its halves joining the end of the list,
///   where they are cut in their turn. xrLC also cuts a split whose lightmap chart outgrows a lightmap; a chart is the
///   light baker's, and under draft light no split has one.
/// - **Joined.** Each split in turn, while it holds under three quarters of the high limit and its box, grown by
///   `EPS_S`, is narrower than the size on some axis, takes in the later split of its material that needs joining too
///   and gives the smallest joined box by volume, the first of equals: one that keeps the two under four thirds of the
///   high limit, keeps every axis of the joined box within four thirds of the size or within a metre of either part's,
///   and is no more than eight times the two parts' cubes. Its faces go before the split's own. xrLC also requires the
///   same lightmap; under draft light every lightmapped face has the one.
///
/// Which split a face lands in is left out of the compiler's bar, as xrLC's own splits follow its lightmaps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeometrySplits {
  /// Each split's faces, by index into the mesh's faces.
  pub splits: Vec<Vec<u32>>,
}

impl GeometrySplits {
  /// `c_SS_LowVertLimit`: the fewest faces a half of a cut may hold.
  pub const LOW_FACES: usize = 64;

  /// `c_SS_HighVertLimit`: the most faces a split may hold before it is cut.
  pub const HIGH_FACES: usize = 2 * 1024;

  /// `c_SS_maxsize`: the widest a split's box may be on an axis before it is cut.
  pub const MAXIMUM_SIZE: f32 = 32.0;

  /// The splits of `mesh`'s faces whose compiler shader renders, by `shaders`.
  pub fn of(mesh: &GeometryMesh, shaders: &CompilerShaders) -> Self {
    let mut splits: Vec<Vec<u32>> = Self::by_material(mesh, shaders);

    Self::subdivide(mesh, &mut splits);
    Self::merge(mesh, &mut splits);

    Self { splits }
  }

  /// `xrPhase_ResolveMaterials`: a split a material, in the order faces first name it.
  fn by_material(mesh: &GeometryMesh, shaders: &CompilerShaders) -> Vec<Vec<u32>> {
    let mut materials: Vec<u16> = Vec::new();

    for face in &mesh.faces {
      if !materials.contains(&face.material) {
        materials.push(face.material);
      }
    }

    let mut splits: Vec<Vec<u32>> = vec![Vec::new(); materials.len()];

    for (index, face) in mesh.faces.iter().enumerate() {
      if shaders.get(face.material).is_some_and(|shader| shader.is_rendered())
        && let Some(slot) = materials.iter().position(|material| *material == face.material)
      {
        splits[slot].push(index as u32);
      }
    }

    splits.retain(|split| !split.is_empty());
    splits
  }

  /// `xrPhase_Subdivide`.
  fn subdivide(mesh: &GeometryMesh, splits: &mut Vec<Vec<u32>>) {
    let mut index: usize = 0;

    while index < splits.len() {
      let split: &[u32] = &splits[index];

      if split.len() < Self::LOW_FACES * 2 {
        index += 1;
        continue;
      }

      let bounds: GeometrySplitBox = GeometrySplitBox::of(mesh, split);
      let size: [f32; 3] = bounds.get_size();

      if size.iter().all(|side| *side <= Self::MAXIMUM_SIZE) && split.len() <= Self::HIGH_FACES {
        index += 1;
        continue;
      }

      match Self::cut(mesh, split, &bounds, size) {
        Some([lower, upper]) => {
          splits.remove(index);
          splits.push(lower);
          splits.push(upper);
        }
        None => index += 1,
      }
    }
  }

  /// One split's cut into two halves of at least [`Self::LOW_FACES`] each, or none found.
  fn cut(mesh: &GeometryMesh, split: &[u32], bounds: &GeometrySplitBox, size: [f32; 3]) -> Option<[Vec<u32>; 2]> {
    let mut axis: usize = if size[0] >= size[1] && size[0] >= size[2] {
      0
    } else if size[1] >= size[0] && size[1] >= size[2] {
      1
    } else {
      2
    };
    let [mut lower_box, mut upper_box] = bounds.halve(axis);
    let mut axes_tried: u32 = 0;
    let mut tries_on_axis: u32 = 0;

    loop {
      tries_on_axis += 1;

      let (lower, upper): (Vec<u32>, Vec<u32>) = split
        .iter()
        .partition(|face| lower_box.contains(&Self::get_center(mesh, **face)));

      if lower.len() >= Self::LOW_FACES && upper.len() >= Self::LOW_FACES {
        return Some([lower, upper]);
      }

      if tries_on_axis < 10 {
        if split.len() <= Self::LOW_FACES * 4 {
          return None;
        }

        if upper.len() > lower.len() {
          let width: f32 = upper_box.get_size()[axis];

          lower_box.max[axis] += width / 2.0;
          upper_box.min[axis] = lower_box.max[axis];
        } else {
          let width: f32 = lower_box.get_size()[axis];

          upper_box.min[axis] -= width / 2.0;
          lower_box.max[axis] = upper_box.min[axis];
        }
      } else {
        tries_on_axis = 0;
        axes_tried += 1;

        if axes_tried >= 3 {
          return None;
        }

        axis = (axis + 1) % 3;
        [lower_box, upper_box] = bounds.halve(axis);
      }
    }
  }

  /// `xrPhase_MergeGeometry`.
  fn merge(mesh: &GeometryMesh, splits: &mut Vec<Vec<u32>>) {
    let mut boxes: Vec<GeometrySplitBox> = splits.iter().map(|split| GeometrySplitBox::of(mesh, split)).collect();
    let material = |split: &[u32]| mesh.faces[split[0] as usize].material;
    let mut index: usize = 0;

    while index < splits.len() {
      let mut original: Option<GeometrySplitBox> = None;

      while let Some(grown) = Self::get_mergeable_box(splits[index].len(), &boxes[index]) {
        let original: GeometrySplitBox = *original.get_or_insert(grown);
        let mut selected: Option<(usize, f32)> = None;

        for candidate in index + 1..splits.len() {
          if material(&splits[candidate]) != material(&splits[index]) {
            continue;
          }

          let Some(other) = Self::get_mergeable_box(splits[candidate].len(), &boxes[candidate]) else {
            continue;
          };
          let Some(volume) =
            Self::get_merged_volume(splits[index].len(), &grown, &original, splits[candidate].len(), &other)
          else {
            continue;
          };

          if selected.is_none_or(|(_, best)| volume < best) {
            selected = Some((candidate, volume));
          }
        }

        let Some((candidate, _)) = selected else {
          break;
        };
        let taken: Vec<u32> = splits.remove(candidate);
        let taken_box: GeometrySplitBox = boxes.remove(candidate);

        splits[index].splice(0..0, taken);
        boxes[index] = boxes[index].merge(&taken_box);
      }

      index += 1;
    }
  }

  /// `NeedMerge`: a split's box grown by `EPS_S` where it holds under three quarters of the high limit and is narrower
  /// than the size on some axis; none where it needs no joining.
  fn get_mergeable_box(faces: usize, bounds: &GeometrySplitBox) -> Option<GeometrySplitBox> {
    if faces >= 3 * Self::HIGH_FACES / 4 {
      return None;
    }

    let grown: GeometrySplitBox = bounds.grow(EPS_S);

    grown
      .get_size()
      .iter()
      .any(|side| *side < Self::MAXIMUM_SIZE)
      .then_some(grown)
  }

  /// `ValidateMerge`: the joined box's volume where two splits may join, none where they may not.
  fn get_merged_volume(
    faces: usize,
    bounds: &GeometrySplitBox,
    original: &GeometrySplitBox,
    other_faces: usize,
    other: &GeometrySplitBox,
  ) -> Option<f32> {
    if faces + other_faces > 4 * Self::HIGH_FACES / 3 {
      return None;
    }

    let merged: GeometrySplitBox = bounds.merge(other);
    let size: [f32; 3] = merged.get_size();
    let (original_size, other_size): ([f32; 3], [f32; 3]) = (original.get_size(), other.get_size());
    // `4 * c_SS_maxsize / 3` in unsigned integers.
    let widest: f32 = (4 * Self::MAXIMUM_SIZE as u32 / 3) as f32;

    if (0..3)
      .any(|axis| size[axis] > widest && size[axis] > original_size[axis] + 1.0 && size[axis] > other_size[axis] + 1.0)
    {
      return None;
    }

    let volume: f32 = merged.get_volume();

    (volume <= 2.0 * 2.0 * 2.0 * (bounds.get_cube().get_volume() + other.get_cube().get_volume())).then_some(volume)
  }

  /// `CalcCenter`: the corners summed in order and divided by three.
  fn get_center(mesh: &GeometryMesh, face: u32) -> Vector3d {
    let [first, second, third] = mesh.faces[face as usize].get_corners(&mesh.vertices);
    let sum: Vector3d = &(&first + &second) + &third;

    Vector3d::new(sum.x / 3.0, sum.y / 3.0, sum.z / 3.0)
  }
}
