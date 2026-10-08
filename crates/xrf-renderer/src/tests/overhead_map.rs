use crate::scene::level::overhead_map::OverheadMap;

#[test]
fn draws_the_columns_a_map_moved_into() {
  assert_eq!(
    OverheadMap::get_strips((64, 0), 2048),
    [Some([1984, 0, 64, 2048]), None]
  );
  assert_eq!(OverheadMap::get_strips((-64, 0), 2048), [Some([0, 0, 64, 2048]), None]);
}

#[test]
fn draws_the_rows_a_map_moved_into() {
  assert_eq!(
    OverheadMap::get_strips((0, 128), 2048),
    [None, Some([0, 1920, 2048, 128])]
  );
  assert_eq!(OverheadMap::get_strips((0, -64), 2048), [None, Some([0, 0, 2048, 64])]);
}

#[test]
fn draws_the_rows_beside_the_columns_once() {
  // Moved both ways, the rows leave out the columns already drawn, so no texel is drawn twice.
  assert_eq!(
    OverheadMap::get_strips((-64, 128), 2048),
    [Some([0, 0, 64, 2048]), Some([64, 1920, 1984, 128])]
  );
  assert_eq!(
    OverheadMap::get_strips((64, -64), 2048),
    [Some([1984, 0, 64, 2048]), Some([0, 0, 1984, 64])]
  );
}

#[test]
fn draws_nothing_where_a_map_did_not_move() {
  assert_eq!(OverheadMap::get_strips((0, 0), 2048), [None, None]);
}
