//! An open level through commits: one to a file the level itself was read from reads it again off the commit and its
//! scene streams afresh; one to a file only a part read, a render read's or the start's, names that part stale.

use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rayon::ThreadPool;
use xrf_app_core::assets::{AssetChangeEvent, AssetChangeSubscriber, AssetMountState};
use xrf_app_core::session::{
  SessionChange, SessionDerived, SessionId, SessionPart, SessionRootChange, SessionScopes, SessionSnapshot,
};
use xrf_app_render::WorldState;
use xrf_dltx::DltxDialect;
use xrf_engine_target::{XrayEngine, XrayEngineResolution};
use xrf_level::{LevelGeomFile, LevelHeaderChunk, LevelShadersChunk, LevelVisualsChunk};
use xrf_renderer::{RenderLevelSource, RenderSceneId, RenderViewportId, RenderWorkers, RenderWorld};
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayMountMode, XrayRevision, XrayRoots, XraySearch, XrayTransaction};
use xrf_world::{WorldEventSink, WorldViewportEvent};

use crate::level::{LevelSource, OpenedLevel, SelectedLevel, SelectedLevels};
use crate::parts::{
  LevelDetailsPart, LevelEnvironment, LevelGeometryPart, LevelLightsPart, LevelMapPart, LevelObjectMotionsPart,
  LevelParticlesPart, LevelSectionsPart, LevelSpawnPart, LevelStartPart, LevelSurfacesPart, LevelTexturesPart,
  LevelWeatherPart,
};
use crate::render::{LevelRenderSource, LevelSceneFollower, ShownLevels};

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

/// The texture the level's one shader entry binds.
const TEXTURE: &str = "textures\\prop\\prop_fence.dds";

/// The renderer script the entry's shader is looked up as, which its surface is described from where it is found.
const SHADER_SCRIPT: &str = "shaders\\r3\\def_shaders_def_vertex.s";

