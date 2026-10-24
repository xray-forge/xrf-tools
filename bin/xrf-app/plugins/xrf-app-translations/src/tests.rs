//! Pins what a save commits back, when the project it began in is no longer the open one, when what it writes moved
//! underneath it, and what an open project refuses to let a formatting run rewrite underneath it.

use std::collections::HashMap;
use std::path::PathBuf;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionScopes};
use xrf_app_core::types::TauriResult;
use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, write_generated_test_resource};
use xrf_translation::{
  TranslationEdit, TranslationProjectDescriptor, TranslationProjectMode, TranslationVariant, read_source,
};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayChangeSet, XrayMountMode, XrayRevision, XrayRoots, XraySearch, XrayTransaction};

use crate::commands::save_file::{save_into_open_project, write_edits};
use crate::state::{TranslationProjectState, TranslationSaveOutcome, TranslationSavePlan};

/// Where a source project keeps its string tables, and therefore what a project opened over one is prefixed with.
const PREFIX: &str = "translations";

/// The file every project here holds, keyed the way a descriptor keys it: relative to the prefix.
const FILE: &str = "st_test.json";

/// A second file, for the saves that must not stand in each other's way.
const OTHER_FILE: &str = "st_other.json";

/// The one entry each file holds, and the one language it is written in.
const ID: &str = "st_test";
const LANGUAGE: &str = "eng";

/// Translations state followed by its own asset state, as the plugin's setup wires them.
fn fixture() -> (AssetMountState, TranslationProjectState) {
  let assets: AssetMountState = AssetMountState::new();
  let state: TranslationProjectState = TranslationProjectState::new(&SessionScopes::default());

  assets.add_follower(state.projects.clone());

  (assets, state)
}

