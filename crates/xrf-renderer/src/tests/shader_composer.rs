use std::collections::BTreeMap;

use xrf_error::{XrfError, XrfResult};

use crate::shader::shader_composer::compose_shader;

fn compose(modules: &[(&'static str, &'static str)], entry: &str, defines: &[&str]) -> XrfResult<String> {
  let modules: BTreeMap<&str, &'static str> = modules.iter().copied().collect();

  compose_shader(entry, defines, &|name: &str| {
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
    &[],
  )
  .unwrap();

  assert_eq!(source, "const C: f32 = 1.0;\nfn b() {}\nfn a() {}\n");
}

#[test]
fn keeps_the_branch_its_define_names() {
  let modules: &[(&str, &str)] = &[("a", "#if FAST\nfast\n#else\nslow\n#endif\nboth")];

  assert_eq!(compose(modules, "a", &["FAST"]).unwrap(), "fast\nboth\n");
  assert_eq!(compose(modules, "a", &[]).unwrap(), "slow\nboth\n");
}

#[test]
fn skips_an_import_inside_a_dropped_branch() {
  let source: String = compose(&[("a", "#if NEVER\n#import \"missing\"\n#endif\nkept")], "a", &[]).unwrap();

  assert_eq!(source, "kept\n");
}

#[test]
fn nests_branches() {
  let modules: &[(&str, &str)] = &[("a", "#if A\n#if B\nab\n#else\na\n#endif\n#endif")];

  assert_eq!(compose(modules, "a", &["A", "B"]).unwrap(), "ab\n");
  assert_eq!(compose(modules, "a", &["A"]).unwrap(), "a\n");
  assert_eq!(compose(modules, "a", &["B"]).unwrap(), "");
}

#[test]
fn rejects_what_it_cannot_resolve() {
  assert!(compose(&[("a", "#import \"missing\"")], "a", &[]).is_err());
  assert!(compose(&[("a", "#if A\nx")], "a", &[]).is_err());
  assert!(compose(&[("a", "#endif")], "a", &[]).is_err());
  assert!(compose(&[("a", "#define X")], "a", &[]).is_err());
}
