//! The layers the application's modules depend through, checked over the sources.
//!
//! `core/` references no plugin. An infrastructure plugin references no other plugin. A domain plugin references only
//! infrastructure plugins, never another domain: what two domains share lives below them, in `core/` or a crate. The
//! composition root (`plugins/registry.rs`, `application.rs`, `ipc/`) wires every plugin and is not checked.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The plugins every domain may build on; every other plugin is a domain.
const INFRASTRUCTURE: &[&str] = &["render"];

/// References that break the layers and are known, as (source file, plugin referenced). An entry that no longer occurs
/// fails the test too, so the list only shrinks.
// todo: Empty once paths live in `core/`, render shows no domain's document and the code two domains share moves
// below them.
const KNOWN_BREACHES: &[(&str, &str)] = &[
  ("plugins/levels/spawn_visuals.rs", "visuals"),
  ("plugins/levels/tests/cube_export.rs", "textures"),
  ("plugins/levels/tests/map_export.rs", "textures"),
  ("plugins/levels/texture_resolver.rs", "textures"),
  ("plugins/levels/viewer/commands/copy_screenshot.rs", "system"),
  ("plugins/levels/viewer/commands/delete_screenshot.rs", "system"),
  ("plugins/levels/viewer/commands/export_cube.rs", "system"),
  ("plugins/levels/viewer/commands/export_map.rs", "system"),
  ("plugins/levels/viewer/commands/find_screenshot_path.rs", "system"),
  ("plugins/levels/viewer/commands/get_view_folder.rs", "system"),
  ("plugins/levels/viewer/commands/write_map_annotations.rs", "system"),
  ("plugins/levels/viewer/commands/write_screenshot_report.rs", "system"),
  ("plugins/levels/viewer/cube_export_format.rs", "textures"),
  ("plugins/levels/viewer/map_export.rs", "textures"),
  ("plugins/levels/viewer/map_export_format.rs", "textures"),
  ("plugins/levels/viewer/routes/read_screenshot.rs", "system"),
  ("plugins/render/commands/capture_cube.rs", "levels"),
  ("plugins/render/commands/capture_cube.rs", "system"),
  ("plugins/render/commands/capture_map.rs", "levels"),
  ("plugins/render/commands/capture_map.rs", "system"),
  ("plugins/render/commands/save_screenshot.rs", "levels"),
  ("plugins/render/commands/save_screenshot.rs", "system"),
  ("plugins/render/commands/show_level.rs", "levels"),
  ("plugins/render/commands/show_model.rs", "visuals"),
  ("plugins/render/commands/show_texture.rs", "textures"),
  ("plugins/render/level_scene_follower.rs", "levels"),
  ("plugins/render/plugin.rs", "levels"),
  ("plugins/render/shown_levels.rs", "levels"),
  ("plugins/render/state.rs", "levels"),
  ("plugins/visuals/render_source.rs", "levels"),
];

/// Where a source file sits among the layers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Owner<'a> {
  Core,
  Plugin(&'a str),
}

#[test]
fn every_module_references_only_the_layers_below_it() {
  let source: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
  let mut breaches: BTreeSet<(String, String)> = BTreeSet::new();

  for file in list_sources(&source) {
    let relative: String = file
      .strip_prefix(&source)
      .unwrap_or_else(|_| unreachable!("a listed source lies under the source directory"))
      .components()
      .map(|component| component.as_os_str().to_string_lossy())
      .collect::<Vec<_>>()
      .join("/");

    let Some(owner) = to_owner(&relative) else {
      continue;
    };

    let text: String =
      std::fs::read_to_string(&file).unwrap_or_else(|error| panic!("Cannot read '{relative}': {error}"));

    for plugin in list_referenced_plugins(&text) {
      if !is_allowed(owner, &plugin) {
        breaches.insert((relative.clone(), plugin));
      }
    }
  }

  let known: BTreeSet<(String, String)> = KNOWN_BREACHES
    .iter()
    .map(|(file, plugin)| ((*file).to_owned(), (*plugin).to_owned()))
    .collect();
  let new: Vec<String> = breaches
    .difference(&known)
    .map(|(file, plugin)| format!("  {file} references plugin '{plugin}'"))
    .collect();
  let gone: Vec<String> = known
    .difference(&breaches)
    .map(|(file, plugin)| format!("  {file} no longer references plugin '{plugin}'"))
    .collect();

  assert!(
    new.is_empty() && gone.is_empty(),
    "The application's layers moved.\nReferences across them (move what both need below them, into `core/` or a crate):\n{}\nKnown breaches that are gone (remove them from the list):\n{}",
    new.join("\n"),
    gone.join("\n"),
  );
}