/// Write a source project under `name` holding `FILE` and `OTHER_FILE`, and answer the roots it is read through.
fn write_project(name: &str, value: &str) -> TauriResult<XrayRoots> {
  for file in [FILE, OTHER_FILE] {
    write_generated_test_resource(
      &format!("{name}/{PREFIX}/{file}"),
      format!(r#"{{"{ID}":{{"{LANGUAGE}":"{value}"}}}}"#),
    )
    .map_err(error_to_string)?;
  }

  Ok(XrayRoots::one(
    build_absolute_generated_test_resource_path(name),
    XrayMountMode::Directory,
  ))
}

/// The entry of `file` a project holds, which is how every assertion here tells the projects apart.
fn entry_in(descriptor: &TranslationProjectDescriptor, file: &str) -> String {
  descriptor.files[file].entries[ID][LANGUAGE]
    .as_ref()
    .expect("the fixture entry to carry its language")
    .to_single_line()
}

fn entry_of(descriptor: &TranslationProjectDescriptor) -> String {
  entry_in(descriptor, FILE)
}

/// What a file reads as now, straight off disk through a mount of its own.
fn value_on_disk(roots: &XrayRoots, file: &str) -> TauriResult<String> {
  Ok(entry_in(&read_source(roots, PREFIX).map_err(error_to_string)?, file))
}

/// What the state reports as open, or `None` when nothing is.
fn value_in_state(state: &TranslationProjectState) -> TauriResult<Option<String>> {
  Ok(state.projects.list_opened()?.first().map(|project| entry_of(project)))
}

/// Opens the project as the editor does: through the shared roots, recording what it reads, and letting the project it
/// held before go once this one is open.
fn open_project(assets: &AssetMountState, state: &TranslationProjectState, roots: &XrayRoots) -> TauriResult<()> {
  let id = SessionId::generate();
  let search: XraySearch = assets.search_fresh(roots)?;
  let revision: XrayRevision = search.get_vfs().get_revision();
  let read: SessionDerived<TranslationProjectDescriptor> =
    TranslationProjectState::read_project(search, roots, TranslationProjectMode::Source, PREFIX)?;
  let held: Vec<SessionId> = state
    .projects
    .list_opened()?
    .iter()
    .map(|project| project.session_id)
    .collect();

  state.projects.begin_fixture_open(id)?.commit_derived(read, revision)?;
  state.projects.close(&held)?;

  Ok(())
}

/// Replace a file's only entry with `value`.
fn edits(value: &str) -> HashMap<String, Vec<TranslationEdit>> {
  HashMap::from([(
    String::from(LANGUAGE),
    vec![TranslationEdit::Set {
      id: String::from(ID),
      value: TranslationVariant::String(String::from(value)),
    }],
  )])
}

fn current_id(state: &TranslationProjectState) -> SessionId {
  state.projects.list_opened().unwrap()[0].session_id
}

/// Writes and settles a save the plan was made for, as the command does once the plan is taken.
fn finish_save(
  assets: &AssetMountState,
  state: &TranslationProjectState,
  plan: &TranslationSavePlan,
  value: &str,
) -> TauriResult<TranslationSaveOutcome> {
  let changes: XrayChangeSet = write_edits(assets, plan, &edits(value))?;

  state.settle_save(plan, &changes)
}

#[test]
fn a_save_answers_the_project_as_its_commit_left_it() -> TauriResult<()> {
  let (assets, state) = fixture();
  let roots: XrayRoots = write_project("translations_state/committed", "before")?;

  open_project(&assets, &state, &roots)?;

  match save_into_open_project(&state, &assets, current_id(&state), FILE, &edits("after"))? {
    TranslationSaveOutcome::Saved { project } => {
      assert_eq!(entry_of(&project), "after");
      assert_eq!(project.revision, assets.get_vfs().get_revision());
    }
    TranslationSaveOutcome::Stale => panic!("a save of the open project is not stale"),
  }

  assert_eq!(value_on_disk(&roots, FILE)?, "after");
  assert_eq!(value_in_state(&state)?.as_deref(), Some("after"));

  Ok(())
}

#[test]
fn a_project_opened_during_a_save_stays_the_open_one() -> TauriResult<()> {
  let (assets, state) = fixture();
  let first: XrayRoots = write_project("translations_state/replaced/first", "first")?;
  let second: XrayRoots = write_project("translations_state/replaced/second", "second")?;

  open_project(&assets, &state, &first)?;

  // The save of the first project pauses here, holding everything it was addressed to.
  let plan: TranslationSavePlan = state.begin_save(current_id(&state), FILE)?;

  open_project(&assets, &state, &second)?;

  // ...and resumes, writing the project it started in, which is the right tree to write.
  assert!(matches!(
    finish_save(&assets, &state, &plan, "first edited")?,
    TranslationSaveOutcome::Stale
  ));

  // The edits landed in the project that asked for them, and the project opened meanwhile is still the open one.
  assert_eq!(value_on_disk(&first, FILE)?, "first edited");
  assert_eq!(value_on_disk(&second, FILE)?, "second");
  assert_eq!(value_in_state(&state)?.as_deref(), Some("second"));

  Ok(())
}

#[test]
fn a_close_during_a_save_is_not_undone_by_it() -> TauriResult<()> {
  let (assets, state) = fixture();
  let roots: XrayRoots = write_project("translations_state/closed", "before")?;

  open_project(&assets, &state, &roots)?;

  let plan: TranslationSavePlan = state.begin_save(current_id(&state), FILE)?;

  state.projects.close(&[current_id(&state)])?;

  assert!(matches!(
    finish_save(&assets, &state, &plan, "after")?,
    TranslationSaveOutcome::Stale
  ));
  assert_eq!(value_on_disk(&roots, FILE)?, "after");
  assert!(value_in_state(&state)?.is_none(), "a closed project stays closed");

  Ok(())
}

#[test]
fn reopening_the_same_project_is_still_a_different_session() -> TauriResult<()> {
  let (assets, state) = fixture();
  let roots: XrayRoots = write_project("translations_state/reopened", "before")?;

  open_project(&assets, &state, &roots)?;

  let plan: TranslationSavePlan = state.begin_save(current_id(&state), FILE)?;

  // The same tree, read again. Every field of the descriptor matches, which is exactly why identity is not derived
  // from them: this is a new opening, and the save that began before it no longer speaks for what is shown.
  open_project(&assets, &state, &roots)?;

  assert!(matches!(
    finish_save(&assets, &state, &plan, "after")?,
    TranslationSaveOutcome::Stale
  ));

  Ok(())
}

#[test]
fn a_save_refuses_a_file_that_changed_since_its_project_read_it() -> TauriResult<()> {
  let (assets, state) = fixture();
  let roots: XrayRoots = write_project("translations_state/moved", "before")?;

  open_project(&assets, &state, &roots)?;

  let plan: TranslationSavePlan = state.begin_save(current_id(&state), FILE)?;
  let search = assets.search(&roots)?;

  // Someone else's commit to the very file, after the plan's project read it.
  assets.commit(
    &XrayTransaction::new(&search)
      .with_write(
        &format!("{PREFIX}\\{FILE}"),
        format!(r#"{{"{ID}":{{"{LANGUAGE}":"theirs"}}}}"#),
      )
      .map_err(error_to_string)?,
  )?;

  let error: String = write_edits(&assets, &plan, &edits("mine")).expect_err("what the save writes moved");

  assert!(error.contains("stale"), "{error}");
  assert_eq!(value_on_disk(&roots, FILE)?, "theirs");
  assert_eq!(
    value_in_state(&state)?.as_deref(),
    Some("theirs"),
    "the open project read the change again"
  );

  Ok(())
}

#[test]
fn saving_one_file_does_not_stand_in_the_way_of_another() -> TauriResult<()> {
  let (assets, state) = fixture();
  let roots: XrayRoots = write_project("translations_state/two_files", "before")?;

  open_project(&assets, &state, &roots)?;

  // Both planned against the project as it opened, as two saves sent together are.
  let first: TranslationSavePlan = state.begin_save(current_id(&state), FILE)?;
  let other: TranslationSavePlan = state.begin_save(current_id(&state), OTHER_FILE)?;

  assert!(matches!(
    finish_save(&assets, &state, &first, "first")?,
    TranslationSaveOutcome::Saved { .. }
  ));

  match finish_save(&assets, &state, &other, "other")? {
    TranslationSaveOutcome::Saved { project } => {
      assert_eq!(entry_in(&project, FILE), "first");
      assert_eq!(entry_in(&project, OTHER_FILE), "other");
    }
    TranslationSaveOutcome::Stale => panic!("the project is still the open one"),
  }

  Ok(())
}

#[test]
fn a_save_needs_an_open_project_holding_the_file() -> TauriResult<()> {
  let (assets, state) = fixture();
  let roots: XrayRoots = write_project("translations_state/absent", "before")?;

  assert!(state.begin_save(SessionId::generate(), FILE).is_err());

  open_project(&assets, &state, &roots)?;

  assert_eq!(
    state.begin_save(current_id(&state), "st_missing.json").unwrap_err(),
    "Translations file 'st_missing.json' is not part of the open project"
  );

  Ok(())
}

#[test]
fn formatting_is_refused_while_a_project_is_open_over_the_same_tree() -> TauriResult<()> {
  let (assets, state) = fixture();
  let roots: XrayRoots = write_project("translations_state/open_session", "before")?;
  let root: PathBuf = roots.roots[0].path.clone();

  // Nothing open, nothing to protect.
  state.require_no_open_session_over(&root)?;

  open_project(&assets, &state, &roots)?;

  // The root itself, and the subtree the sources actually live in: the editor holds buffers over both, and a rewrite
  // under either would be undone by the next save.
  let error: String = state
    .require_no_open_session_over(&root)
    .expect_err("an open project to refuse a format over its own root");

  assert!(error.contains("Close the open translations project"), "{error}");
  assert!(
    state.require_no_open_session_over(&root.join(PREFIX)).is_err(),
    "a subtree of the open root is reached by the same rewrite"
  );

  // A sibling tree is not the open one, and a name that merely starts the same is not beneath it.
  let sibling: XrayRoots = write_project("translations_state/open_session_other", "before")?;

  state.require_no_open_session_over(&sibling.roots[0].path)?;

  // Closing releases it.
  state.projects.close(&[current_id(&state)])?;
  state.require_no_open_session_over(&root)?;

  Ok(())
}
