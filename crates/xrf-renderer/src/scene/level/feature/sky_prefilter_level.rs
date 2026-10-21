/// One mip of the prefiltered sky cube as its pass fills it: its texels a side, the mips before it the pass reads
/// (none for the first, which reads the sky's own cubes), and its six faces as the pass writes them; then the same of
/// the sky alone's cube (`SkyPrefilter::alone`), whose first mip reads the whole cube's first in place of the mips
/// before it, to leave out what the surroundings cover.
pub struct SkyPrefilterLevel {
  pub size: u32,
  pub source: Option<wgpu::TextureView>,
  pub target: wgpu::TextureView,
  pub alone_source: wgpu::TextureView,
  pub alone_target: wgpu::TextureView,
}
