import { AssetTextureShape } from "@/core/ipc/types/xrf-app";

/**
 * Names a mip chain by what it means rather than by its count.
 *
 * @param levels - Levels the DDS header declares, one meaning no chain at all.
 * @returns Human readable description of the chain.
 */
export function formatMipmapLevels(levels: number): string {
  return levels > 1 ? `${levels} mips` : "no mips";
}

/**
 * What a texture file is, in one line: how large, in what layout, with what chain.
 *
 * @param shape - Pixel layout the DDS header declares.
 * @returns The caption.
 */
export function describeTextureShape(shape: AssetTextureShape): string {
  return `${shape.width} x ${shape.height} · ${shape.format} · ${formatMipmapLevels(shape.mipmapLevels)}`;
}
