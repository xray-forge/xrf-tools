use std::collections::HashMap;

use xrf_math::{EPS_S, Vector3d};

use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::geometry::split::geometry_cut::GeometryCut;
use crate::geometry::split::geometry_split_box::GeometrySplitBox;
use crate::input::compiler_shaders::CompilerShaders;
use crate::light::lightmap::chart::light_chart::LightChart;
use crate::light::lightmap::chart::light_chart_scene::LightChartScene;

/// One split while the splits are cut: its faces, and the chart it holds whole, by index into the charts so far.
struct ChartedSplit {
  faces: Vec<u32>,
  chart: Option<usize>,
}

/// A level's rendered faces grouped into the splits that each become a visual, as xrLC groups them: by material
/// (`xrPhase_ResolveMaterials`), each lightmapped one into its charts (`xrPhase_UVmap`), cut in space
/// (`xrPhase_Subdivide`), then joined again where small ([`Self::merge`], `xrPhase_MergeGeometry`).
///
/// A split is cut on its box's longest axis while it is wider than [`Self::MAXIMUM_SIZE`] or holds more than
/// [`Self::HIGH_FACES`], each half keeping at least [`Self::LOW_FACES`]; a chart's split cut gives each half a chart of
/// its own. xrLC also cuts a split whose chart outgrows a page, which a chart's largest side never does.
///
/// Which split a face lands in is not held to xrLC's, as xrLC's splits follow its lightmap pages, and this crate packs
/// the charts onto pages its own way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GeometrySplits {
  /// Each split's faces, by index into the mesh's faces.
  pub splits: Vec<Vec<u32>>,
}

impl GeometrySplits {
  /// `c_SS_LowVertLimit`: the fewest faces a half of a cut may hold.
  pub(crate) const LOW_FACES: usize = 64;

  /// `c_SS_HighVertLimit`: the most faces a split may hold before it is cut.
  pub(crate) const HIGH_FACES: usize = 2 * 1024;

  /// `c_SS_maxsize`: the widest a split's box may be on an axis before it is cut.
  pub(crate) const MAXIMUM_SIZE: f32 = 32.0;

  /// The splits of `mesh`'s faces whose compiler shader renders, by `shaders`, with no chart cut: cut and joined.
  #[cfg(test)]
  pub(crate) fn of(mesh: &GeometryMesh, shaders: &CompilerShaders) -> Self {
    let mut splits: Self = Self::cut(mesh, shaders, None).splits;

    splits.merge(mesh, None);
    splits
  }

  /// The splits of `mesh`'s faces whose compiler shader renders, by `shaders`, before they are joined, and where
  /// `scene` is given the lightmap charts they were cut by, in xrLC's order of its deflectors: a chart cut in two
  /// leaves it, its halves joining the end.
  pub(crate) fn cut(
    mesh: &GeometryMesh,
    shaders: &CompilerShaders,
    scene: Option<&LightChartScene<'_>>,
  ) -> GeometryCut {
    let mut splits: Vec<ChartedSplit> = Self::by_material(mesh, shaders)
      .into_iter()
      .map(|faces| ChartedSplit { faces, chart: None })
      .collect();
    let mut charts: Vec<Option<LightChart>> = Vec::new();

    if let Some(scene) = scene {
      Self::chart(&mut splits, &mut charts, scene);
    }

    Self::subdivide(mesh, &mut splits, &mut charts, scene);

    GeometryCut {
      splits: Self {
        splits: splits.into_iter().map(|split| split.faces).collect(),
      },
      charts: charts.into_iter().flatten().collect(),
    }
  }

  /// `xrPhase_UVmap`: each split of a lightmapped material cut into its charts, each chart's faces a split of their own
  /// at the end in the split's order; a split left empty goes.
  fn chart(splits: &mut Vec<ChartedSplit>, charts: &mut Vec<Option<LightChart>>, scene: &LightChartScene<'_>) {
    let mut kept: Vec<ChartedSplit> = Vec::with_capacity(splits.len());
    let mut charted: Vec<ChartedSplit> = Vec::new();

    for ChartedSplit { faces: split, chart } in splits.drain(..) {
      let material: u16 = split
        .first()
        .map_or(0, |face| scene.mesh.faces[*face as usize].material);

      if !scene.is_charted(material) {
        kept.push(ChartedSplit { faces: split, chart });
        continue;
      }

      let cuts: Vec<LightChart> = LightChart::cut(&split, scene);
      let owners: HashMap<u32, usize> = cuts
        .iter()
        .enumerate()
        .flat_map(|(index, cut)| cut.faces.iter().map(move |face| (*face, index)))
        .collect();
      let mut taken: Vec<Vec<u32>> = vec![Vec::new(); cuts.len()];
      let mut left: Vec<u32> = Vec::new();

      for face in split {
        match owners.get(&face) {
          Some(index) => taken[*index].push(face),
          None => left.push(face),
        }
      }

      for (cut, faces) in cuts.into_iter().zip(taken) {
        charts.push(Some(cut));
        charted.push(ChartedSplit {
          faces,
          chart: Some(charts.len() - 1),
        });
      }

      if !left.is_empty() {
        kept.push(ChartedSplit {
          faces: left,
          chart: None,
        });
      }
    }

    splits.extend(kept);
    splits.extend(charted);
  }

