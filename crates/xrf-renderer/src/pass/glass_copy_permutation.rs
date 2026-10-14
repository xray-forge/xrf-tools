use xrf_renderer_core::ShaderPermutation;

/// The draw lists the glass copy's dispatch reads, each layout's glass list (`frame/glass_copy_dispatch`).
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct GlassCopyPermutation {
  pub glass_list_baked: u32,
  pub glass_list_tree: u32,
  pub glass_list_model: u32,
}
