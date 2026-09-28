/**
 * The textures a surface samples, by the key each was put under.
 */
export interface IRendererSurfaceTextures {
  base?: string;
  detail?: string;
  /** The first half of a bump pair: the packed normal and its gloss. */
  bump?: string;
  /** The second half: the correction and the height. */
  bumpCompanion?: string;
  /** A baked lightmap: hemisphere occlusion in alpha, sun occlusion in green. */
  hemi?: string;
  /** Water's normal map, `s_nmap`, sampled twice as it scrolls. */
  normal?: string;
  /** Water's foam, `s_leaves`, laid where the water is shallow. */
  foam?: string;
  /** Water's distortion, `s_distort`: how far what is behind it is moved. */
  distortion?: string;
}
