use std::path::PathBuf;
use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionOpening, SessionScopes, SessionSnapshot};
use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsMipChain, DdsMipmaps, Quality};
use xrf_texture::TextureEncodingFormat;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::catalog::TextureCatalogMode;
use crate::encoding::TextureEncodingSession;
use crate::source::TextureSource;
use crate::state::{TextureBrowseSession, TextureState};
use crate::tests::fixtures::{BASE, source_image};

#[test]
fn a_comparison_is_known_by_its_own_token_until_its_tool_closes_it() {
  let state: TextureState = TextureState::new(AssetMountState::new(), &SessionScopes::default());
  let first: SessionId = SessionId::generate();

  state
    .comparisons
    .begin_fixture_open(first)
    .expect("first session")
    .commit(comparison(first, "first"))
    .expect("first comparison");

  let second: SessionId = SessionId::generate();
  let opening: SessionOpening<TextureEncodingSession> =
    state.comparisons.begin_fixture_open(second).expect("second session");

  assert!(
    state.comparisons.require(first).is_ok(),
    "an unfinished replacement leaves the previous candidates readable"
  );
  assert!(
    state.comparisons.require(second).is_err(),
    "an unfinished comparison has no candidates"
  );
  opening.commit(comparison(second, "second")).expect("second comparison");
  state.close(&[first]).expect("the tool lets the first go");

  assert!(
    state.comparisons.require(first).is_err(),
    "matching labels and formats do not identify a comparison"
  );
  assert_eq!(
    state.comparisons.require(second).expect("new comparison").roots,
    roots("second")
  );
}

#[test]
fn closing_releases_cached_bytes_and_refuses_unfinished_publication() {
  let state: TextureState = TextureState::new(AssetMountState::new(), &SessionScopes::default());
  let id: SessionId = SessionId::generate();

  state
    .comparisons
    .begin_fixture_open(id)
    .expect("session")
    .commit(comparison(id, "first"))
    .expect("comparison");

  let held = Arc::downgrade(&state.comparisons.require(id).expect("snapshot"));
  let unfinished: SessionId = SessionId::generate();
  let late: SessionOpening<TextureEncodingSession> = state
    .comparisons
    .begin_fixture_open(unfinished)
    .expect("unfinished session");

  // Scopes made without a pool drop what a close lets go at once.
  state.close(&[id, unfinished]).expect("close");

  assert!(held.upgrade().is_none(), "closing releases unreferenced encodes");
  assert!(state.comparisons.require(id).is_err());
  assert!(
    late.commit(comparison(unfinished, "late")).is_err(),
    "an unfinished comparison publishes nothing"
  );
  assert!(
    state.comparisons.begin_open(id).is_err(),
    "a closed comparison opens no more"
  );
}

#[test]
fn a_late_comparison_cannot_replace_a_reopened_session() {
  let state: TextureState = TextureState::new(AssetMountState::new(), &SessionScopes::default());
  let first: SessionId = SessionId::generate();
  let late: SessionOpening<TextureEncodingSession> =
    state.comparisons.begin_fixture_open(first).expect("first session");

  state.close(&[first]).expect("close");

  let second: SessionId = SessionId::generate();

  state
    .browses
    .begin_fixture_open(second)
    .expect("reopened session")
    .commit(browse("second"))
    .expect("reopen");

  assert!(late.commit(comparison(first, "first")).is_err());
  assert_eq!(*state.browses.require(second).expect("opened").value, browse("second"));
}

#[test]
fn a_late_open_cannot_undo_close() {
  let state: TextureState = TextureState::new(AssetMountState::new(), &SessionScopes::default());
  let first: SessionId = SessionId::generate();
  let late: SessionOpening<TextureBrowseSession> = state.browses.begin_fixture_open(first).expect("pending open");

  state.close(&[first]).expect("close");

  let second: SessionId = SessionId::generate();

  state
    .browses
    .begin_fixture_open(second)
    .expect("new open")
    .commit(browse("second"))
    .expect("new listing");

  assert!(late.commit(browse("first")).is_err());
  assert_eq!(*state.browses.require(second).expect("opened").value, browse("second"));
}

#[test]
fn an_accepted_snapshot_outlives_close_without_locking_the_session() {
  let state: TextureState = TextureState::new(AssetMountState::new(), &SessionScopes::default());
  let id: SessionId = SessionId::generate();

  state
    .comparisons
    .begin_fixture_open(id)
    .expect("session")
    .commit(comparison(id, "first"))
    .expect("comparison");

  let held: Arc<SessionSnapshot<TextureEncodingSession>> = state.comparisons.require(id).expect("snapshot");
  let bytes: Vec<u8> = held
    .require(TextureEncodingFormat::Bc3)
    .expect("candidate")
    .write_to_bytes()
    .expect("encoded bytes");

  state.close(&[id]).expect("close while snapshot is held");

  let next: SessionId = SessionId::generate();

  state
    .comparisons
    .begin_fixture_open(next)
    .expect("next session")
    .commit(comparison(next, "second"))
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
