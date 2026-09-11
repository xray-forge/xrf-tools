use tauri::http::{Request, StatusCode};

use crate::core::session::DocumentSessionId;
use crate::plugins::sprite_equipment::document::read_sprite;
use crate::plugins::sprite_equipment::state::{EquipmentSpriteDocument, EquipmentSpriteMetadata, EquipmentSpriteState};
use crate::plugins::sprite_equipment::stream::sprite_response;

fn document(bytes: Vec<u8>) -> EquipmentSpriteDocument {
  EquipmentSpriteDocument {
    metadata: EquipmentSpriteMetadata {
      path: String::from("equipment.dds"),
      name: String::from("equipment.png"),
      system_ltx_path: String::from("system.ltx"),
      is_dltx: false,
      equipment_descriptors: Vec::new(),
    },
    preview: bytes,
  }
}

#[test]
fn a_stream_url_cannot_read_a_replacement_with_the_same_name() {
  let state: EquipmentSpriteState = EquipmentSpriteState::new("sprite test");
  let first: DocumentSessionId = DocumentSessionId::new();
  let second: DocumentSessionId = DocumentSessionId::new();

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
  let first: DocumentSessionId = DocumentSessionId::new();

  state.begin_open(first).unwrap();
  state.commit_open(first, document(vec![1, 2, 3])).unwrap();
  state.begin_reload(DocumentSessionId::new(), first).unwrap();

  assert!(read_sprite("", "", false).is_err());

  let opened = state.require(first).unwrap();

  assert_eq!(opened.preview, [1, 2, 3]);
  assert_eq!(opened.metadata.system_ltx_path, "system.ltx");
}
