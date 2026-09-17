use crate::core::window::monitor_work_area::MonitorWorkArea;
use crate::core::window::tests::fixtures::{LEFT, MIN_HEIGHT, MIN_WIDTH, PRIMARY, geometry};
use crate::core::window::window_geometry::WindowGeometry;
use crate::core::window::window_geometry_fit::fit_window_geometry;

#[test]
fn refuses_a_rectangle_no_run_ever_measured() {
  let saved: WindowGeometry = WindowGeometry::default();

  assert_eq!(fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT), None);
}

#[test]
fn refuses_a_rectangle_whose_scale_factor_is_nonsense() {
  let saved: WindowGeometry = WindowGeometry {
    scale: 0.0,
    ..geometry(100, 100, 1280, 800)
  };

  assert_eq!(fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT), None);
}

#[test]
fn keeps_a_rectangle_that_is_already_inside_a_work_area() {
  let saved: WindowGeometry = geometry(100, 80, 1280, 800);

  assert_eq!(
    fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT),
    Some(saved)
  );
}

#[test]
fn pulls_a_window_back_from_beyond_the_right_edge() {
  let saved: WindowGeometry = geometry(1800, 80, 1280, 800);

  assert_eq!(
    fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT),
    Some(geometry(640, 80, 1280, 800))
  );
}

#[test]
fn pulls_a_window_back_from_under_the_taskbar() {
  let saved: WindowGeometry = geometry(100, 1000, 1280, 800);

  assert_eq!(
    fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT),
    Some(geometry(100, 240, 1280, 800))
  );
}

#[test]
fn forgets_a_position_on_a_monitor_that_is_gone() {
  let saved: WindowGeometry = geometry(LEFT.x, 80, 1280, 800);

  assert_eq!(fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT), None);
}

/// A window left across the seam between two monitors comes back whole onto the one that held most of it, which is the
/// cost of promising that the title bar is always reachable.
#[test]
fn gathers_a_window_that_straddled_two_monitors_onto_one() {
  let saved: WindowGeometry = geometry(-1000, 80, 1280, 800);

  assert_eq!(
    fit_window_geometry(saved, &[PRIMARY, LEFT], MIN_WIDTH, MIN_HEIGHT),
    Some(geometry(LEFT.x, 80, 1280, 800))
  );
}

#[test]
fn rescales_a_rectangle_measured_where_the_monitor_was_denser() {
  let dense: MonitorWorkArea = MonitorWorkArea { scale: 2.0, ..PRIMARY };
  let saved: WindowGeometry = geometry(100, 80, 1280, 800);

  let fitted: WindowGeometry = fit_window_geometry(saved, &[dense], MIN_WIDTH, MIN_HEIGHT).expect("A fitted rectangle");

  assert_eq!((fitted.width, fitted.height), (1920, 1040));
  assert_eq!(fitted.scale, 2.0);
}

#[test]
fn does_not_open_smaller_than_the_window_configuration_allows() {
  let saved: WindowGeometry = geometry(100, 80, 320, 240);

  let fitted: WindowGeometry =
    fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT).expect("A fitted rectangle");

  assert_eq!((fitted.width, fitted.height), (900, 600));
}

#[test]
fn does_not_open_larger_than_the_work_area() {
  let saved: WindowGeometry = geometry(0, 0, 3840, 2160);

  let fitted: WindowGeometry =
    fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT).expect("A fitted rectangle");

  assert_eq!((fitted.x, fitted.y, fitted.width, fitted.height), (0, 0, 1920, 1040));
}

#[test]
fn keeps_a_window_that_was_closed_maximized_maximized() {
  let saved: WindowGeometry = WindowGeometry {
    maximized: true,
    ..geometry(100, 80, 1280, 800)
  };

  let fitted: WindowGeometry =
    fit_window_geometry(saved, &[PRIMARY], MIN_WIDTH, MIN_HEIGHT).expect("A fitted rectangle");

  assert!(fitted.maximized);
  assert_eq!((fitted.x, fitted.y, fitted.width, fitted.height), (100, 80, 1280, 800));
}
