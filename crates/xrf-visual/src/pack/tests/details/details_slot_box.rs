use crate::pack::details::details_slot_box::DetailsSlotBox;

/// World slot `(1, 2)`, from a metre up to three: `x` from 2 to 4, `z` from 4 to 6.
fn slot() -> DetailsSlotBox {
  DetailsSlotBox::of(1, 2, 1.0, 2.0)
}

#[test]
fn keeps_a_triangle_crossing_the_box() {
  assert!(slot().overlaps(&[[0.0, 2.0, 0.0], [10.0, 2.0, 0.0], [0.0, 2.0, 10.0]]));
}

#[test]
fn drops_a_triangle_above_or_beside_the_box() {
  assert!(!slot().overlaps(&[[0.0, 3.5, 0.0], [10.0, 3.5, 0.0], [0.0, 3.5, 10.0]]));
  assert!(!slot().overlaps(&[[5.0, 2.0, 0.0], [9.0, 2.0, 0.0], [5.0, 2.0, 9.0]]));
}

// Its bounds overlap the box, but the triangle only covers the corner of them opposite it: the edge axes separate it.
#[test]
fn drops_a_triangle_whose_bounds_alone_overlap_the_box() {
  assert!(!slot().overlaps(&[[3.9, 2.0, 7.0], [7.0, 2.0, 3.9], [7.0, 2.0, 7.0]]));
}

// The engine grows every slot box by `EPS_L`, so ground a hair past the edge is still under it.
#[test]
fn keeps_a_triangle_within_the_growth_of_the_box() {
  assert!(slot().overlaps(&[[4.0005, 2.0, 0.0], [9.0, 2.0, 0.0], [4.0005, 2.0, 9.0]]));
}
