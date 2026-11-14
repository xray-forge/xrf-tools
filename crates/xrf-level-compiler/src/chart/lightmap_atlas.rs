use std::cmp::Reverse;
use std::collections::HashMap;

use rayon::prelude::*;
use xrf_math::Vector2d;

use crate::chart::light_chart::LightChart;
use crate::chart::light_chart_mask::LightChartMask;
use crate::chart::light_chart_scene::LightChartScene;
use crate::chart::lightmap_faces::LightmapFaces;
use crate::chart::lightmap_layer::LightmapLayer;
use crate::chart::lightmap_page::LightmapPage;
use crate::chart::lightmap_placement::LightmapPlacement;
use crate::chart::lit_chart::LitChart;

/// A mask's spans in one orientation, and whether it is the crosswise one.
type MaskOrientation<'a> = (&'a [Option<(u32, u32)>], bool);

/// A level's charts packed onto lightmap pages, as `xrPhase_MergeLM` takes them and in its order, each placed by our
/// own packer ([`LightmapPage`]) where xrLC scans its page a texel at a time (`_rect_place`).
///
/// Page after page, the charts not yet placed are sorted, a stable sort: by the largest chart of their material, then
/// their material's sector, then the material, then their own area, each the larger first. The first of them that
/// together take no more than eight pages' texels are tried in turn on the new page, upright first and crosswise where
/// that fits nowhere; those that fit nowhere wait for the next page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightmapAtlas {
  /// How many pages the charts take.
  pub pages: u32,
  /// Each chart's place, in the charts' order.
  pub placements: Vec<LightmapPlacement>,
}

impl LightmapAtlas {
  /// The texels of the charts a page is tried with, eight pages' worth.
  const SELECTION: u32 = LightChart::PAGE_SIDE * LightChart::PAGE_SIDE * 8;

  /// The last of the sixteen border rings a page takes, from 254 down.
  const LAST_BORDER: u8 = 254 - 15;

  /// The pages `charts` take, lit as `lit` ([`LightChartMask`]), their materials' sectors as `scene` gives them.
  pub fn pack(charts: &[LightChart], lit: &[LitChart], scene: &LightChartScene<'_>) -> Self {
    let masks: Vec<LightChartMask> = lit.par_iter().map(|lit| LightChartMask::of(&lit.layer)).collect();
    let mut placements: Vec<Option<LightmapPlacement>> = vec![None; charts.len()];
    let mut waiting: Vec<usize> = (0..charts.len()).collect();
    let mut pages: u32 = 0;

    while !waiting.is_empty() {
      let mut largest: HashMap<u16, u32> = HashMap::new();

      for chart in &waiting {
        let area: &mut u32 = largest.entry(charts[*chart].material).or_default();

        *area = (*area).max(masks[*chart].get_area());
      }

      let key = |chart: usize| {
        let material: u16 = charts[chart].material;

        (
          largest[&material],
          scene.sectors.get(usize::from(material)).copied().unwrap_or(0),
          material,
          masks[chart].get_area(),
        )
      };

      waiting.sort_by_key(|chart| Reverse(key(*chart)));

      let mut taken: u32 = 0;
      let selected: usize = waiting
        .iter()
        .take_while(|chart| {
          taken += masks[**chart].get_area();
          taken <= Self::SELECTION
        })
        .count();
      let mut page: LightmapPage = LightmapPage::default();
      // Where a mask of the same spans last fitted on the page, or none where it fitted nowhere: the page only fills, so
      // the next of them fits nowhere before.
      let mut found_before: HashMap<MaskOrientation<'_>, Option<u32>> = HashMap::new();

      for chart in &waiting[..selected] {
        let mask: &LightChartMask = &masks[*chart];
        let found = [false, true].into_iter().find_map(|is_rotated| {
          let (width, height) = if is_rotated {
            (mask.height, mask.width)
          } else {
            (mask.width, mask.height)
          };
          let rows: &[Option<(u32, u32)>] = mask.get_rows(is_rotated);
          let from: Option<u32> = *found_before.entry((rows, is_rotated)).or_insert(Some(0));
          let place: Option<(u32, u32)> = from.and_then(|from| page.find(rows, width, from));

          found_before.insert((rows, is_rotated), place.map(|(_, y)| y));
          place.map(|(x, y)| LightmapPlacement {
            page: pages,
            x,
            y,
            width,
            height,
            is_rotated,
          })
        });

        if let Some(placement) = found {
          page.take(mask.get_rows(placement.is_rotated), placement.x, placement.y);
          placements[*chart] = Some(placement);
        }
      }

      waiting.retain(|chart| placements[*chart].is_none());
      pages += 1;
    }

    Self {
      pages,
      placements: placements.into_iter().flatten().collect(),
    }
  }

  /// The two textures of page `page` of the charts `lit`, its texels row by row, as `CLightmap::Capture` and `Save` make
  /// them: each chart placed on it takes its place with the texels it takes ([`LightmapLayer::TAKEN`]), crosswise
  /// where it lies so; those are then lit, the rest not, and sixteen border rings fill round them. `_1` holds the static
  /// light and the sun in alpha, `_2` the sun in every colour and the hemisphere in alpha.
  pub fn compose(&self, page: u32, lit: &[LitChart]) -> [Vec<[u8; 4]>; 2] {
    let side: u32 = LightChart::PAGE_SIDE;
    let mut target: LightmapLayer = LightmapLayer::new(side, side);

    for (placement, chart) in self
      .placements
      .iter()
      .zip(lit)
      .filter(|(placement, _)| placement.page == page)
    {
      let layer: &LightmapLayer = &chart.layer;

      for y in 0..layer.height {
        for x in 0..layer.width {
          let from: usize = layer.get_index(x, y);

          if layer.marker[from] < LightmapLayer::TAKEN {
            continue;
          }

          let to: usize = if placement.is_rotated {
            target.get_index(placement.x + y, placement.y + x)
          } else {
            target.get_index(placement.x + x, placement.y + y)
          };

          target.surface[to] = layer.surface[from];
          target.marker[to] = layer.marker[from];
        }
      }
    }

    for marker in &mut target.marker {
      *marker = if *marker >= LightmapLayer::TAKEN {
        LightmapLayer::LIT
      } else {
        0
      };
    }

    for reference in (Self::LAST_BORDER..=254).rev() {
      target.apply_borders(reference);
    }

    [target.get_packed(), target.get_packed_hemi()]
  }

  /// Each of `face_count` faces' page and lightmap coordinates where `charts` were placed, none for a face no chart
  /// holds.
  pub fn get_faces(&self, charts: &[LightChart], face_count: usize) -> LightmapFaces {
    let mut faces: LightmapFaces = LightmapFaces {
      pages: vec![None; face_count],
      coordinates: vec![[Vector2d::new(0.0, 0.0); 3]; face_count],
    };

    for (chart, placement) in charts.iter().zip(&self.placements) {
      for (face, coordinates) in chart.faces.iter().zip(chart.get_page_coordinates(placement)) {
        faces.pages[*face as usize] = Some(placement.page);
        faces.coordinates[*face as usize] = coordinates;
      }
    }

    faces
  }
}
