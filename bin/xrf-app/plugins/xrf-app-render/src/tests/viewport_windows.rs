use xrf_renderer::{RenderHoldId, RenderViewportId};

use crate::viewport::viewport_windows::ViewportWindows;

#[test]
fn a_window_gives_up_only_its_own_viewports_in_attach_order() {
  let mut windows: ViewportWindows = ViewportWindows::default();

  windows.insert(RenderViewportId(3), "main");
  windows.insert(RenderViewportId(1), "main");
  windows.insert(RenderViewportId(2), "tool");

  assert_eq!(
    windows.take_window("main"),
    vec![RenderViewportId(1), RenderViewportId(3)]
  );
  assert_eq!(windows.take_window("main"), Vec::new());
  assert_eq!(windows.take_window("tool"), vec![RenderViewportId(2)]);
}

#[test]
fn a_viewport_detached_on_its_own_is_not_given_up_again() {
  let mut windows: ViewportWindows = ViewportWindows::default();

  windows.insert(RenderViewportId(1), "main");
  windows.insert(RenderViewportId(2), "main");

  assert_eq!(windows.remove(RenderViewportId(1)), Some("main".to_string()));
  assert_eq!(windows.remove(RenderViewportId(1)), None);
  assert_eq!(windows.take_window("main"), vec![RenderViewportId(2)]);
}

#[test]
fn the_exit_gives_up_every_viewport_and_hold_in_order() {
  let mut windows: ViewportWindows = ViewportWindows::default();

  windows.insert(RenderViewportId(2), "tool");
  windows.insert(RenderViewportId(1), "main");
  windows.insert_hold(RenderHoldId(1), "tool");

  assert_eq!(
    windows.take_all(),
    (vec![RenderViewportId(1), RenderViewportId(2)], vec![RenderHoldId(1)])
  );
  assert_eq!(windows.take_window("main"), Vec::new());
  assert_eq!(windows.take_window_holds("tool"), Vec::new());
}

#[test]
fn a_window_gives_up_only_its_own_holds_and_never_one_released() {
  let mut windows: ViewportWindows = ViewportWindows::default();

  windows.insert_hold(RenderHoldId(2), "main");
  windows.insert_hold(RenderHoldId(1), "main");
  windows.insert_hold(RenderHoldId(3), "tool");
  windows.insert(RenderViewportId(1), "main");

  assert_eq!(windows.remove_hold(RenderHoldId(2)), Some("main".to_string()));
  assert_eq!(windows.remove_hold(RenderHoldId(2)), None);
  assert_eq!(windows.take_window_holds("main"), vec![RenderHoldId(1)]);
  assert_eq!(
    windows.take_window("main"),
    vec![RenderViewportId(1)],
    "its viewports stay apart from its holds"
  );
  assert_eq!(windows.take_window_holds("tool"), vec![RenderHoldId(3)]);
}
