use xrf_error::XrfResult;

use crate::element::DialogElementKind;
use crate::file::DialogFile;
use crate::project::descriptor::{DialogDescriptor, DialogPhraseDescriptor};

/// A dialog exercising what the flat descriptors have to survive: repeated elements, an ordered
/// `next` list, a scripted line with no translation key, and an element the schema does not define.
const DIALOG: &str = r#"<game_dialogs>
  <dialog id="escape_trader" priority="3">
    <precondition>xr_conditions.actor_has_pda</precondition>
    <init_func>dialogs.init_trader</init_func>
    <has_info>escape_trader_met</has_info>
    <has_info>escape_quest_started</has_info>
    <phrase_list>
      <phrase id="0">
        <text>escape_trader_0</text>
        <next>1</next>
        <next>2</next>
        <next>3</next>
      </phrase>
      <phrase id="1">
        <script_text>dialogs.trader_price</script_text>
        <give_info>trader_price_asked</give_info>
        <disable_info>trader_greeting_pending</disable_info>
        <go_back>1</go_back>
      </phrase>
      <phrase id="2">
        <text>escape_trader_2</text>
        <is_final>1</is_final>
      </phrase>
    </phrase_list>
    <phrase id="3">
      <text>escape_trader_3</text>
    </phrase>
  </dialog>
</game_dialogs>"#;

/// Describe the fixture with no text tree behind it, which is what these tests are about: the shape
/// of the projection, not the resolution. Resolution has its own module.
fn describe() -> XrfResult<DialogDescriptor> {
  let file: DialogFile = DialogFile::read_from_bytes(DIALOG.as_bytes())?;
  let dialog = file.find_dialog("escape_trader").expect("the fixture declares it");

  Ok(DialogDescriptor::new(
    r"configs\gameplay\dialogs_escape.xml",
    dialog,
    None,
  ))
}

fn phrase(descriptor: &DialogDescriptor, id: &str) -> DialogPhraseDescriptor {
  descriptor
    .phrases
    .iter()
    .find(|phrase| phrase.id == id)
    .unwrap_or_else(|| panic!("the fixture declares phrase {id}"))
    .clone()
}

#[test]
fn carries_the_dialog_identity_and_its_own_elements() -> XrfResult {
  let descriptor: DialogDescriptor = describe()?;

  assert_eq!(descriptor.logical_path, r"configs\gameplay\dialogs_escape.xml");
  assert_eq!(descriptor.id, "escape_trader");
  assert_eq!(descriptor.priority, Some(3));

  // Dialog-level elements only. The phrases are a separate field, not four more elements.
  let names: Vec<&str> = descriptor.elements.iter().map(|it| it.name.as_str()).collect();

  assert_eq!(names, ["precondition", "init_func", "has_info", "has_info"]);

  Ok(())
}

#[test]
fn keeps_repeated_elements_as_repeated_entries() -> XrfResult {
  // The whole reason elements are a list: a set would silently drop the second info gate.
  let descriptor: DialogDescriptor = describe()?;
  let gates: Vec<&str> = descriptor
    .elements
    .iter()
    .filter(|it| it.kind == DialogElementKind::HasInfo)
    .map(|it| it.value.as_str())
    .collect();

  assert_eq!(gates, ["escape_trader_met", "escape_quest_started"]);

  Ok(())
}

#[test]
fn preserves_next_order_because_it_is_what_the_player_sees() -> XrfResult {
  let descriptor: DialogDescriptor = describe()?;

  assert_eq!(phrase(&descriptor, "0").next, ["1", "2", "3"]);
  // A terminal phrase offers nothing, rather than reporting an absent list.
  assert_eq!(phrase(&descriptor, "2").next, Vec::<String>::new());

  Ok(())
}

#[test]
fn projects_text_and_final_without_dropping_the_elements_they_came_from() -> XrfResult {
  let descriptor: DialogDescriptor = describe()?;
  let entry: DialogPhraseDescriptor = phrase(&descriptor, "0");

  assert_eq!(entry.text_key.as_deref(), Some("escape_trader_0"));
  assert!(!entry.is_final);

  // The projection is a convenience, not a replacement: `next` is still readable as elements.
  let names: Vec<&str> = entry.elements.iter().map(|it| it.name.as_str()).collect();

  assert_eq!(names, ["text", "next", "next", "next"]);

  assert!(phrase(&descriptor, "2").is_final);

  Ok(())
}

#[test]
fn reports_a_scripted_line_as_having_no_key_rather_than_a_missing_one() -> XrfResult {
  // Anomaly does this 107 times, so absent text is a state and not a defect.
  let descriptor: DialogDescriptor = describe()?;
  let entry: DialogPhraseDescriptor = phrase(&descriptor, "1");

  assert_eq!(entry.text_key, None);
  assert!(
    entry
      .elements
      .iter()
      .any(|it| it.kind == DialogElementKind::ScriptText && it.value == "dialogs.trader_price")
  );

  Ok(())
}

#[test]
fn keeps_an_off_schema_element_under_its_written_name() -> XrfResult {
  // `go_back` is not an engine element, but one shipped project writes it. Reclassifying it as
  // nothing would lose it; renaming it would corrupt a later rewrite.
  let descriptor: DialogDescriptor = describe()?;
  let unknown = phrase(&descriptor, "1")
    .elements
    .into_iter()
    .find(|it| it.kind == DialogElementKind::Unknown)
    .expect("the fixture writes one");

  assert_eq!(unknown.name, "go_back");
  assert_eq!(unknown.value, "1");

  Ok(())
}

#[test]
fn records_whether_a_phrase_was_written_inside_a_phrase_list() -> XrfResult {
  // Both forms occur in shipped data, and an insertion has to reproduce the one already in use.
  let descriptor: DialogDescriptor = describe()?;

  assert!(phrase(&descriptor, "0").is_in_phrase_list);
  assert!(!phrase(&descriptor, "3").is_in_phrase_list);

  Ok(())
}

#[test]
fn serializes_element_kinds_as_camel_case() -> XrfResult {
  // The wire contract the frontend switches on; `ScriptText` reaching TypeScript as `ScriptText`
  // would not match the generated union.
  let descriptor: DialogDescriptor = describe()?;
  let json: String = serde_json::to_string(&phrase(&descriptor, "1"))?;

  assert!(json.contains(r#""kind":"scriptText""#), "unexpected payload: {json}");
  assert!(json.contains(r#""kind":"giveInfo""#), "unexpected payload: {json}");
  assert!(json.contains(r#""isInPhraseList":true"#), "unexpected payload: {json}");

  Ok(())
}
