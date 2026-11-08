use xrf_renderer_core::ShaderPermutation;

/// The draw lists the glass copy's dispatch reads, each layout's glass list.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub(crate) struct GlassCopyPermutation {
  pub glass_list_baked: u32,
  pub glass_list_tree: u32,
  pub glass_list_model: u32,
}

impl GlassCopyPermutation {
  /// The module whose pipelines it makes.
  pub(crate) const MODULE: &'static str = "frame/glass_copy_dispatch";
}
