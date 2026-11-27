use xrf_ltx::Ltx;

use crate::config::config_text::ConfigText;

#[test]
fn splits_names_as_parse_names_does() {
  assert_eq!(
    ConfigText::split_names("sim_default_stalker_0, sim_default_stalker_1,  ,a.b\\c-d"),
    vec!["sim_default_stalker_0", "sim_default_stalker_1", "a.b\\c-d"]
  );
  assert!(ConfigText::split_names(" , ").is_empty());
}

#[test]
fn walks_a_sections_keys_sorted_and_once_as_the_engine_does() {
  let ltx: Ltx = Ltx::read_from_str("[start]\nzeta = a\nalpha = b\nmid = c\n").expect("reads");
  let section = ltx.section("start").expect("the section");

  assert_eq!(ConfigText::sorted_keys(section), vec!["alpha", "mid", "zeta"]);
}
