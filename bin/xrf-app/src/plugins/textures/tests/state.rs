use std::path::PathBuf;
use std::sync::Arc;

use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsMipChain, DdsMipmaps, Quality};
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::core::session::{SessionId, SessionSnapshot};
use crate::plugins::textures::catalog::TextureCatalogMode;
use crate::plugins::textures::encoding::{TextureEncodingFormat, TextureEncodingSession};
use crate::plugins::textures::source::TextureSource;
use crate::plugins::textures::state::{TextureBrowseSession, TextureState};
use crate::plugins::textures::tests::fixtures::{BASE, source_image};

#[test]
fn replacing_a_comparison_refuses_old_tokens_even_for_the_same_reference() {
  let state: TextureState = TextureState::new();
  let first: SessionId = SessionId::new();

  state.comparison.begin_open(first).expect("first session");

  state
    .comparison
    .commit_open(first, comparison(first, "first"))
    .expect("first comparison");

  let second: SessionId = SessionId::new();

  state.comparison.begin_open(second).expect("second session");

  assert!(
    state.comparison.require(first).is_ok(),
    "a failed replacement must leave the previous candidates readable"
  );
  assert!(
    state.comparison.require(second).is_err(),
    "an unfinished comparison has no candidates"
  );
  state
    .comparison
    .commit_open(second, comparison(second, "second"))
    .expect("second comparison");

  assert!(
    state.comparison.require(first).is_err(),
    "matching labels and formats do not identify a comparison"
  );
  assert_eq!(
    state.comparison.require(second).expect("new comparison").roots,
    roots("second")
  );
}

#[test]
fn closing_releases_cached_bytes_and_refuses_unfinished_publication() {
  let state: TextureState = TextureState::new();
  let id: SessionId = SessionId::new();

  state.comparison.begin_open(id).expect("session");

  state
    .comparison
    .commit_open(id, comparison(id, "first"))
    .expect("comparison");
  let held = Arc::downgrade(&state.comparison.require(id).expect("snapshot"));

  state.close(&[id]).expect("close");

  assert!(held.upgrade().is_none(), "closing releases unreferenced encodes");
  assert!(state.comparison.require(id).is_err());
  assert!(state.comparison.commit_open(id, comparison(id, "first")).is_err());
  assert!(state.browse.get().expect("browse").is_none());
}

#[test]
fn a_late_comparison_cannot_replace_a_reopened_session() {
  let state: TextureState = TextureState::new();
  let first: SessionId = SessionId::new();

  state.comparison.begin_open(first).expect("first session");

  state.close(&[first]).expect("close");

  let second: SessionId = SessionId::new();

  state.browse.begin_open(second).expect("reopened session");
  state.browse.commit_open(second, browse("second")).expect("reopen");

  assert!(state.comparison.commit_open(first, comparison(first, "first")).is_err());
  assert_eq!(
    state.browse.get().expect("browse").expect("opened").value,
    browse("second")
  );
}

#[test]
fn a_late_open_cannot_undo_close_or_a_newer_open() {
  let state: TextureState = TextureState::new();
  let first: SessionId = SessionId::new();

  state.browse.begin_open(first).expect("pending open");
  state.close(&[first]).expect("close");

  assert!(state.browse.commit_open(first, browse("first")).is_err());

  let second: SessionId = SessionId::new();

  state.browse.begin_open(second).expect("new open");
  state.browse.commit_open(second, browse("second")).expect("new listing");

  assert!(state.browse.commit_open(first, browse("first")).is_err());
  assert_eq!(
    state.browse.get().expect("browse").expect("opened").value,
    browse("second")
  );
}

#[test]
fn an_accepted_snapshot_outlives_close_without_locking_the_session() {
  let state: TextureState = TextureState::new();
  let id: SessionId = SessionId::new();

  state.comparison.begin_open(id).expect("session");
  state
    .comparison
    .commit_open(id, comparison(id, "first"))
    .expect("comparison");

  let held: Arc<SessionSnapshot<TextureEncodingSession>> = state.comparison.require(id).expect("snapshot");
  let bytes: Vec<u8> = held
    .require(TextureEncodingFormat::Bc3)
    .expect("candidate")
    .write_to_bytes()
    .expect("encoded bytes");

  state.close(&[id]).expect("close while snapshot is held");

  let next: SessionId = SessionId::new();

  state.comparison.begin_open(next).expect("next session");
  state
    .comparison
    .commit_open(next, comparison(next, "second"))
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

fn comparison(session_id: SessionId, path: &str) -> TextureEncodingSession {
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
