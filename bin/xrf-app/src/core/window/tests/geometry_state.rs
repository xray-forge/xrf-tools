use crate::core::window::tests::fixtures::geometry;
use crate::core::window::window_geometry::WindowGeometry;
use crate::core::window::window_geometry_state::WindowGeometryState;

#[test]
fn only_the_first_record_of_a_drag_waits_for_it_to_settle() {
  let mut state: WindowGeometryState = WindowGeometryState::new(geometry(100, 80, 1280, 800));

  state.record(geometry(101, 80, 1280, 800));
  assert!(state.begin_settling());

  state.record(geometry(102, 80, 1280, 800));
  assert!(!state.begin_settling());
}

#[test]
fn a_move_after_one_settled_waits_again() {
  let mut state: WindowGeometryState = WindowGeometryState::new(geometry(100, 80, 1280, 800));

  state.record(geometry(101, 80, 1280, 800));
  state.begin_settling();
  state.take_unwritten();

  state.record(geometry(200, 80, 1280, 800));

  assert!(state.begin_settling());
}

/// Only where a drag comes to rest is worth a preference, so what settles is the last position and not the first.
#[test]
fn writes_where_a_drag_came_to_rest_and_not_where_it_passed() {
  let mut state: WindowGeometryState = WindowGeometryState::new(geometry(100, 80, 1280, 800));

  state.record(geometry(101, 80, 1280, 800));
  state.begin_settling();
  state.record(geometry(400, 300, 1280, 800));

  assert_eq!(state.take_unwritten(), Some(geometry(400, 300, 1280, 800)));
}

/// A first run that never moved must not overwrite the rectangle a previous one left.
#[test]
fn never_writes_a_window_nobody_measured() {
  let mut state: WindowGeometryState = WindowGeometryState::new(WindowGeometry::default());

  state.record(WindowGeometry::default());

  assert_eq!(state.take_unwritten(), None);
}

#[test]
fn does_not_write_the_same_rectangle_twice() {
  let mut state: WindowGeometryState = WindowGeometryState::new(geometry(100, 80, 1280, 800));

  state.record(geometry(400, 300, 1280, 800));
  assert!(state.take_unwritten().is_some());

  state.record(geometry(400, 300, 1280, 800));
  assert_eq!(state.take_unwritten(), None);
}

#[test]
fn owes_one_save_per_write_and_nothing_when_nothing_was_written() {
  let mut state: WindowGeometryState = WindowGeometryState::new(geometry(100, 80, 1280, 800));

  assert!(!state.take_unsaved());

  state.record(geometry(400, 300, 1280, 800));
  state.take_unwritten();

  assert!(state.take_unsaved());
  assert!(!state.take_unsaved());
}
