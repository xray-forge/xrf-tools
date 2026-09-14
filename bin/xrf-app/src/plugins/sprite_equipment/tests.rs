use tauri::http::{Request, StatusCode};

use xrf_vfs::XrayRoots;

use crate::core::assets::AssetMountState;
use crate::core::session::SessionId;
use crate::plugins::sprite_equipment::document::EquipmentSpriteDocument;
use crate::plugins::sprite_equipment::location::EquipmentSheetLocation;
use crate::plugins::sprite_equipment::metadata::EquipmentSpriteMetadata;
use crate::plugins::sprite_equipment::source::{EquipmentSheetSource, EquipmentSpriteOpen};
use crate::plugins::sprite_equipment::state::EquipmentSpriteState;
use crate::plugins::sprite_equipment::stream::sprite_response;

/// An open naming one loose sheet and no configuration.
fn open_of(path: &str) -> EquipmentSpriteOpen {
  EquipmentSpriteOpen {
    roots: XrayRoots::default(),
    sheet: EquipmentSheetSource::File { path: path.into() },
    config: None,
    is_dltx: false,
  }
}

fn document(bytes: Vec<u8>) -> EquipmentSpriteDocument {
  EquipmentSpriteDocument {
    metadata: EquipmentSpriteMetadata {
      name: String::from("equipment.png"),
      open: open_of("equipment.dds"),
      location: EquipmentSheetLocation {
        asset: None,
        path: Some(String::from("equipment.dds")),
        write_target: Some(String::from("equipment.dds")),
      },
      config_error: None,
      occupants: Vec::new(),
    },
    preview: bytes,
  }
}

#[test]
fn a_stream_url_cannot_read_a_replacement_with_the_same_name() {
  let state: EquipmentSpriteState = EquipmentSpriteState::new("sprite test");
  let first: SessionId = SessionId::new();
  let second: SessionId = SessionId::new();

  state.begin_open(first).unwrap();
  state.commit_open(first, document(vec![1])).unwrap();

  let request = Request::builder()
    .uri(format!("/{first}/equipment.png"))
    .body(Vec::new())
    .unwrap();

  assert_eq!(sprite_response(&state, &request).unwrap().body(), &[1]);

  state.begin_open(second).unwrap();
  state.commit_open(second, document(vec![2])).unwrap();
  state.close(&[first]).unwrap();

  assert_eq!(
    sprite_response(&state, &request).unwrap().status(),
    StatusCode::NOT_FOUND
  );

  let request = Request::builder()
    .uri(format!("/{second}/equipment.png"))
    .body(Vec::new())
    .unwrap();

  assert_eq!(sprite_response(&state, &request).unwrap().body(), &[2]);

  state.close(&[second]).unwrap();

  assert_eq!(
    sprite_response(&state, &request).unwrap().status(),
    StatusCode::NOT_FOUND
  );
}

#[test]
fn a_failed_reload_preserves_the_previous_metadata_and_preview() {
  let state: EquipmentSpriteState = EquipmentSpriteState::new("sprite test");
  let first: SessionId = SessionId::new();

  state.begin_open(first).unwrap();
  state.commit_open(first, document(vec![1, 2, 3])).unwrap();
  state.begin_reload(SessionId::new(), first).unwrap();

  // A sheet that cannot be decoded is the ordinary way a reload fails, and it must leave the open one alone rather
  // than half-replacing it.
  assert!(EquipmentSpriteDocument::read(&AssetMountState::new(), open_of("nothing-here.dds")).is_err());

  let opened = state.require(first).unwrap();

  assert_eq!(opened.preview, [1, 2, 3]);
  assert_eq!(opened.metadata.location.path.as_deref(), Some("equipment.dds"));
}
