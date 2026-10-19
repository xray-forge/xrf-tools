use std::collections::BTreeSet;

use xrf_renderer_core::ShaderPermutation;

use crate::pass::contact_march_permutation::ContactMarchPermutation;
use crate::pass::glass_copy_permutation::GlassCopyPermutation;
use crate::pass::static_cull_permutation::StaticCullPermutation;
use crate::pass::static_draw_permutation::StaticDrawPermutation;
use crate::pass::sun_filter_permutation::SunFilterPermutation;
use crate::shader::shader_library::ShaderLibrary;

/// Every permutation names override constants the modules it makes pipelines of declare, of the types its fields give
/// them: one a module misnames or retypes is refused only when a GPU makes the pipeline.
#[test]
fn every_permutation_matches_its_modules_overrides() {
  let library: ShaderLibrary = ShaderLibrary::default();
  let permutations: [(&str, String); 6] = [
    of::<SunFilterPermutation>(SunFilterPermutation::MODULE),
    of::<ContactMarchPermutation>(ContactMarchPermutation::LIGHTS_MODULE),
    of::<ContactMarchPermutation>(ContactMarchPermutation::VISIBILITY_MODULE),
    of::<GlassCopyPermutation>(GlassCopyPermutation::MODULE),
    of::<StaticCullPermutation>(StaticCullPermutation::MODULE),
    of::<StaticDrawPermutation>(StaticDrawPermutation::MODULE),
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

/// A module a permutation makes pipelines of, and the override constants the permutation sets.
fn of<P: ShaderPermutation>(module: &'static str) -> (&'static str, String) {
  (module, P::get_wgsl_overrides())
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
