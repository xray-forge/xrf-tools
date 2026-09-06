import { ITextureBumpTexels } from "@/core/textures/lib/texture-surface";
import { decodeXrayBumpTexel, IVisualBumpTexel } from "@/core/visuals/lib/visual-bump";
import { readVisualTexel } from "@/core/visuals/lib/visual-texture";

/** How many digits a reconstructed value is shown to, which is enough to see a quantisation step. */
const DECIMALS: number = 3;

/** Which texel is being read, in the file's own coordinates. */
export interface ITextureTexelPosition {
  x: number;
  y: number;
}

/** What one texel of a pair holds, in the two vocabularies a person needs at once. */
export interface ITextureTexelReadout {
  position: string;
  /** The bump's four bytes as stored. */
  bump: string;
  /** The companion's four bytes as stored. */
  companion: string;
  /** The tangent-space normal the engine reconstructs, signed. */
  normal: string;
  gloss: string;
  height: string;
}

/**
 * Reads one texel of a bump pair and states both what is stored and what the engine makes of it.
 *
 * The decode is {@link decodeXrayBumpTexel}, which is the same expression the shader substitutes, so a readout can be
 * checked against a tile and a tile against the lit surface without a third answer entering the argument.
 *
 * @param texels - Both halves on the cpu.
 * @param position - Texel to read, in the file's own coordinates.
 * @returns The row values, already worded.
 */
export function describeTextureTexel(
  texels: ITextureBumpTexels,
  position: ITextureTexelPosition
): ITextureTexelReadout {
  const bump: [number, number, number, number] = readVisualTexel(texels.bump, position.x, position.y);
  const companion: [number, number, number, number] = readVisualTexel(texels.companion, position.x, position.y);
  const decoded: IVisualBumpTexel = decodeXrayBumpTexel(bump, companion);

  return {
    bump: describeBytes(bump),
    companion: describeBytes(companion),
    gloss: decoded.gloss.toFixed(DECIMALS),
    height: decoded.height.toFixed(DECIMALS),
    normal: decoded.normal.map((it: number) => it.toFixed(DECIMALS)).join(", "),
    position: `${position.x}, ${position.y} of ${texels.bump.width} x ${texels.bump.height}`,
  };
}

/**
 * Which texel a pointer is over, from where it fell on a tile.
 *
 * The tile draws the whole plane, so the fraction across it is the fraction across the file. The last column and row
 * are reachable because a pointer exactly on the far edge would otherwise index past the end.
 *
 * @param texels - Both halves on the cpu, for the size the tile is showing.
 * @param across - How far across the tile the pointer is, from 0 at the left.
 * @param down - How far down the tile the pointer is, from 0 at the top.
 * @returns The texel under the pointer.
 */
export function toTextureTexelPosition(
  texels: ITextureBumpTexels,
  across: number,
  down: number
): ITextureTexelPosition {
  return {
    x: clamp(Math.floor(across * texels.bump.width), texels.bump.width),
    y: clamp(Math.floor(down * texels.bump.height), texels.bump.height),
  };
}

/** One texel's channels as the bytes a file holds, which is how a packer's output is read. */
function describeBytes(texel: ReadonlyArray<number>): string {
  return texel.map((it: number) => Math.round(it * 255)).join(", ");
}

/** Keeps an index inside a plane. */
function clamp(value: number, size: number): number {
  return Math.min(Math.max(value, 0), size - 1);
}
