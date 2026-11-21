use std::cmp::Reverse;
use std::collections::HashMap;

use xrf_job::parallel::prelude::*;
use xrf_math::Vector2d;

use crate::light::lightmap::atlas::lightmap_faces::LightmapFaces;
use crate::light::lightmap::atlas::lightmap_image_pair::LightmapImagePair;
use crate::light::lightmap::atlas::lightmap_images::LightmapImages;
use crate::light::lightmap::atlas::lightmap_page::LightmapPage;
use crate::light::lightmap::atlas::lightmap_placement::LightmapPlacement;
use crate::light::lightmap::chart::light_chart::LightChart;
use crate::light::lightmap::chart::light_chart_mask::LightChartMask;
use crate::light::lightmap::chart::light_chart_scene::LightChartScene;
use crate::light::lightmap::chart::lightmap_orientation::LightmapOrientation;
use crate::light::lightmap::lit::lightmap_layer::LightmapLayer;
use crate::light::lightmap::lit::lit_chart::LitChart;

/// A mask's spans as it lies in one orientation, which the packer remembers where it last fitted by.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MaskOrientation<'a> {
  rows: &'a [Option<(u32, u32)>],
  orientation: LightmapOrientation,
}

/// A level's charts packed onto lightmap pages in `xrPhase_MergeLM`'s order, each placed by our own packer
/// ([`LightmapPage`]) where xrLC scans its page a texel at a time (`_rect_place`).
///
/// For each page, the unplaced charts are stably sorted by their material's largest chart, the material's sector, the
/// material, then their own area, larger first. The first of them within eight pages' texels together are tried in
/// turn, upright first, crosswise where upright fits nowhere; the rest wait for the next page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LightmapAtlas {
  /// How many pages the charts take.
  pub(crate) pages: u32,
  /// Each chart's place, in the charts' order.
  pub(crate) placements: Vec<LightmapPlacement>,
}

impl LightmapAtlas {
  /// The texels of the charts a page is tried with, eight pages' worth.
  const SELECTION: u32 = LightChart::PAGE_SIDE * LightChart::PAGE_SIDE * 8;

  /// The last of the sixteen border rings a page takes, from 254 down.
  const LAST_BORDER: u8 = 254 - 15;

  /// The pages `charts` take, lit as `lit` ([`LightChartMask`]), their materials' sectors as `scene` gives them.
  pub(crate) fn pack(charts: &[LightChart], lit: &[LitChart], scene: &LightChartScene<'_>) -> Self {
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
      // Where a mask of the same spans last fitted on the page, or none where it fitted nowhere: the page only fills,
      // so the next of them fits nowhere before.
      let mut found_before: HashMap<MaskOrientation<'_>, Option<u32>> = HashMap::new();

      for chart in &waiting[..selected] {
        let mask: &LightChartMask = &masks[*chart];
        let found = LightmapOrientation::ALL.into_iter().find_map(|orientation| {
          let (width, height) = match orientation {
            LightmapOrientation::Upright => (mask.width, mask.height),
            LightmapOrientation::Crosswise => (mask.height, mask.width),
          };
          let rows: &[Option<(u32, u32)>] = mask.get_rows(orientation);
          let key: MaskOrientation<'_> = MaskOrientation { rows, orientation };
          let from: Option<u32> = *found_before.entry(key).or_insert(Some(0));
          let place: Option<(u32, u32)> = from.and_then(|from| page.find(rows, width, from));

          found_before.insert(key, place.map(|(_, y)| y));
          place.map(|(x, y)| LightmapPlacement {
            page: pages,
            x,
            y,
            width,
            height,
            orientation,
          })
        });

        if let Some(placement) = found {
          page.take(mask.get_rows(placement.orientation), placement.x, placement.y);
          placements[*chart] = Some(placement);
        }
      }

      let unplaced: usize = waiting.len();

      waiting.retain(|chart| placements[*chart].is_none());
      // A chart's sides stay within `LightChart::MAXIMUM_SIDE`, so an empty page takes the first chart; a page
      // taking none would repeat forever.
      debug_assert!(waiting.len() < unplaced, "Page {pages} took none of {unplaced} charts");
      pages += 1;
    }

    Self {
      pages,
      placements: placements.into_iter().flatten().collect(),
    }
  }

  /// Every page's textures of the charts `lit`, [`LightChart::PAGE_SIDE`] texels a side ([`Self::compose`]).
  pub(crate) fn compose_all(&self, lit: &[LitChart]) -> LightmapImages {
    LightmapImages {
      side: LightChart::PAGE_SIDE,
      pages: (0..self.pages)
        .into_par_iter()
        .map(|page| self.compose(page, lit))
        .collect(),
    }
  }

  /// The two textures of page `page` of the charts `lit`, as `CLightmap::Capture` and `Save` make them: each chart
  /// placed there copies the texels it takes ([`LightmapLayer::TAKEN`]), crosswise where it lies so, and sixteen border
  /// rings fill round them.
  pub(crate) fn compose(&self, page: u32, lit: &[LitChart]) -> LightmapImagePair {
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

          let to: usize = match placement.orientation {
            LightmapOrientation::Upright => target.get_index(placement.x + x, placement.y + y),
            LightmapOrientation::Crosswise => target.get_index(placement.x + y, placement.y + x),
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

    LightmapImagePair {
      first: target.get_packed(),
      second: target.get_packed_hemi(),
    }
  }

  /// Each of `face_count` faces' page and lightmap coordinates where `charts` were placed, none for a face no chart
  /// holds.
  pub(crate) fn get_faces(&self, charts: &[LightChart], face_count: usize) -> LightmapFaces {
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
