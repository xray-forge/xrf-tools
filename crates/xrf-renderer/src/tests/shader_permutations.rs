use std::collections::BTreeSet;

use xrf_renderer_core::ShaderPermutation;

use crate::pass::ambient_occlusion_permutation::AmbientOcclusionPermutation;
use crate::pass::contact_march_permutation::ContactMarchPermutation;
use crate::pass::glass_copy_permutation::GlassCopyPermutation;
use crate::pass::rtao_permutation::RtaoPermutation;
use crate::pass::static_cull_permutation::StaticCullPermutation;
use crate::pass::static_draw_permutation::StaticDrawPermutation;
use crate::pass::sun_filter_permutation::SunFilterPermutation;
use crate::pass::vbao_permutation::VbaoPermutation;
use crate::shader::shader_library::ShaderLibrary;

/// Every permutation names override constants its module declares, of the types its fields give them: one a module
/// misnames or retypes is refused only when a GPU makes the pipeline.
#[test]
fn every_permutation_matches_its_modules_overrides() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let permutations: [(&str, String); 9] = [
    ("frame/sun", SunFilterPermutation::get_wgsl_overrides()),
    (
      "frame/ambient_occlusion",
      AmbientOcclusionPermutation::get_wgsl_overrides(),
    ),
    ("frame/vbao", VbaoPermutation::get_wgsl_overrides()),
    ("frame/rtao", RtaoPermutation::get_wgsl_overrides()),
    ("frame/lights", ContactMarchPermutation::get_wgsl_overrides()),
    ("frame/light_visibility", ContactMarchPermutation::get_wgsl_overrides()),
    ("frame/glass_copy_dispatch", GlassCopyPermutation::get_wgsl_overrides()),
    ("static/cull", StaticCullPermutation::get_wgsl_overrides()),
    ("static/gbuffer", StaticDrawPermutation::get_wgsl_overrides()),
  ];

  for (module, overrides) in permutations {
    let declared: BTreeSet<String> = list_overrides(&library, module);

    assert!(!overrides.is_empty(), "'{module}': a permutation of no constants");

    for line in overrides.lines() {
      assert!(
        declared.contains(line),
        "'{module}' does not declare `{line}`: {declared:?}"
      );
    }
  }
}

/// `override NAME: type;` for each override constant a module declares, its initializer left out.
fn list_overrides(library: &ShaderLibrary, name: &str) -> BTreeSet<String> {
  let source: String = library.compose(name).expect("composes");
  let module: naga::Module = naga::front::wgsl::parse_str(&source).expect("parses");

  module
    .overrides
    .iter()
    .map(|(_, declared)| {
      let wgsl_type: &str = match module.types[declared.ty].inner {
        naga::TypeInner::Scalar(naga::Scalar::BOOL) => "bool",
        naga::TypeInner::Scalar(naga::Scalar::U32) => "u32",
        naga::TypeInner::Scalar(naga::Scalar::I32) => "i32",
        naga::TypeInner::Scalar(naga::Scalar::F32) => "f32",
        ref other => panic!("'{name}' declares an override of {other:?}"),
      };

      format!(
        "override {}: {wgsl_type};",
        declared.name.as_deref().expect("a named override")
      )
    })
    .collect()
}
