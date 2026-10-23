//! An open level through commits: one to a file the level itself was read from reads it again off the commit and its
//! scene streams afresh; one to a file only a part read, a render read's or the start's, names that part stale.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rayon::ThreadPool;
use xrf_dltx::select_ltx_dialect;
use xrf_engine_target::{XrayEngine, XrayEngineResolution};
use xrf_level::{LevelGeomFile, LevelHeaderChunk, LevelVisualsChunk};
use xrf_renderer::{RenderLevelSource, RenderSceneId, RenderViewportId, RenderWorkers, RenderWorld};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayMountMode, XrayRevision, XrayRoots, XraySearch, XrayTransaction};
use xrf_world::{WorldEventSink, WorldViewportEvent};

use crate::core::assets::{AssetChangeEvent, AssetChangeSubscriber, AssetMountState};
use crate::core::session::{
  Session, SessionChange, SessionDerived, SessionId, SessionRootChange, SessionScopes, SessionSnapshot, Sessions,
};
use crate::plugins::levels::configs::LEVEL_SECTIONS_PART;
use crate::plugins::levels::details::LEVEL_DETAILS_PART;
use crate::plugins::levels::level_scene_follower::LevelSceneFollower;
use crate::plugins::levels::lights::LEVEL_LIGHTS_PART;
use crate::plugins::levels::object_motions::LEVEL_OBJECT_MOTIONS_PART;
use crate::plugins::levels::opened_level::OpenedLevel;
use crate::plugins::levels::particles::LEVEL_PARTICLES_PART;
use crate::plugins::levels::render_source::LevelRenderSource;
use crate::plugins::levels::shown_levels::ShownLevels;
use crate::plugins::levels::spawn::LEVEL_SPAWN_PART;
use crate::plugins::levels::start::LEVEL_START_PART;
use crate::plugins::levels::state::level_environment::LEVEL_WEATHER_PART;
use crate::plugins::levels::state::{LevelEnvironment, LevelSource, LevelState, SelectedLevel};
use crate::plugins::render::world_state::WorldState;

/// Longer than a synthetic level takes to read on the test pool.
const WAIT: Duration = Duration::from_secs(10);

/// The level's own directory in its tree.
const LEVEL: &str = "levels\\l01";

/// One chunk as a file stores it.
fn chunk(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut bytes: Vec<u8> = id.to_le_bytes().to_vec();

  bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
  bytes.extend_from_slice(payload);

  bytes
}

/// The smallest `level` a read takes: a header and an empty run of visuals.
fn level_bytes() -> Vec<u8> {
  let mut bytes: Vec<u8> = chunk(LevelHeaderChunk::CHUNK_ID, &[14, 0, 1, 0]);

  bytes.extend(chunk(LevelVisualsChunk::CHUNK_ID, &[]));

  bytes
}

/// The smallest `level.geom`: no vertex buffers and no index buffers.
fn geometry_bytes() -> Vec<u8> {
  let mut bytes: Vec<u8> = chunk(LevelGeomFile::VERTEX_BUFFERS_CHUNK_ID, &0u32.to_le_bytes());

  bytes.extend(chunk(LevelGeomFile::INDEX_BUFFERS_CHUNK_ID, &0u32.to_le_bytes()));

  bytes
}

/// A loose tree holding one synthetic level behind an asset state, with the roots and the search an open takes.
fn level_tree(name: &str) -> (AssetMountState, XrayRoots, XraySearch) {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("level_follow/{name}"));
  let level: PathBuf = root.join("levels").join("l01");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&level).expect("level directory created");
  fs::write(level.join("level"), level_bytes()).expect("level written");
  fs::write(level.join("level.geom"), geometry_bytes()).expect("geometry written");

  let assets: AssetMountState = AssetMountState::new();
  let roots: XrayRoots = XrayRoots::one(root, XrayMountMode::Directory);
  let search: XraySearch = assets.search(&roots).expect("the tree mounts");

  (assets, roots, search)
}

