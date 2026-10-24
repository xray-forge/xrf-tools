use bytes::Bytes;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionScope, SessionScopes};
use xrf_vfs::XrayRoots;

use crate::{
  EquipmentSheetLocation, EquipmentSheetSource, EquipmentSpriteDocument, EquipmentSpriteMetadata, EquipmentSpriteOpen,
  EquipmentSprites,
};

/// An open naming one loose sheet and no configuration.
fn open_of(path: &str) -> EquipmentSpriteOpen {
  EquipmentSpriteOpen {
    roots: XrayRoots::default(),
    sheet: EquipmentSheetSource::File { path: path.into() },
    config: None,
    is_dltx: false,
  }
}

fn document(bytes: &'static [u8]) -> EquipmentSpriteDocument {
  EquipmentSpriteDocument {
    metadata: EquipmentSpriteMetadata {
      open: open_of("equipment.dds"),
      location: EquipmentSheetLocation::of_file("equipment.dds"),
      config_error: None,
      occupants: Vec::new(),
    },
    preview: Bytes::from_static(bytes),
  }
}

#[test]
fn a_preview_is_read_from_the_handle_that_holds_it_and_none_closed() {
  let sprites: EquipmentSprites = EquipmentSprites::new("sprite test", &SessionScopes::default());
  let (first, second): (SessionId, SessionId) = (SessionId::generate(), SessionId::generate());

  sprites
    .begin_fixture_open(first)
    .unwrap()
    .commit(document(&[1]))
    .unwrap();
  sprites
    .begin_fixture_open(second)
    .unwrap()
    .commit(document(&[2]))
    .unwrap();

  assert_eq!(sprites.require(first).unwrap().preview, Bytes::from_static(&[1]));
  assert_eq!(sprites.require(second).unwrap().preview, Bytes::from_static(&[2]));

  sprites.close(&[first]);

  assert!(sprites.require(first).is_err(), "a closed sheet serves no picture");
  assert_eq!(sprites.require(second).unwrap().preview, Bytes::from_static(&[2]));
}

#[test]
fn a_failed_reload_leaves_the_previous_sheet_and_binds_no_handle() {
  let scopes: SessionScopes = SessionScopes::default();
  let sprites: EquipmentSprites = EquipmentSprites::new("sprite test", &scopes);
  let tool: SessionScope = SessionScope::new("main", "sprite-equipment-editor");
  let (first, reload): (SessionId, SessionId) = (SessionId::generate(), SessionId::generate());

  scopes.bind(first, tool.clone());
  sprites.begin_open(first).unwrap().commit(document(&[1, 2, 3])).unwrap();
  scopes.bind(reload, tool.clone());

  let (opening, previous) = sprites.begin_reload(reload, first).unwrap();

  // A sheet that cannot be decoded is the ordinary way a reload fails, and it must leave the open one alone rather
  // than half-replacing it.
  assert!(EquipmentSpriteDocument::read(&AssetMountState::new(), open_of("nothing-here.dds")).is_err());
  assert_eq!(previous.metadata.open, open_of("equipment.dds"));

  drop(opening);

  let opened = sprites.require(first).unwrap();

  assert_eq!(opened.preview, Bytes::from_static(&[1, 2, 3]));
  assert_eq!(opened.metadata.location.path.as_deref(), Some("equipment.dds"));
  assert!(sprites.require(reload).is_err());
  assert_eq!(scopes.list(&tool), vec![first], "the given-up reload is unbound");
}
