/// One mip of the prefiltered sky cube as its pass fills it: its texels a side, the mips before it the pass reads
/// (none for the first, which reads the sky's own cubes), and its six faces as the pass writes them.
pub struct SkyPrefilterLevel {
  pub size: u32,
  pub source: Option<wgpu::TextureView>,
  pub target: wgpu::TextureView,
}
