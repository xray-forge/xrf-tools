use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::project::descriptor::DialogDescriptor;
use crate::project::dialog_project::DialogProject;
use crate::project::layout::DialogProjectLayout;
use crate::project::mode::DialogProjectMode;

/// Two phrases: one with a key the text tree defines, one built from script at runtime.
const DIALOG: &str = r#"<game_dialogs>
  <dialog id="trader">
    <phrase_list>
      <phrase id="0"><text>trader_hello</text><next>1</next></phrase>
      <phrase id="1"><script_text>dialogs.price</script_text></phrase>
      <phrase id="2"><text>trader_absent</text></phrase>
    </phrase_list>
  </dialog>
</game_dialogs>"#;

fn table(entries: &[(&str, &str)]) -> String {
  let rows: String = entries
    .iter()
    .map(|(id, text)| format!("\t<string id=\"{id}\">\n\t\t<text>{text}</text>\n\t</string>\n"))
    .collect();

  format!("<string_table>\n{rows}</string_table>")
}

/// A gamedata tree whose text sits under `configs\text\<language>`, as the engine ships it.
fn create_gamedata(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-dialog-text-{name}-{}", std::process::id()));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  let gameplay: PathBuf = root.join("configs").join("gameplay");
  let english: PathBuf = root.join("configs").join("text").join("eng");
  let russian: PathBuf = root.join("configs").join("text").join("rus");

  fs::create_dir_all(&gameplay)?;
  fs::create_dir_all(&english)?;
  fs::create_dir_all(&russian)?;
  fs::write(gameplay.join("dialogs.xml"), DIALOG)?;
  fs::write(
    english.join("st_dialogs.xml"),
    table(&[("trader_hello", "Hello, stalker")]),
  )?;
  // Deliberately missing `trader_hello`, so one language is behind the other.
  fs::write(russian.join("st_dialogs.xml"), table(&[("other_key", "Other")]))?;

  Ok(root)
}

fn open(root: &Path) -> XrfResult<DialogProject> {
  DialogProject::open(
    &XrayRoots::one(root.to_path_buf(), XrayMountMode::Directory),
    &DialogProjectLayout::new(DialogProjectMode::Gamedata),
  )
}

fn describe(project: &DialogProject, language: Option<&str>) -> DialogDescriptor {
  project
    .describe_dialog(r"configs\gameplay\dialogs.xml", "trader", language)
    .expect("the fixture declares it")
}

#[test]
fn resolves_a_phrase_key_into_the_line_the_player_reads() -> XrfResult {
  let root: PathBuf = create_gamedata("resolves")?;
  let project: DialogProject = open(&root)?;
  let descriptor: DialogDescriptor = describe(&project, Some("eng"));

  assert_eq!(descriptor.language.as_deref(), Some("eng"));
  assert_eq!(descriptor.phrases[0].text_key.as_deref(), Some("trader_hello"));
  assert_eq!(descriptor.phrases[0].text.as_deref(), Some("Hello, stalker"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn leaves_a_scripted_phrase_alone_rather_than_reporting_it_untranslated() -> XrfResult {
  // It has no key, so there is nothing to look up and nothing missing. Anomaly does this 107 times.
  let root: PathBuf = create_gamedata("scripted")?;
  let project: DialogProject = open(&root)?;
  let descriptor: DialogDescriptor = describe(&project, Some("eng"));

  assert_eq!(descriptor.phrases[1].text_key, None);
  assert_eq!(descriptor.phrases[1].text, None);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn keeps_the_key_when_a_language_has_no_line_for_it() -> XrfResult {
  // Untranslated work, not a broken reference. A surface shows the key so a writer can see which.
  let root: PathBuf = create_gamedata("untranslated")?;
  let project: DialogProject = open(&root)?;
  let descriptor: DialogDescriptor = describe(&project, Some("rus"));

  assert_eq!(descriptor.language.as_deref(), Some("rus"));
  assert_eq!(descriptor.phrases[0].text_key.as_deref(), Some("trader_hello"));
  assert_eq!(descriptor.phrases[0].text, None);
  // The tree does define it in another language, which is what tells the two situations apart.
  assert!(project.get_text().contains_key("trader_hello"));
  assert!(!project.get_text().contains_key("trader_absent"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn takes_the_first_language_when_the_caller_names_none() -> XrfResult {
  let root: PathBuf = create_gamedata("default-language")?;
  let project: DialogProject = open(&root)?;

  // Discovery order, which the reader sorts, so this is stable across runs and machines.
  assert_eq!(project.list_languages(), ["eng", "rus"]);
  assert_eq!(describe(&project, None).language.as_deref(), Some("eng"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn resolves_nothing_for_a_language_the_tree_does_not_hold() -> XrfResult {
  // Answering with the default instead would show French text labelled as something else.
  let root: PathBuf = create_gamedata("unknown-language")?;
  let project: DialogProject = open(&root)?;
  let descriptor: DialogDescriptor = describe(&project, Some("fra"));

  assert_eq!(descriptor.language, None);
  assert_eq!(descriptor.phrases[0].text, None);
  // The phrases still describe, so the graph draws with keys rather than failing to open.
  assert_eq!(descriptor.phrases.len(), 3);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn reports_what_the_text_tree_offered_on_the_project() -> XrfResult {
  let root: PathBuf = create_gamedata("descriptor")?;
  let project: DialogProject = open(&root)?;
  let descriptor = project.describe();

  assert_eq!(descriptor.languages, ["eng", "rus"]);
  // `trader_hello` and `other_key`; the tree never defines `trader_absent`.
  assert_eq!(descriptor.text_keys, 2);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn opens_without_a_text_tree_and_says_so() -> XrfResult {
  // A dialogs root whose text sits somewhere this layout does not look. Refusing to open would leave
  // the structure unreadable over text that is not needed to read it.
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-dialog-text-none-{}", std::process::id()));
  let gameplay: PathBuf = root.join("configs").join("gameplay");

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&gameplay)?;
  fs::write(gameplay.join("dialogs.xml"), DIALOG)?;

  let project: DialogProject = open(&root)?;
  let descriptor: DialogDescriptor = describe(&project, None);

  assert!(project.list_languages().is_empty());
  assert_eq!(project.describe().text_keys, 0);
  assert_eq!(descriptor.language, None);
  assert_eq!(descriptor.phrases[0].text_key.as_deref(), Some("trader_hello"));
  assert_eq!(descriptor.phrases[0].text, None);

  fs::remove_dir_all(root)?;

  Ok(())
}