#[test]
fn reads_the_plugins_a_source_references() {
  let text: &str = "use crate::plugins::levels::state::LevelState;\n\
     // crate::plugins::visuals in a comment\n\
     use crate::plugins::{\n  render::state::RenderState,\n  textures::{a, b},\n  self,\n};\n\
     let found = crate::plugins::system::paths::SystemPathsState::new(None);\n\
     use xrf_plugins::other;";

  assert_eq!(
    list_referenced_plugins(text),
    BTreeSet::from(["levels", "render", "system", "textures"].map(String::from))
  );
}

#[test]
fn allows_only_references_down_the_layers() {
  assert!(!is_allowed(Owner::Core, "render"));
  assert!(!is_allowed(Owner::Plugin("render"), "levels"));
  assert!(is_allowed(Owner::Plugin("levels"), "render"));
  assert!(is_allowed(Owner::Plugin("levels"), "levels"));
  assert!(!is_allowed(Owner::Plugin("levels"), "visuals"));
}

/// The layer a source file belongs to, or `None` for the composition root.
fn to_owner(relative: &str) -> Option<Owner<'_>> {
  match relative.split('/').collect::<Vec<&str>>().as_slice() {
    ["core", ..] => Some(Owner::Core),
    ["plugins", "tests", ..] => None,
    ["plugins", plugin, _, ..] => Some(Owner::Plugin(plugin)),
    _ => None,
  }
}

fn is_allowed(owner: Owner<'_>, plugin: &str) -> bool {
  match owner {
    Owner::Core => false,
    Owner::Plugin(own) if own == plugin => true,
    Owner::Plugin(own) if INFRASTRUCTURE.contains(&own) => false,
    Owner::Plugin(_) => INFRASTRUCTURE.contains(&plugin),
  }
}

/// Every plugin a source names through a `plugins::` path, outside line comments; a group names each of its members.
fn list_referenced_plugins(text: &str) -> BTreeSet<String> {
  let code: String = text
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .collect::<Vec<&str>>()
    .join("\n");
  let mut plugins: BTreeSet<String> = BTreeSet::new();

  for (start, _) in code.match_indices("plugins::") {
    let is_path_start: bool = code[..start]
      .chars()
      .next_back()
      .is_none_or(|previous| !(previous.is_alphanumeric() || previous == '_'));

    if !is_path_start {
      continue;
    }

    let rest: &str = &code[start + "plugins::".len()..];

    match rest.strip_prefix('{') {
      Some(group) => plugins.extend(list_group_heads(group)),
      None => plugins.extend(read_identifier(rest)),
    }
  }

  plugins.remove("self");
  plugins
}

/// The first segment of each member of a `{ .. }` group, `group` starting just past its opening brace.
fn list_group_heads(group: &str) -> Vec<String> {
  let mut heads: Vec<String> = Vec::new();
  let mut depth: usize = 0;
  let mut is_member_start: bool = true;

  for (index, character) in group.char_indices() {
    match character {
      '{' => depth += 1,
      '}' if depth == 0 => break,
      '}' => depth -= 1,
      ',' if depth == 0 => is_member_start = true,
      _ if is_member_start && !character.is_whitespace() => {
        heads.extend(read_identifier(&group[index..]));
        is_member_start = false;
      }
      _ => {}
    }
  }

  heads
}

fn read_identifier(text: &str) -> Option<String> {
  let identifier: String = text
    .chars()
    .take_while(|character| character.is_alphanumeric() || *character == '_')
    .collect();

  (!identifier.is_empty()).then_some(identifier)
}

fn list_sources(directory: &Path) -> Vec<PathBuf> {
  let mut sources: Vec<PathBuf> = Vec::new();
  let entries: std::fs::ReadDir =
    std::fs::read_dir(directory).unwrap_or_else(|error| panic!("Cannot list '{}': {error}", directory.display()));

  for entry in entries.flatten() {
    let path: PathBuf = entry.path();

    if path.is_dir() {
      sources.extend(list_sources(&path));
    } else if path.extension().is_some_and(|extension| extension == "rs") {
      sources.push(path);
    }
  }

  sources
}
