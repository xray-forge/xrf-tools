use crate::EnvFlagValue;

#[test]
fn takes_a_switch_in_any_of_its_spellings_and_case() {
  for raw in ["1", "true", "On", " YES "] {
    assert_eq!(EnvFlagValue::Switch.interpret(raw).as_deref(), Some("on"), "{raw:?}");
  }

  for raw in ["0", "False", "off", "no"] {
    assert_eq!(EnvFlagValue::Switch.interpret(raw).as_deref(), Some("off"), "{raw:?}");
  }

  for raw in ["", "2", "enabled", "of"] {
    assert_eq!(EnvFlagValue::Switch.interpret(raw), None, "{raw:?}");
  }
}

#[test]
fn takes_a_choice_in_any_case_in_the_tables_spelling() {
  let value: EnvFlagValue = EnvFlagValue::Choice(&["d3d12", "vulkan"]);

  assert_eq!(value.interpret("Vulkan").as_deref(), Some("vulkan"));
  assert_eq!(value.interpret(" D3D12 ").as_deref(), Some("d3d12"));
  assert_eq!(value.interpret("dx12"), None);
}

#[test]
fn takes_a_path_or_text_as_set_unless_empty() {
  assert_eq!(EnvFlagValue::Path.interpret(" C:\\x ").as_deref(), Some("C:\\x"));
  assert_eq!(
    EnvFlagValue::Text.interpret("info,wgpu_hal=warn").as_deref(),
    Some("info,wgpu_hal=warn")
  );
  assert_eq!(EnvFlagValue::Path.interpret("  "), None);
}

#[test]
fn describes_a_choice_as_a_list() {
  assert_eq!(EnvFlagValue::Choice(&["a"]).describe(), "`a`");
  assert_eq!(EnvFlagValue::Choice(&["a", "b", "c"]).describe(), "`a`, `b` or `c`");
}