/// The smallest `level` a read takes: a header, a shader table of one entry and an empty run of visuals.
fn level_bytes() -> Vec<u8> {
  let mut bytes: Vec<u8> = chunk(LevelHeaderChunk::CHUNK_ID, &[14, 0, 1, 0]);
  let mut shaders: Vec<u8> = 1u32.to_le_bytes().to_vec();

  shaders.extend_from_slice(b"def_shaders\\def_vertex/prop\\prop_fence\0");
  bytes.extend(chunk(LevelShadersChunk::CHUNK_ID, &shaders));
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
  let root: PathBuf = GeneratedResources::path(&format!("level_follow/{name}"));
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

/// The levels as the plugin holds them, followed through `assets`' commits: read again on `pool` where one is given, as
/// the plugin does, or inside the commit, so a test reads what a commit moved as soon as it returns.
fn new_levels(assets: &AssetMountState, pool: Option<Arc<ThreadPool>>) -> SelectedLevels {
  let levels: SelectedLevels = SelectedLevels::new("level", &SessionScopes::default());
  let levels: SelectedLevels = match pool {
    Some(pool) => levels.with_deferred_derivation(assets, pool, OpenedLevel::derive),
    None => levels.with_derivation(assets, OpenedLevel::derive),
  };

  levels.with_view_parts(LevelRenderSource::VIEW_PARTS)
}

/// Opens the tree's level as one more handle of `levels`, as `levels|open_level` does.
fn open(levels: &SelectedLevels, assets: &AssetMountState, roots: &XrayRoots) -> Arc<SessionSnapshot<SelectedLevel>> {
  let id: SessionId = SessionId::generate();
  let (opened, revision): (SessionDerived<SelectedLevel>, XrayRevision) = read_level(assets, roots);

  levels
    .begin_fixture_open(id)
    .unwrap()
    .commit_fixture(opened, revision)
    .unwrap()
}

/// The one change a commit moved, once the subscriber took it.
fn take_change(log: &ChangeLog) -> SessionChange {
  assert!(wait_until(|| !log.list().is_empty()), "the level moved");

  let mut changes: Vec<SessionChange> = log.drain();

  assert_eq!(changes.len(), 1, "{changes:?}");

  changes.remove(0)
}

/// The tree's level as an open reads it, with the revision of the search it read.
fn read_level(assets: &AssetMountState, roots: &XrayRoots) -> (SessionDerived<SelectedLevel>, XrayRevision) {
  let source: LevelSource = LevelSource::Asset {
    logical_path: String::from(LEVEL),
  };
  let search: XraySearch = assets.search(roots).expect("the tree mounts");
  let revision: XrayRevision = search.get_vfs().get_revision();
  let opened: SessionDerived<SelectedLevel> =
    OpenedLevel::read(search, source, roots.clone(), DltxDialect::select(false), |_, _| {
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

  fn drain(&self) -> Vec<SessionChange> {
    std::mem::take(&mut *self.taken.lock().unwrap())
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
  let levels: SelectedLevels = new_levels(&assets, Some(Arc::clone(&pool)));
  let world: WorldState = WorldState::new(RenderWorkers::new(Arc::clone(&pool)));
  let shown: ShownLevels = ShownLevels::default();
  let log: ChangeLog = ChangeLog::default();
  let viewport: RenderViewportId = RenderViewportId(1);

  assets.add_subscriber(LevelSceneFollower {
    levels: levels.clone(),
    shown: shown.clone(),
    world: world.clone(),
  });
  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<SelectedLevel>> = open(&levels, &assets, &roots);
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
  let next: Arc<SessionSnapshot<SelectedLevel>> = levels.require(opened.session_id).expect("the level stays open");

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

#[test]
fn a_commit_to_a_texture_of_a_shown_level_moves_its_scene_onto_the_next_snapshot_in_place() {
  let (assets, roots, search) = level_tree("kept");
  let pool: Arc<ThreadPool> = test_pool();
  let levels: SelectedLevels = new_levels(&assets, Some(Arc::clone(&pool)));
  let world: WorldState = WorldState::new(RenderWorkers::new(Arc::clone(&pool)));
  let shown: ShownLevels = ShownLevels::default();
  let log: ChangeLog = ChangeLog::default();
  let viewport: RenderViewportId = RenderViewportId(1);

  assets.add_subscriber(LevelSceneFollower {
    levels: levels.clone(),
    shown: shown.clone(),
    world: world.clone(),
  });
  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<SelectedLevel>> = open(&levels, &assets, &roots);
  let source: Arc<LevelRenderSource> = shown.get_source(Arc::clone(&opened), &assets);
  let scene: RenderSceneId = {
    let mut world = world.lock();

    world.attach(viewport, Box::new(Silence));
    world
      .show_level(viewport, Some(Arc::clone(&source) as Arc<dyn RenderLevelSource>))
      .expect("the level shown")
  };

  write(&assets, &search, TEXTURE, "created");

  let change: SessionChange = take_change(&log);
  let next: Arc<SessionSnapshot<SelectedLevel>> = levels.require(opened.session_id).expect("the level stays open");
  let followed: Arc<LevelRenderSource> = shown.find_source(opened.session_id).expect("the next source is drawn");

  assert_eq!(change.root, SessionRootChange::Kept);
  assert_eq!(change.stale_parts, vec![LevelTexturesPart::NAME]);
  assert!(!change.is_view_renewed);
  assert_eq!(
    world.lock().get_asked_scene(viewport),
    Some(scene),
    "the scene reloads in place"
  );
  assert!(Arc::ptr_eq(followed.get_level(), &next));
  assert!(Arc::ptr_eq(&next.value, &opened.value), "the level itself was kept");
  assert!(
    followed.shares_textures_with(&source),
    "the textures resolved go on with the scene"
  );
}

#[test]
fn a_show_holding_a_snapshot_a_commit_moved_past_draws_the_source_the_commit_moved_on() {
  let (assets, roots, search) = level_tree("late_show");
  let pool: Arc<ThreadPool> = test_pool();
  let levels: SelectedLevels = new_levels(&assets, Some(Arc::clone(&pool)));
  let world: WorldState = WorldState::new(RenderWorkers::new(Arc::clone(&pool)));
  let shown: ShownLevels = ShownLevels::default();
  let viewport: RenderViewportId = RenderViewportId(1);

  assets.add_subscriber(LevelSceneFollower {
    levels: levels.clone(),
    shown: shown.clone(),
    world: world.clone(),
  });

  let opened: Arc<SessionSnapshot<SelectedLevel>> = open(&levels, &assets, &roots);
  let source: Arc<LevelRenderSource> = shown.get_source(Arc::clone(&opened), &assets);

  {
    let mut world = world.lock();

    world.attach(viewport, Box::new(Silence));
    world
      .show_level(viewport, Some(Arc::clone(&source) as Arc<dyn RenderLevelSource>))
      .expect("the level shown");
  }

  write(&assets, &search, TEXTURE, "created");

  let next: Arc<SessionSnapshot<SelectedLevel>> = levels.require(opened.session_id).expect("the level stays open");
  let followed: Arc<LevelRenderSource> = shown.find_source(opened.session_id).expect("the next source is drawn");
  // A show that took its snapshot before the commit, landing after the follow.
  let late: Arc<LevelRenderSource> = shown.get_source(Arc::clone(&opened), &assets);

  assert!(
    Arc::ptr_eq(&late, &followed),
    "the late show draws what the follow drew"
  );
  assert!(Arc::ptr_eq(
    shown
      .find_source(opened.session_id)
      .expect("a source is drawn")
      .get_level(),
    &next
  ));
}

#[test]
fn every_show_of_one_snapshot_shares_one_source() {
  let (assets, roots, _) = level_tree("shared_source");
  let levels: SelectedLevels = new_levels(&assets, Some(test_pool()));
  let shown: ShownLevels = ShownLevels::default();
  let opened: Arc<SessionSnapshot<SelectedLevel>> = open(&levels, &assets, &roots);
  let first: Arc<LevelRenderSource> = shown.get_source(Arc::clone(&opened), &assets);
  let second: Arc<LevelRenderSource> = shown.get_source(Arc::clone(&opened), &assets);

  assert!(Arc::ptr_eq(&first, &second));
}

/// The levels a tool opens renew their view by the parts a level's scene is built from, which the state itself names.
#[test]
fn a_commit_to_the_geometry_of_an_open_level_renews_its_view() {
  let (assets, roots, search) = level_tree("view_parts");
  let levels: SelectedLevels = new_levels(&assets, Some(test_pool()));
  let log: ChangeLog = ChangeLog::default();

  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<SelectedLevel>> = open(&levels, &assets, &roots);

  write(&assets, &search, &format!("{LEVEL}\\level.geom"), "edited");

  let change: SessionChange = take_change(&log);

  assert_eq!(change.session_id, opened.session_id);
  assert_eq!(change.root, SessionRootChange::Kept);
  assert_eq!(change.stale_parts, vec![LevelGeometryPart::NAME]);
  assert!(change.is_view_renewed, "the sectors are packed from the geometry");
}

/// The textures are resolved over the surfaces, so what moves the surfaces moves them too: as the open kept them, and
/// as a read of the textures again asks for the surfaces.
#[test]
fn a_commit_to_what_the_surfaces_read_lets_the_textures_go_with_them() {
  let (assets, roots, search) = level_tree("surface_reads");
  let levels: SelectedLevels = new_levels(&assets, None);
  let log: ChangeLog = ChangeLog::default();

  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<SelectedLevel>> = open(&levels, &assets, &roots);

  write(&assets, &search, SHADER_SCRIPT, "created");

  let change: SessionChange = take_change(&log);

  assert_eq!(change.root, SessionRootChange::Kept);
  assert_eq!(
    change.stale_parts,
    vec![LevelSurfacesPart::NAME, LevelTexturesPart::NAME]
  );
  assert!(change.is_view_renewed, "the surfaces the scene is drawn with moved");

  // Read again, the textures ask for the surfaces, whose inputs join theirs.
  let current: Arc<SessionSnapshot<SelectedLevel>> = levels.require(opened.session_id).unwrap();

  LevelTexturesPart::get(&current, &assets.search(&roots).expect("the tree mounts"));
  write(&assets, &search, SHADER_SCRIPT, "edited");

  let change: SessionChange = take_change(&log);

  assert_eq!(
    change.stale_parts,
    vec![LevelSurfacesPart::NAME, LevelTexturesPart::NAME]
  );
}

/// The lights' read joins the spawn it asks for, so the spawn is committed after the lights went stale on their own.
#[test]
fn a_commit_to_a_file_only_a_part_read_names_that_part_stale() {
  let (assets, roots, search) = level_tree("render_reads");
  let levels: SelectedLevels = new_levels(&assets, None);
  let log: ChangeLog = ChangeLog::default();

  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<SelectedLevel>> = open(&levels, &assets, &roots);
  let source: LevelRenderSource = LevelRenderSource::new(Arc::clone(&opened), AssetMountState::clone(&assets));

  // Each read finds nothing of what it looks for in the synthetic level, which it records as it would a file.
  assert!(source.read_details().unwrap().is_none());
  assert!(source.read_particles().is_err());
  assert!(source.read_lights().is_ok());
  assert!(source.read_object_motion("zone.anm").is_err());
  LevelWeatherPart::record(&opened, &search, |search| {
    LevelEnvironment::read_modifiers(&opened, search)
  });

  for (path, parts, is_view_renewed) in [
    (format!("{LEVEL}\\level.details"), vec![LevelDetailsPart::NAME], false),
    (String::from("particles.xr"), vec![LevelParticlesPart::NAME], false),
    (String::from("lanims.xr"), vec![LevelLightsPart::NAME], false),
    (
      String::from("anims\\zone.anm"),
      vec![LevelObjectMotionsPart::NAME],
      false,
    ),
    (format!("{LEVEL}\\level.env_mod"), vec![LevelWeatherPart::NAME], false),
    // The map, the geometry and the textures the surfaces bind are read from files of their own, so an edit of any
    // moves that part and not the level: a texture come or gone moves the textures alone, the surfaces kept. Only the
    // geometry renews the scene: its sectors are packed from it.
    (format!("{LEVEL}\\level.ltx"), vec![LevelMapPart::NAME], false),
    (format!("{LEVEL}\\level.geom"), vec![LevelGeometryPart::NAME], true),
    (String::from(TEXTURE), vec![LevelTexturesPart::NAME], false),
    // Where the level starts is decided over its spawn apart from the level itself, as the sections the lights asked
    // for are chosen by it.
    (
      String::from("spawns\\all.spawn"),
      vec![LevelSectionsPart::NAME, LevelSpawnPart::NAME, LevelStartPart::NAME],
      false,
    ),
  ] {
    write(&assets, &search, &path, "created");

    let change: SessionChange = take_change(&log);

    assert_eq!(change.root, SessionRootChange::Kept, "{path}");
    assert_eq!(change.stale_parts, parts, "{path}");
    assert_eq!(change.is_view_renewed, is_view_renewed, "{path}");
  }

  let current: Arc<SessionSnapshot<SelectedLevel>> = levels.require(opened.session_id).unwrap();

  assert!(
    Arc::ptr_eq(&current.value, &opened.value),
    "the level itself was never read again"
  );
}
