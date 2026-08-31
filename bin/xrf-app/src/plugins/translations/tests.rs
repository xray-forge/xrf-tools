//! Pins what a save commits back, when the project it began in is no longer the open one.

use std::collections::HashMap;

use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, write_generated_test_resource};
use xrf_translation::{TranslationEdit, TranslationProjectDescriptor, TranslationVariant, read_source};
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::core::types::TauriResult;
use crate::plugins::translations::commands::save_file::{save_into_open_project, write_edits};
use crate::plugins::translations::state::{TranslationProjectState, TranslationSaveOutcome, TranslationSavePlan};

/// Where a source project keeps its string tables, and therefore what a project opened over one is prefixed with.
const PREFIX: &str = "translations";

/// The one file every project here holds, keyed the way a descriptor keys it: relative to the prefix.
const FILE: &str = "st_test.json";

/// The one entry that file holds, and the one language it is written in.
const ID: &str = "st_test";
const LANGUAGE: &str = "eng";

/// Write a one-file source project under `name` and answer the roots it is read through.
fn write_project(name: &str, value: &str) -> TauriResult<XrayRoots> {
  write_generated_test_resource(
    &format!("{name}/{PREFIX}/{FILE}"),
    format!(r#"{{"{ID}":{{"{LANGUAGE}":"{value}"}}}}"#),
  )
  .map_err(|error| error.to_string())?;

  Ok(XrayRoots::one(
    build_absolute_generated_test_resource_path(name),
    XrayMountMode::Directory,
  ))
}

fn read_project(roots: &XrayRoots) -> TauriResult<TranslationProjectDescriptor> {
  read_source(roots, PREFIX).map_err(|error| error.to_string())
}

/// The one entry a project holds, which is how every assertion here tells the projects apart.
fn entry_of(descriptor: &TranslationProjectDescriptor) -> String {
  descriptor.files[FILE].entries[ID][LANGUAGE]
    .as_ref()
    .expect("the fixture entry to carry its language")
    .to_single_line()
}

/// What the file reads as now, straight off disk.
fn value_on_disk(roots: &XrayRoots) -> TauriResult<String> {
  Ok(entry_of(&read_project(roots)?))
}

/// What the state reports as open, or `None` when nothing is.
fn value_in_state(state: &TranslationProjectState) -> TauriResult<Option<String>> {
  Ok(state.get_project()?.as_ref().map(entry_of))
}

fn open_project(state: &TranslationProjectState, roots: &XrayRoots) -> TauriResult<()> {
  state.open_project(read_project(roots)?)
}

/// Replace the file's only entry with `value`.
fn edits(value: &str) -> HashMap<String, Vec<TranslationEdit>> {
  HashMap::from([(
    String::from(LANGUAGE),
    vec![TranslationEdit::Set {
      id: String::from(ID),
      value: TranslationVariant::String(String::from(value)),
    }],
  )])
}

#[test]
fn a_save_commits_the_project_it_refreshed() -> TauriResult<()> {
  let state: TranslationProjectState = TranslationProjectState::new();
  let roots: XrayRoots = write_project("translations_state/committed", "before")?;

  open_project(&state, &roots)?;

  match save_into_open_project(&state, FILE, &edits("after"))? {
    TranslationSaveOutcome::Saved { project } => assert_eq!(entry_of(&project), "after"),
    TranslationSaveOutcome::Stale => panic!("a save of the open project is not stale"),
  }

  assert_eq!(value_on_disk(&roots)?, "after");
  assert_eq!(value_in_state(&state)?.as_deref(), Some("after"));

  Ok(())
}

#[test]
fn a_project_opened_during_a_save_stays_the_open_one() -> TauriResult<()> {
  let state: TranslationProjectState = TranslationProjectState::new();
  let first: XrayRoots = write_project("translations_state/replaced/first", "first")?;
  let second: XrayRoots = write_project("translations_state/replaced/second", "second")?;

  open_project(&state, &first)?;

  // The save of the first project pauses here, holding everything it was addressed to.
  let plan: TranslationSavePlan = state.begin_save(FILE)?;

  open_project(&state, &second)?;

  // ...and resumes, writing and re-reading the project it started in, which is the right tree to write.
  let refreshed: TranslationProjectDescriptor = write_edits(&plan, &edits("first edited"))?;

  assert!(matches!(
    state.commit_save(&plan, refreshed)?,
    TranslationSaveOutcome::Stale
  ));

  // The edits landed in the project that asked for them, and the project opened meanwhile is still the open one.
  assert_eq!(value_on_disk(&first)?, "first edited");
  assert_eq!(value_on_disk(&second)?, "second");
  assert_eq!(value_in_state(&state)?.as_deref(), Some("second"));

  Ok(())
}

#[test]
fn a_close_during_a_save_is_not_undone_by_it() -> TauriResult<()> {
  let state: TranslationProjectState = TranslationProjectState::new();
  let roots: XrayRoots = write_project("translations_state/closed", "before")?;

  open_project(&state, &roots)?;

  let plan: TranslationSavePlan = state.begin_save(FILE)?;

  state.close_project()?;

  let refreshed: TranslationProjectDescriptor = write_edits(&plan, &edits("after"))?;

  assert!(matches!(
    state.commit_save(&plan, refreshed)?,
    TranslationSaveOutcome::Stale
  ));

  assert_eq!(value_on_disk(&roots)?, "after");
  assert!(value_in_state(&state)?.is_none(), "a closed project stays closed");

  Ok(())
}

#[test]
fn reopening_the_same_project_is_still_a_different_session() -> TauriResult<()> {
  let state: TranslationProjectState = TranslationProjectState::new();
  let roots: XrayRoots = write_project("translations_state/reopened", "before")?;

  open_project(&state, &roots)?;

  let plan: TranslationSavePlan = state.begin_save(FILE)?;

  // The same tree, read again. Every field of the descriptor matches, which is exactly why identity is not derived
  // from them: this is a new opening, and the save that began before it no longer speaks for what is shown.
  open_project(&state, &roots)?;

  assert!(matches!(
    state.commit_save(&plan, read_project(&roots)?)?,
    TranslationSaveOutcome::Stale
  ));

  Ok(())
}

#[test]
fn a_save_needs_an_open_project_holding_the_file() -> TauriResult<()> {
  let state: TranslationProjectState = TranslationProjectState::new();
  let roots: XrayRoots = write_project("translations_state/absent", "before")?;

  assert_eq!(state.begin_save(FILE).unwrap_err(), "No translations project is open");

  open_project(&state, &roots)?;

  assert_eq!(
    state.begin_save("st_missing.json").unwrap_err(),
    "Translations file 'st_missing.json' is not part of the open project"
  );

  Ok(())
}
