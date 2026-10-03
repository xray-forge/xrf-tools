//! A small gamedata tree the dialog commands' tests read.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_error::XrfResult;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::core::command_testing::run_command_for_result;
use crate::core::generic_command::{CommandResult, GenericCommand};

const DIALOGS: &str = r#"<?xml version="1.0" encoding="windows-1251"?>
<game_dialogs>
  <dialog id="snag_cache_dialog">
    <dont_has_info>snag_cache_known</dont_has_info>
    <phrase_list>
      <phrase id="0"><text>snag_cache_0</text><next>1</next></phrase>
      <phrase id="1"><text>snag_cache_1</text><give_info>snag_cache_known</give_info><action>dialogs.give_task</action></phrase>
    </phrase_list>
  </dialog>
  <dialog id="snag_share_dialog" priority="5">
    <has_info>snag_cache_known</has_info>
    <phrase_list><phrase id="0"><text>snag_share_0</text></phrase></phrase_list>
  </dialog>
  <dialog id="actor_break_dialog">
    <phrase_list><phrase id="0"><text>actor_break_0</text></phrase></phrase_list>
  </dialog>
</game_dialogs>
"#;

/// A later file declaring a dialog the first one already does, as a mod overlay would.
const OVERLAY: &str = r#"<game_dialogs><dialog id="actor_break_dialog"><phrase_list><phrase id="0"><text>overlay_0</text></phrase></phrase_list></dialog></game_dialogs>"#;

const CHARACTERS: &str = r#"<xml>
  <specific_character id="snag">
    <class>snag</class>
    <start_dialog>snag_cache_dialog</start_dialog>
    <actor_dialog>snag_share_dialog</actor_dialog>
    <actor_dialog>snag_missing_dialog</actor_dialog>
  </specific_character>
</xml>
"#;

const PROFILES: &str = r#"<xml><character id="snag"><class>snag</class></character></xml>"#;

const INFO_PORTIONS: &str = r#"<game_information_portions>
  <info_portion id="global_dialogs"><dialog>actor_break_dialog</dialog></info_portion>
</game_information_portions>
"#;

const STRINGS: &str = r#"<string_table>
  <string id="snag_cache_0"><text>Want to earn some?</text></string>
  <string id="snag_cache_1"><text>Deal.</text></string>
</string_table>
"#;

/// Lay out the tree under a root of its own, since tests run in parallel.
pub fn create_tree(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("dialog_commands/{name}"));
  let gameplay: PathBuf = root.join("configs").join("gameplay");
  let text: PathBuf = root.join("configs").join("text").join("eng");

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&gameplay)?;
  fs::create_dir_all(&text)?;

  fs::write(gameplay.join("dialogs.xml"), DIALOGS)?;
  fs::write(gameplay.join("dialogs_overlay.xml"), OVERLAY)?;
  fs::write(gameplay.join("character_desc_zaton.xml"), CHARACTERS)?;
  fs::write(gameplay.join("npc_profile.xml"), PROFILES)?;
  fs::write(gameplay.join("info_portions.xml"), INFO_PORTIONS)?;
  fs::write(text.join("st_dialogs.xml"), STRINGS)?;

  Ok(root)
}

/// Run a dialog command over a tree, as the process would, and read back the result it deposited.
pub fn run_for_result<T: GenericCommand>(command: &T, root: &Path, extra: &[&str]) -> CommandResult<Value> {
  let mut arguments: Vec<String> = vec![
    String::from(command.operation()),
    String::from("--path"),
    root.display().to_string(),
    String::from("--source"),
    String::from("directory"),
    String::from("--silent"),
    String::from("--json"),
  ];

  arguments.extend(extra.iter().map(|it| String::from(*it)));

  Ok(run_command_for_result(command, &arguments)?.expect("Expected the command to deposit a result"))
}
