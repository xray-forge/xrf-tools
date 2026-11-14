use crate::light::lightmap::chart::lightmap_orientation::LightmapOrientation;
use crate::light::lightmap::lit::lightmap_layer::LightmapLayer;

/// The texels a lit chart takes on a page, as xrLC places and blits it (`_rect_place`, `blit`): those of its layer
/// marked at least [`LightmapLayer::TAKEN`], its lit texels and the two rings round them. Kept as each row's span from
/// its first texel to its last, upright and crosswise, which the packer places by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LightChartMask {
  /// Its rectangle's texels a side, borders included.
  pub(crate) width: u32,
  pub(crate) height: u32,
  /// Each row's first and last texel taken, none for a row it leaves empty.
  pub(crate) rows: Vec<Option<(u32, u32)>>,
  /// Each column's, the rows of the mask laid crosswise.
  pub(crate) columns: Vec<Option<(u32, u32)>>,
}

impl LightChartMask {
  /// The mask of a lit chart's bordered `layer`.
  pub(crate) fn of(layer: &LightmapLayer) -> Self {
    let (width, height) = (layer.width, layer.height);
    let is_taken = |x: u32, y: u32| layer.marker[layer.get_index(x, y)] >= LightmapLayer::TAKEN;
    let span = |taken: &mut dyn Iterator<Item = (u32, bool)>| {
      taken
        .filter(|(_, is_taken)| *is_taken)
        .fold(None, |span: Option<(u32, u32)>, (at, _)| {
          Some(span.map_or((at, at), |(first, _)| (first, at)))
        })
    };

    Self {
      width,
      height,
      rows: (0..height)
        .map(|y| span(&mut (0..width).map(|x| (x, is_taken(x, y)))))
        .collect(),
      columns: (0..width)
        .map(|x| span(&mut (0..height).map(|y| (y, is_taken(x, y)))))
        .collect(),
    }
  }

  /// The texels of the rectangle it takes, `Area`.
  pub(crate) fn get_area(&self) -> u32 {
    self.width * self.height
  }

  /// Each row's span as the mask lies in `orientation`.
  pub(crate) fn get_rows(&self, orientation: LightmapOrientation) -> &[Option<(u32, u32)>] {
    match orientation {
      LightmapOrientation::Upright => &self.rows,
      LightmapOrientation::Crosswise => &self.columns,
    }
  }
}
