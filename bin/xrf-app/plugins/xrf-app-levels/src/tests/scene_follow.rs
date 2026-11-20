use std::sync::{Arc, Mutex};

use xrf_app_core::assets::{AssetChangeEvent, AssetChangeSubscriber, AssetMountState};
use xrf_app_core::session::{SessionChange, SessionPart, SessionRootChange, SessionSnapshot};
use xrf_editable_level::{EditableLightObject, EditableSceneObject};
use xrf_vfs::{XraySearch, XrayTransaction};

use crate::scene::{
  SceneObjectProperties, SceneObjectsDescription, SceneObjectsPart, SceneSource, SelectedScene, SelectedScenes,
};
use crate::tests::fixtures::{
  SCENE_NAME, new_scenes, open_scene, scene_custom_object, scene_part, write_scene, write_scene_library_object,
  write_scene_part,
};

#[derive(Clone, Default)]
struct ChangeLog {
  taken: Arc<Mutex<Vec<SessionChange>>>,
}

impl ChangeLog {
  fn drain(&self) -> Vec<SessionChange> {
    std::mem::take(&mut *self.taken.lock().unwrap())
  }
}

impl AssetChangeSubscriber for ChangeLog {
  fn take_change(&self, _: &AssetMountState, event: &AssetChangeEvent) {
    self.taken.lock().unwrap().extend(event.sessions.iter().cloned());
  }
}

fn scene_object(reference: &str) -> EditableSceneObject {
  EditableSceneObject {
    object: scene_custom_object("stol_0000", 0.0),
    reference: String::from(reference),
    flags: 0,
  }
}

fn is_found(scene: &SessionSnapshot<SelectedScene>, objects: &XraySearch) -> bool {
  let described: Arc<SceneObjectsDescription> = SceneObjectsPart::get(scene, objects);

  matches!(
    described.tools[0].objects[0].properties,
    SceneObjectProperties::SceneObject { is_found: true, .. }
  )
}

#[test]
fn a_commit_to_a_part_file_or_a_part_file_come_reads_the_scene_again() {
  let source: SceneSource = write_scene("scene_follow/parts");

  write_scene_part(
    &source,
    "scene_object.part",
    scene_part(vec![scene_object("statics\\stol")]),
  );

  let assets: AssetMountState = AssetMountState::new();
  let scenes: SelectedScenes = new_scenes(&assets);
  let log: ChangeLog = ChangeLog::default();

  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&scenes, &assets, &source);
  let search: XraySearch = assets.search(&opened.roots).unwrap();

  assert_eq!(
    opened.tools.iter().map(|tool| tool.name.as_str()).collect::<Vec<_>>(),
    ["scene_object"],
    "the part file found and listed below the scene's directory"
  );

  assets
    .commit(
      &XrayTransaction::new(&search)
        .with_write(
          &format!("{SCENE_NAME}\\light.part"),
          scene_part(vec![EditableLightObject::new(scene_custom_object("light_0000", 0.0))]),
        )
        .unwrap(),
    )
    .unwrap();

  let changes: Vec<SessionChange> = log.drain();
  let next: Arc<SessionSnapshot<SelectedScene>> = scenes.require(opened.session_id).unwrap();

  assert_eq!(changes.len(), 1, "{changes:?}");
  assert_eq!(changes[0].root, SessionRootChange::Derived);
  assert!(next.revision > opened.revision);
  assert_eq!(
    next.tools.iter().map(|tool| tool.name.as_str()).collect::<Vec<_>>(),
    ["scene_object", "light"],
    "the part file come is read with the scene"
  );

  assets
    .commit(
      &XrayTransaction::new(&search)
        .with_write(&format!("{SCENE_NAME}\\scene_object.part"), "edited")
        .unwrap(),
    )
    .unwrap();

  let changes: Vec<SessionChange> = log.drain();
  let edited: Arc<SessionSnapshot<SelectedScene>> = scenes.require(opened.session_id).unwrap();

  assert_eq!(changes.len(), 1, "{changes:?}");
  assert_eq!(changes[0].root, SessionRootChange::Derived);
  assert_eq!(
    edited
      .problems
      .iter()
      .map(|problem| problem.tool.as_str())
      .collect::<Vec<_>>(),
    ["scene_object"],
    "the part edited into one that does not read is a problem now"
  );
}

#[test]
fn a_library_object_come_into_the_objects_roots_names_only_the_objects_part_stale() {
  let source: SceneSource = write_scene("scene_follow/objects");

  write_scene_library_object(&source, "statics\\stol\\stol_4");
  write_scene_part(
    &source,
    "scene_object.part",
    scene_part(vec![scene_object("statics\\stol\\stol_5")]),
  );

  let assets: AssetMountState = AssetMountState::new();
  let scenes: SelectedScenes = new_scenes(&assets);
  let log: ChangeLog = ChangeLog::default();

  assets.add_subscriber(log.clone());

  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&scenes, &assets, &source);
  let objects: XraySearch = assets.search(&opened.objects_roots).unwrap();

  assert!(!is_found(&opened, &objects));

  assets
    .commit(
      &XrayTransaction::new(&objects)
        .with_write("statics\\stol\\stol_5.object", "object")
        .unwrap(),
    )
    .unwrap();

  let changes: Vec<SessionChange> = log.drain();
  let next: Arc<SessionSnapshot<SelectedScene>> = scenes.require(opened.session_id).unwrap();

  assert_eq!(changes.len(), 1, "{changes:?}");
  assert_eq!(changes[0].root, SessionRootChange::Kept);
  assert_eq!(changes[0].stale_parts, [SceneObjectsPart::NAME]);
  assert!(Arc::ptr_eq(&next.value, &opened.value), "the scene itself was kept");
  assert!(
    is_found(&next, &assets.search(&next.objects_roots).unwrap()),
    "read again, the library object is found"
  );
}
