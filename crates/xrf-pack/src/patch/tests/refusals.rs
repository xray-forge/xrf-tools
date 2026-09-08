//! What a run refuses to do, and why each refusal is worth the run it stops.

use std::path::PathBuf;

use crate::patch::config::ArchivePatchConfig;
use crate::patch::tests::fixtures::{BASE_FILES, CONFIG_EDITED, config, create_tree, destination};
use crate::patch::{ArchivePatchOptions, ArchivePatcher};

/// The message a refused run produced.
fn refusal(configured: &ArchivePatchConfig, options: ArchivePatchOptions) -> String {
  ArchivePatcher::patch_opt(configured, options)
    .expect_err("the run is refused")
    .to_string()
}

#[test]
fn a_scope_matching_nothing_is_refused() {
  // The refusal that keeps a release gate from passing vacuously after a directory is renamed.
  let scope: &str = "patch_scope_matching_nothing_is_refused";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", BASE_FILES);
  let mut configured: ArchivePatchConfig = config(&base, &target, &destination(scope));

  configured.include = vec![String::from("renamed")];

  let message: String = refusal(&configured, ArchivePatchOptions::default());

  assert!(
    message.contains("matched no entry"),
    "'{message}' says the scope matched nothing"
  );
}

#[test]
fn strict_fails_a_run_whose_base_holds_what_the_target_does_not() {
  let scope: &str = "patch_strict_fails_on_removals";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);
  let configured: ArchivePatchConfig = config(&base, &target, &destination(scope));
  let message: String = refusal(&configured, ArchivePatchOptions::default().with_strict(true));

  assert!(
    message.contains("cannot express a deletion"),
    "'{message}' says why the patch cannot carry the removal"
  );
}

#[test]
fn removals_alone_do_not_fail_a_run() {
  let scope: &str = "patch_removals_alone_do_not_fail";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);
  let result = ArchivePatcher::compare(&config(&base, &target, &destination(scope))).expect("removals are reported");

  assert_eq!(result.removed.len(), 2, "they are named rather than refused");
}

#[test]
fn a_destination_inside_a_compared_root_is_refused() {
  // `db/patches/` is both where a patch belongs and a directory someone would name as a base root.
  let scope: &str = "patch_destination_inside_a_root_is_refused";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);
  let inside: PathBuf = base.join("patches");
  let message: String = refusal(&config(&base, &target, &inside), ArchivePatchOptions::default());

  assert!(
    message.contains("is inside the base root"),
    "'{message}' names the root it would have been written into"
  );
}

#[test]
fn a_comparison_may_target_a_directory_a_publication_could_not() {
  // The destination guard protects a write. A comparison writes nothing, so it has nothing to protect.
  let scope: &str = "patch_comparison_ignores_the_destination_guard";
  let base: PathBuf = create_tree(scope, "base", BASE_FILES);
  let target: PathBuf = create_tree(scope, "target", &[("configs\\system.ltx", CONFIG_EDITED)]);

  ArchivePatcher::compare(&config(&base, &target, &base.join("patches"))).expect("a comparison has no destination");
}

#[test]
fn an_empty_root_set_is_refused_before_anything_is_mounted() {
  let scope: &str = "patch_empty_root_set_is_refused";
  let target: PathBuf = create_tree(scope, "target", BASE_FILES);
  let configured: ArchivePatchConfig = ArchivePatchConfig::new(Vec::new(), vec![target], destination(scope), "patch");
  let message: String = refusal(&configured, ArchivePatchOptions::default());

  assert!(message.contains("no base root"), "'{message}' names the missing side");
}

#[test]
fn a_root_holding_nothing_is_refused() {
  // An empty world would report every file of the other as a difference, which is not a comparison.
  let scope: &str = "patch_empty_root_is_refused";
  let base: PathBuf = create_tree(scope, "base", &[]);
  let target: PathBuf = create_tree(scope, "target", BASE_FILES);
  let message: String = refusal(
    &config(&base, &target, &destination(scope)),
    ArchivePatchOptions::default(),
  );

  assert!(
    message.contains("Nothing mounted") || message.contains("entry to compare"),
    "'{message}' says the side holds nothing"
  );
}
