use xrf_build_info::BuildKind;

use crate::window::window_build_kind::WindowBuildKind;

#[test]
fn the_document_is_told_the_kind_under_its_serialized_name() {
  for (kind, name) in [
    (BuildKind::Local, "local"),
    (BuildKind::Development, "development"),
    (BuildKind::Optimized, "optimized"),
  ] {
    assert_eq!(
      WindowBuildKind::get_script(kind),
      format!("Object.defineProperty(window, \"__XRF_BUILD_KIND__\", {{ value: \"{name}\" }});")
    );
  }
}
