use std::time::Duration;

use xrf_game_platform::{PlatformEvent, PlatformEventKind, PlatformScancode};

use crate::input::developer_keys::DeveloperKeys;
use crate::layer::developer_frame::DeveloperFrame;
use crate::layer::developer_layer::DeveloperLayer;
use crate::layer::developer_view::DeveloperView;

const VIEW: DeveloperView = DeveloperView {
  width: 1280,
  height: 720,
  scale: 1.0,
  pixel_density: 1.0,
};

fn create_key(scancode: PlatformScancode, is_down: bool, is_repeat: bool) -> PlatformEvent {
  PlatformEvent {
    at: Duration::ZERO,
    kind: PlatformEventKind::Key {
      scancode,
      is_down,
      is_repeat,
    },
  }
}

fn create_move() -> PlatformEvent {
  PlatformEvent {
    at: Duration::ZERO,
    kind: PlatformEventKind::MouseMoved {
      x: 10.0,
      y: 20.0,
      dx: 1.0,
      dy: 1.0,
    },
  }
}

#[test]
fn console_key_toggles_on_press_only() {
  let mut layer: DeveloperLayer = DeveloperLayer::default();

  assert!(layer.take(&create_key(DeveloperKeys::CONSOLE, true, false), VIEW));
  assert!(layer.is_open());
  assert!(layer.take(&create_key(DeveloperKeys::CONSOLE, true, true), VIEW));
  assert!(layer.take(&create_key(DeveloperKeys::CONSOLE, false, false), VIEW));
  assert!(layer.is_open());
  assert!(layer.take(&create_key(DeveloperKeys::CONSOLE, true, false), VIEW));
  assert!(!layer.is_open());
}

#[test]
fn escape_closes_the_open_console_only() {
  let mut layer: DeveloperLayer = DeveloperLayer::default();

  assert!(!layer.take(&create_key(DeveloperKeys::CLOSE, true, false), VIEW));

  layer.take(&create_key(DeveloperKeys::CONSOLE, true, false), VIEW);

  assert!(layer.take(&create_key(DeveloperKeys::CLOSE, true, false), VIEW));
  assert!(!layer.is_open());
}

#[test]
fn modifier_held_before_opening_still_counts() {
  let mut layer: DeveloperLayer = DeveloperLayer::default();

  layer.take(&create_key(PlatformScancode(229), true, false), VIEW);
  layer.take(&create_key(DeveloperKeys::CONSOLE, true, false), VIEW);

  assert_eq!(layer.get_modifiers(), egui::Modifiers::SHIFT);

  layer.take(&create_key(PlatformScancode(229), false, false), VIEW);

  assert_eq!(layer.get_modifiers(), egui::Modifiers::NONE);
}

#[test]
fn console_drawer_opens_at_its_height_and_stays_there() {
  let mut layer: DeveloperLayer = DeveloperLayer::default();

  layer.take(&create_key(DeveloperKeys::CONSOLE, true, false), VIEW);

  // The drawer is all the layer draws without readouts: its lowest vertex is its bottom edge.
  let bottoms: Vec<f32> = (0..6)
    .map(|_| {
      layer
        .build(VIEW, &[], &[])
        .layer
        .meshes
        .iter()
        .flat_map(|mesh| mesh.vertices.iter().map(|vertex| vertex.position[1]))
        .fold(0.0, f32::max)
    })
    .collect();
  let expected: f32 = VIEW.height as f32 * 0.4;

  assert!(
    bottoms.iter().all(|bottom| (bottom - expected).abs() <= 2.0),
    "the drawer's bottom moved: {bottoms:?}, expected {expected}"
  );
}

#[test]
fn closed_layer_leaves_input_to_the_game() {
  let mut layer: DeveloperLayer = DeveloperLayer::default();

  assert!(!layer.take(&create_key(PlatformScancode(26), true, false), VIEW));
  assert!(!layer.take(&create_move(), VIEW));
}

#[test]
fn open_layer_keeps_input_from_the_game() {
  let mut layer: DeveloperLayer = DeveloperLayer::default();

  layer.take(&create_key(DeveloperKeys::CONSOLE, true, false), VIEW);

  assert!(layer.take(&create_key(PlatformScancode(26), true, false), VIEW));
  assert!(layer.take(&create_move(), VIEW));
}

#[test]
fn frames_send_the_font_atlas_first_then_meshes() {
  let mut layer: DeveloperLayer = DeveloperLayer::default();
  let readout: [String; 1] = [String::from("Tick 1")];
  let first: DeveloperFrame = layer.build(VIEW, &readout, &[]);

  assert!(!first.textures.set.is_empty());

  // egui sizes a new area on its first pass without drawing it.
  let frame: DeveloperFrame = layer.build(VIEW, &readout, &[]);

  assert!(!frame.layer.meshes.is_empty());
  assert!(
    frame
      .layer
      .meshes
      .iter()
      .all(|mesh| mesh.indices.iter().all(|index| (*index as usize) < mesh.vertices.len()))
  );
}
