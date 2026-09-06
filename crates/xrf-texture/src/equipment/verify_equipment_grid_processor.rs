use std::collections::HashMap;

use xrf_ltx::Ltx;

use crate::equipment::InventorySpriteDescriptor;

/// Two sections whose inventory icon rectangles cover a common cell.
#[derive(Clone, Debug, PartialEq)]
pub struct EquipmentGridOverlap {
  pub first: String,
  pub second: String,
  /// One cell both rectangles cover, enough to locate the clash on the sheet.
  pub cell: (u32, u32),
  pub overlapping_cells: u32,
}

/// Finds inventory icon rectangles that overlap without being identical.
///
/// Identical rectangles are not reported. Sharing a slot is legitimate and expected for `_nimble`,
/// `_snag` and the `pri_a15_` quest copies.
pub struct VerifyEquipmentGridProcessor {}

impl VerifyEquipmentGridProcessor {
  pub fn find_overlaps(ltx: &Ltx) -> Vec<EquipmentGridOverlap> {
    let descriptors: Vec<InventorySpriteDescriptor> = InventorySpriteDescriptor::new_list_from_ltx(ltx);

    // Cell -> the rectangles covering it. Rectangles are small and the sheet is sparse, so mapping
    // cells is cheaper and clearer than comparing every pair of rectangles.
    let mut cells: HashMap<(u32, u32), Vec<&InventorySpriteDescriptor>> = HashMap::new();

    for descriptor in &descriptors {
      for y in descriptor.y..descriptor.y.saturating_add(descriptor.h.max(1)) {
        for x in descriptor.x..descriptor.x.saturating_add(descriptor.w.max(1)) {
          cells.entry((x, y)).or_default().push(descriptor);
        }
      }
    }

    let mut counted: HashMap<(String, String), (u32, (u32, u32))> = HashMap::new();

    for (cell, occupants) in cells {
      for (index, first) in occupants.iter().enumerate() {
        for second in occupants.iter().skip(index + 1) {
          if Self::is_same_rect(first, second) {
            continue;
          }

          let key: (String, String) = if first.section <= second.section {
            (first.section.clone(), second.section.clone())
          } else {
            (second.section.clone(), first.section.clone())
          };

          let entry: &mut (u32, (u32, u32)) = counted.entry(key).or_insert((0, cell));

          entry.0 += 1;
          // Report the top left of the clash so it is stable regardless of iteration order.
          if cell.1 < entry.1.1 || (cell.1 == entry.1.1 && cell.0 < entry.1.0) {
            entry.1 = cell;
          }
        }
      }
    }

    let mut overlaps: Vec<EquipmentGridOverlap> = counted
      .into_iter()
      .map(|((first, second), (overlapping_cells, cell))| EquipmentGridOverlap {
        first,
        second,
        cell,
        overlapping_cells,
      })
      .collect();

    overlaps.sort_by(|a, b| (a.cell.1, a.cell.0, &a.first).cmp(&(b.cell.1, b.cell.0, &b.first)));

    overlaps
  }

  fn is_same_rect(first: &InventorySpriteDescriptor, second: &InventorySpriteDescriptor) -> bool {
    first.x == second.x && first.y == second.y && first.w == second.w && first.h == second.h
  }
}

#[cfg(test)]
mod tests {
  use xrf_ltx::Ltx;

  use super::VerifyEquipmentGridProcessor;

  fn ltx_of(entries: &[(&str, u32, u32, u32, u32)]) -> Ltx {
    let mut ltx: Ltx = Ltx::new();

    for (name, x, y, w, h) in entries {
      let mut section = ltx.with_section(*name);

      section
        .set("$inventory_icon", "true")
        .set("inv_grid_x", x.to_string())
        .set("inv_grid_y", y.to_string())
        .set("inv_grid_width", w.to_string())
        .set("inv_grid_height", h.to_string());
    }

    ltx
  }

  #[test]
  fn reports_a_rect_reaching_into_a_neighbour() {
    // The real case this exists for: a 4 wide rect spanning a 2 wide one beside it.
    let ltx: Ltx = ltx_of(&[("wpn_rpg7_missile", 15, 5, 4, 1), ("wpn_colt1911", 16, 5, 2, 1)]);
    let overlaps = VerifyEquipmentGridProcessor::find_overlaps(&ltx);

    assert_eq!(overlaps.len(), 1);
    assert_eq!(overlaps[0].first, "wpn_colt1911");
    assert_eq!(overlaps[0].second, "wpn_rpg7_missile");
    assert_eq!(overlaps[0].cell, (16, 5));
    assert_eq!(overlaps[0].overlapping_cells, 2);
  }

  #[test]
  fn ignores_identical_rects_because_variants_share_slots_by_design() {
    let ltx: Ltx = ltx_of(&[("wpn_pm", 18, 0, 2, 1), ("wpn_pm_actor", 18, 0, 2, 1)]);

    assert!(VerifyEquipmentGridProcessor::find_overlaps(&ltx).is_empty());
  }

  #[test]
  fn ignores_rects_that_only_touch() {
    let ltx: Ltx = ltx_of(&[("wpn_pm", 18, 0, 2, 1), ("wpn_fort", 20, 0, 1, 1)]);

    assert!(VerifyEquipmentGridProcessor::find_overlaps(&ltx).is_empty());
  }

  #[test]
  fn detects_vertical_overlap() {
    let ltx: Ltx = ltx_of(&[("wpn_svd", 20, 0, 6, 2), ("wpn_something", 21, 1, 2, 1)]);
    let overlaps = VerifyEquipmentGridProcessor::find_overlaps(&ltx);

    assert_eq!(overlaps.len(), 1);
    assert_eq!(overlaps[0].cell, (21, 1));
    assert_eq!(overlaps[0].overlapping_cells, 2);
  }
}