/// Opens the tree's level in `session` as `levels|open_level` does.
fn open(
  session: &Session<SelectedLevel>,
  assets: &AssetMountState,
  roots: &XrayRoots,
) -> Arc<SessionSnapshot<SelectedLevel>> {
  let id: SessionId = SessionId::new();
  let (opened, revision): (SessionDerived<SelectedLevel>, XrayRevision) = read_level(assets, roots);

  session.begin_open(id).unwrap();
  session
    .commit_open_with(id, opened.value, revision, opened.inputs, opened.parts)
    .unwrap()
}

/// Opens the tree's level as one more handle of `levels`, as `levels|open_level` does.
fn open_handle(
  levels: &Sessions<SelectedLevel>,
  assets: &AssetMountState,
  roots: &XrayRoots,
) -> Arc<SessionSnapshot<SelectedLevel>> {
  let id: SessionId = SessionId::new();
  let (opened, revision): (SessionDerived<SelectedLevel>, XrayRevision) = read_level(assets, roots);

  levels.begin_open(id).unwrap();
  levels
    .commit_open_with(id, opened.value, revision, opened.inputs, opened.parts)
    .unwrap()
}

/// The tree's level as an open reads it, with the revision of the search it read.
fn read_level(assets: &AssetMountState, roots: &XrayRoots) -> (SessionDerived<SelectedLevel>, XrayRevision) {
  let source: LevelSource = LevelSource::Asset {
    logical_path: String::from(LEVEL),
  };
  let search: XraySearch = assets.search(roots).expect("the tree mounts");
  let revision: XrayRevision = search.get_vfs().get_revision();
  let opened: SessionDerived<SelectedLevel> =
    OpenedLevel::read(search, source, roots.clone(), select_ltx_dialect(false), |_, _| {
      XrayEngineResolution::named(XrayEngine::Vanilla)
    })
    .expect("the level reads");

  (opened, revision)
}

/// Commits `bytes` as one file of the tree, as a save does.
fn write(assets: &AssetMountState, search: &XraySearch, path: &str, bytes: impl Into<Vec<u8>>) {
  assets
    .commit(&XrayTransaction::new(search).with_write(path, bytes).unwrap())
    .expect("the write commits");
}

fn test_pool() -> Arc<ThreadPool> {
  Arc::new(
    rayon::ThreadPoolBuilder::new()
      .num_threads(2)
      .build()
      .expect("a test pool"),
  )
}

/// Waits for `is_done`, a few seconds at most.
fn wait_until(mut is_done: impl FnMut() -> bool) -> bool {
  let started: Instant = Instant::now();

  while !is_done() {
    if started.elapsed() > WAIT {
      return false;
    }

    std::thread::sleep(Duration::from_millis(5));
  }

  true
}

/// Every session change a subscriber took, in order.
#[derive(Clone, Default)]
struct ChangeLog {
  taken: Arc<Mutex<Vec<SessionChange>>>,
}

impl ChangeLog {
  fn list(&self) -> Vec<SessionChange> {
    self.taken.lock().unwrap().clone()
  }
}

impl AssetChangeSubscriber for ChangeLog {
  fn take_change(&self, _: &AssetMountState, event: &AssetChangeEvent) {
    self.taken.lock().unwrap().extend(event.sessions.iter().cloned());
  }
}

/// A viewport's world events sent nowhere.
struct Silence;

impl WorldEventSink for Silence {
  fn send(&self, _: WorldViewportEvent) {}
}

