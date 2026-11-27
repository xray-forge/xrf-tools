use crate::file::config_file::ConfigFile;
use crate::issue::config_issue::ConfigIssue;

#[test]
fn names_its_sections_in_the_files_order() {
  let file: ConfigFile = ConfigFile::parse("misc\\simulation.ltx", "[b]\n[a]\nkey = 1\n").expect("reads");

  assert_eq!(file.get_path(), "misc\\simulation.ltx");
  assert_eq!(file.get_section_names().collect::<Vec<_>>(), vec!["b", "a"]);
  assert!(file.has_section("a") && !file.has_section("c"));
}

#[test]
fn writes_an_issue_with_each_part_it_has() {
  let issue = |section: Option<&str>, key: Option<&str>| {
    ConfigIssue {
      file: String::from("alife.ltx"),
      section: section.map(String::from),
      key: key.map(String::from),
      message: String::from("is missing"),
    }
    .to_string()
  };

  assert_eq!(issue(None, None), "alife.ltx: is missing");
  assert_eq!(issue(Some("alife"), None), "alife.ltx [alife]: is missing");
  assert_eq!(
    issue(Some("alife"), Some("time_factor")),
    "alife.ltx [alife] time_factor: is missing"
  );
}