  /// Joins the splits as `xrPhase_MergeGeometry` does, each only with splits of its first face's material and, where
  /// `pages` gives each face's lightmap page, page.
  ///
  /// Only splits of one material and page can join, so each looks for its candidates among the later splits of its own
  /// kind alone, in their order, which gives xrLC's choice without its search over every split.
  pub(crate) fn merge(&mut self, mesh: &GeometryMesh, pages: Option<&[Option<u32>]>) {
    // A split of no face has no kind to join by, and draws nothing.
    let mut splits: Vec<Option<Vec<u32>>> = self
      .splits
      .drain(..)
      .filter(|split| !split.is_empty())
      .map(Some)
      .collect();
    let mut boxes: Vec<GeometrySplitBox> = splits
      .iter()
      .flatten()
      .map(|split| GeometrySplitBox::of(mesh, split))
      .collect();
    let kind = |split: &[u32]| {
      let face: u32 = split[0];

      (
        mesh.faces[face as usize].material,
        pages.and_then(|pages| pages[face as usize]),
      )
    };
    let mut kinds: HashMap<(u16, Option<u32>), Vec<usize>> = HashMap::new();

    for (index, split) in splits.iter().enumerate() {
      kinds
        .entry(kind(split.as_deref().unwrap_or_default()))
        .or_default()
        .push(index);
    }

    for index in 0..splits.len() {
      let Some(split) = splits[index].as_deref() else {
        continue;
      };
      let same: &mut Vec<usize> = kinds.get_mut(&kind(split)).expect("every split's kind");
      let mut original: Option<GeometrySplitBox> = None;

      while let Some(grown) = Self::get_mergeable_box(splits[index].as_ref().map_or(0, Vec::len), &boxes[index]) {
        let original: GeometrySplitBox = *original.get_or_insert(grown);
        let faces: usize = splits[index].as_ref().map_or(0, Vec::len);
        let mut selected: Option<(usize, f32)> = None;
        let start: usize = same.partition_point(|other| *other <= index);

        for (slot, candidate) in same.iter().enumerate().skip(start) {
          let other_faces: usize = splits[*candidate].as_ref().map_or(0, Vec::len);
          let Some(other) = Self::get_mergeable_box(other_faces, &boxes[*candidate]) else {
            continue;
          };
          let Some(volume) = Self::get_merged_volume(faces, &grown, &original, other_faces, &other) else {
            continue;
          };

          if selected.is_none_or(|(_, best)| volume < best) {
            selected = Some((slot, volume));
          }
        }

        let Some((slot, _)) = selected else {
          break;
        };
        let candidate: usize = same.remove(slot);
        let taken: Vec<u32> = splits[candidate].take().unwrap_or_default();
        let taken_box: GeometrySplitBox = boxes[candidate];

        if let Some(split) = splits[index].as_mut() {
          split.splice(0..0, taken);
        }

        boxes[index] = boxes[index].merge(&taken_box);
      }
    }

    self.splits = splits.into_iter().flatten().collect();
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

  /// `xrPhase_Subdivide`: each split cut while it is too wide or holds too many faces, a chart's split giving each half
  /// a chart of its own.
  fn subdivide(
    mesh: &GeometryMesh,
    splits: &mut Vec<ChartedSplit>,
    charts: &mut Vec<Option<LightChart>>,
    scene: Option<&LightChartScene<'_>>,
  ) {
    let mut index: usize = 0;

    while index < splits.len() {
      let split: &[u32] = &splits[index].faces;

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

      let Some(halves) = Self::halve(mesh, split, &bounds, size) else {
        index += 1;
        continue;
      };
      let chart: Option<usize> = splits.remove(index).chart;

      for half in halves {
        let charted: Option<usize> = chart.zip(scene).map(|(chart, scene)| {
          charts[chart] = None;
          charts.push(Some(LightChart::of(half.clone(), scene)));
          charts.len() - 1
        });

        splits.push(ChartedSplit {
          faces: half,
          chart: charted,
        });
      }
    }
  }

  /// One split's cut into two halves of at least [`Self::LOW_FACES`] each, or none found: halved on the box's longest
  /// axis, each face by its centre, the cut moved by half the larger half's width towards it up to ten times while the
  /// split holds more than four times the least, then on the next axis, three in all.
  fn halve(mesh: &GeometryMesh, split: &[u32], bounds: &GeometrySplitBox, size: [f32; 3]) -> Option<[Vec<u32>; 2]> {
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

  /// `ValidateMerge`: the joined box's volume where two splits may join, none where they may not. They may where they
  /// hold at most four thirds of the high limit together, every axis of the joined box stays within four thirds of the
  /// size or a metre of either part's, and the box is at most eight times the two parts' cubes.
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