#[test]
fn a_commit_to_a_file_the_level_was_read_from_reads_it_again_and_streams_its_scene_afresh() {
  let (assets, roots, search) = level_tree("derived");
  let pool: Arc<ThreadPool> = test_pool();
  let state: LevelState = LevelState::new(Arc::clone(&pool), &SessionScopes::default());
  let world: WorldState = WorldState::new(RenderWorkers::new(Arc::clone(&pool)));
  let shown: ShownLevels = ShownLevels::default();
  let log: ChangeLog = ChangeLog::default();
  let viewport: RenderViewportId = RenderViewportId(1);

  assets.add_follower(state.levels.clone());
  assets.add_subscriber(LevelSceneFollower {
    levels: state.levels.clone(),
    shown: shown.clone(),
    world: world.clone(),
  });
  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<SelectedLevel>> = open_handle(&state.levels, &assets, &roots);
  let source: Arc<LevelRenderSource> = shown.get_source(Arc::clone(&opened), &assets);
  let scene: RenderSceneId = {
    let mut world = world.lock();

    world.attach(viewport, Box::new(Silence));
    world
      .show_level(viewport, Some(source as Arc<dyn RenderLevelSource>))
      .expect("the level shown")
  };

  write(&assets, &search, &format!("{LEVEL}\\level"), level_bytes());

  assert!(
    wait_until(|| log.list().len() == 2),
    "the derivation landed: {:?}",
    log.list()
  );

  let changes: Vec<SessionChange> = log.list();
  let next: Arc<SessionSnapshot<SelectedLevel>> =
    state.levels.require(opened.session_id).expect("the level stays open");

  assert_eq!(changes[0].root, SessionRootChange::Deriving);
  assert_eq!(
    changes[0].revision, opened.revision,
    "the session waits at its own revision"
  );
  assert_eq!(changes[1].root, SessionRootChange::Derived);
  assert_eq!(changes[1].revision, next.revision);
  assert!(next.revision > opened.revision);
  assert!(!Arc::ptr_eq(&next.value, &opened.value), "the value was read again");
  assert_eq!(next.session_id, opened.session_id);

  let reopened: Option<RenderSceneId> = world.lock().get_asked_scene(viewport);

  assert!(
    reopened.is_some_and(|reopened| reopened != scene),
    "{reopened:?} after {scene:?}"
  );
  assert!(Arc::ptr_eq(
    shown
      .find_source(opened.session_id)
      .expect("the next source is drawn")
      .get_level(),
    &next
  ));
}

/// The lights' read joins the spawn it asks for, so the spawn is committed after the lights went stale on their own.
#[test]
fn a_commit_to_a_file_only_a_part_read_names_that_part_stale() {
  let (assets, roots, search) = level_tree("render_reads");
  let session: Session<SelectedLevel> = Session::new("level");
  let opened: Arc<SessionSnapshot<SelectedLevel>> = open(&session, &assets, &roots);
  let source: LevelRenderSource = LevelRenderSource::new(Arc::clone(&opened), AssetMountState::clone(&assets));

  // Each read finds nothing of what it looks for in the synthetic level, which it records as it would a file.
  assert!(source.read_details().unwrap().is_none());
  assert!(source.read_particles().is_err());
  assert!(source.read_lights().is_ok());
  assert!(source.read_object_motion("zone.anm").is_err());
  LevelEnvironment::record_weather(&opened, &search, |search| {
    LevelEnvironment::read_modifiers(&opened, search)
  });

  for (path, parts) in [
    (format!("{LEVEL}\\level.details"), vec![LEVEL_DETAILS_PART]),
    (String::from("particles.xr"), vec![LEVEL_PARTICLES_PART]),
    (String::from("lanims.xr"), vec![LEVEL_LIGHTS_PART]),
    (String::from("anims\\zone.anm"), vec![LEVEL_OBJECT_MOTIONS_PART]),
    (format!("{LEVEL}\\level.env_mod"), vec![LEVEL_WEATHER_PART]),
    // Where the level starts is decided over its spawn apart from the level itself, as the sections the lights asked
    // for are chosen by it.
    (
      String::from("spawns\\all.spawn"),
      vec![LEVEL_SECTIONS_PART, LEVEL_SPAWN_PART, LEVEL_START_PART],
    ),
  ] {
    write(&assets, &search, &path, "created");

    let change: SessionChange = session.follow(&assets).unwrap().expect("the level moved");

    assert_eq!(change.root, SessionRootChange::Kept, "{path}");
    assert_eq!(change.stale_parts, parts, "{path}");
  }

  let current: Arc<SessionSnapshot<SelectedLevel>> = session.get().unwrap().unwrap();

  assert!(
    Arc::ptr_eq(&current.value, &opened.value),
    "the level itself was never read again"
  );
}
