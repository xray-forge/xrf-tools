use xrf_renderer::RenderViewportId;

use crate::plugins::render::viewport_windows::ViewportWindows;

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
