use std::path::PathBuf;
use std::sync::Arc;

use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsMipChain, DdsMipmaps, Quality};
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::plugins::textures::TextureSessionId;
use crate::plugins::textures::catalog::TextureCatalogMode;
use crate::plugins::textures::encoding::{TextureEncodingFormat, TextureEncodingSession};
use crate::plugins::textures::source::TextureSource;
use crate::plugins::textures::state::{TextureBrowseSession, TextureState};
use crate::plugins::textures::tests::fixtures::{BASE, source_image};

#[test]
fn replacing_a_comparison_refuses_old_tokens_even_for_the_same_reference() {
  let state: TextureState = TextureState::new();
  let first: TextureSessionId = state.begin_session().expect("first session");

  state
    .hold_comparison(comparison(first, "first"))
    .expect("first comparison");

  let second: TextureSessionId = state.begin_session().expect("second session");

  assert!(
    state.get_comparison(first).is_err(),
    "old candidates expire before the new encode finishes"
  );
  assert!(
    state.get_comparison(second).is_err(),
    "an unfinished comparison has no candidates"
  );
  state
    .hold_comparison(comparison(second, "second"))
    .expect("second comparison");

  assert!(
    state.get_comparison(first).is_err(),
    "matching labels and formats do not identify a comparison"
  );
  assert_eq!(
    state.get_comparison(second).expect("new comparison").roots,
    roots("second")
  );
}

#[test]
fn closing_releases_cached_bytes_and_refuses_unfinished_publication() {
  let state: TextureState = TextureState::new();
  let id: TextureSessionId = state.begin_session().expect("session");

  state.hold_comparison(comparison(id, "first")).expect("comparison");
  let held = Arc::downgrade(&state.get_comparison(id).expect("snapshot"));

  state.close().expect("close");

  assert!(held.upgrade().is_none(), "closing releases unreferenced encodes");
  assert!(state.get_comparison(id).is_err());
  assert!(state.hold_comparison(comparison(id, "first")).is_err());
  assert!(state.get_browse().expect("browse").is_none());
}

#[test]
fn a_late_comparison_cannot_replace_a_reopened_session() {
  let state: TextureState = TextureState::new();
  let first: TextureSessionId = state.begin_session().expect("first session");

  state.close().expect("close");
  let second: TextureSessionId = state.begin_session().expect("reopened session");

  state.open_browse(second, browse("second")).expect("reopen");
  assert!(state.hold_comparison(comparison(first, "first")).is_err());
  assert_eq!(state.get_browse().expect("browse"), Some(browse("second")));
}

#[test]
fn a_late_open_cannot_undo_close_or_a_newer_open() {
  let state: TextureState = TextureState::new();
  let first: TextureSessionId = state.begin_session().expect("pending open");

  state.close().expect("close");
  assert!(state.open_browse(first, browse("first")).is_err());

  let second: TextureSessionId = state.begin_session().expect("new open");
  state.open_browse(second, browse("second")).expect("new listing");

  assert!(state.open_browse(first, browse("first")).is_err());
  assert_eq!(state.get_browse().expect("browse"), Some(browse("second")));
}

#[test]
fn an_accepted_snapshot_outlives_close_without_locking_the_session() {
  let state: TextureState = TextureState::new();
  let id: TextureSessionId = state.begin_session().expect("session");

  state.hold_comparison(comparison(id, "first")).expect("comparison");
  let held: Arc<TextureEncodingSession> = state.get_comparison(id).expect("snapshot");
  let bytes: Vec<u8> = held
    .require(TextureEncodingFormat::Bc3)
    .expect("candidate")
    .write_to_bytes()
    .expect("encoded bytes");

  state.close().expect("close while snapshot is held");
  let next: TextureSessionId = state.begin_session().expect("next session");
  state
    .hold_comparison(comparison(next, "second"))
    .expect("new comparison");

  assert_eq!(held.session_id, id);
  assert_eq!(
    held
      .require(TextureEncodingFormat::Bc3)
      .expect("old candidate")
      .write_to_bytes()
      .expect("old bytes"),
    bytes
  );
  assert!(
    !held
      .require(TextureEncodingFormat::Bc3)
      .expect("candidate")
      .to_png()
      .expect("preview")
      .bytes
      .is_empty()
  );
}

fn roots(path: &str) -> XrayRoots {
  XrayRoots::one(PathBuf::from(path), XrayMountMode::Directory)
}

fn browse(path: &str) -> TextureBrowseSession {
  TextureBrowseSession {
    roots: roots(path),
    mode: TextureCatalogMode::Roots,
  }
}

fn comparison(session_id: TextureSessionId, path: &str) -> TextureEncodingSession {
  let chain: DdsMipChain = DdsMipChain::build(&source_image(16), DdsMipmaps::Disabled).expect("mip chain");

  TextureEncodingSession {
    session_id,
    roots: roots(path),
    source: TextureSource::Asset {
      reference: BASE.to_owned(),
    },
    label: BASE.to_owned(),
    attempts: vec![DdsEncodeAttempt::measure(&chain, DdsEncodeCandidate::Bc3, Quality::Fast).expect("encode")],
  }
}
