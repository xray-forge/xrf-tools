use std::collections::BTreeMap;

use xrf_error::{XrfError, XrfResult};

use crate::shader::shader_composer::compose_shader;

fn compose(modules: &[(&'static str, &'static str)], entry: &str) -> XrfResult<String> {
  let modules: BTreeMap<&str, &'static str> = modules.iter().copied().collect();

  compose_shader(entry, &|name: &str| {
    modules
      .get(name)
      .copied()
      .ok_or_else(|| XrfError::new_not_found_error(format!("No shader module '{name}'")))
  })
}

#[test]
fn inlines_each_import_once() {
  let source: String = compose(
    &[
      ("a", "#import \"common\"\n#import \"b\"\nfn a() {}"),
      ("b", "#import \"common\"\nfn b() {}"),
      ("common", "const C: f32 = 1.0;"),
    ],
    "a",
  )
  .unwrap();

  assert_eq!(source, "const C: f32 = 1.0;\nfn b() {}\nfn a() {}\n");
}

#[test]
fn gathers_enables_to_the_top_once() {
  let source: String = compose(
    &[
      ("a", "enable x;\n#import \"b\"\nfn a() {}"),
      ("b", "enable x;\nenable y;\nfn b() {}"),
    ],
    "a",
  )
  .unwrap();

  assert_eq!(source, "enable x;\nenable y;\nfn b() {}\nfn a() {}\n");
}

#[test]
fn rejects_what_it_cannot_resolve() {
  assert!(compose(&[("a", "#import \"missing\"")], "a").is_err());
  assert!(compose(&[("a", "#if A\nx")], "a").is_err());
}
