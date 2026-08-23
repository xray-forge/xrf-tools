/**
 * Names a mip chain by what it means rather than by its count.
 *
 * A single level is not "1 mip" but the absence of a chain, which is the state worth spotting: such a texture has to be
 * sampled with a linear filter or webgl renders it black, and most of Anomaly's textures ship without one.
 *
 * @param levels - Levels the DDS header declares, one meaning no chain at all.
 * @returns Human readable description of the chain.
 */
export function formatMipmapLevels(levels: number): string {
  return levels > 1 ? `${levels} mips` : "no mips";
}
