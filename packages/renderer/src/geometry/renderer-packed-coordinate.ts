import { IRendererPackedVertices } from "#/contract/scene/renderer-geometry";

/** What a baked coordinate's shorts are divided by: `unpack_tc_base` scales by `32 / 32768` (`common_functions.h`). */
export const PACKED_BASE_QUANT: number = 1024;

/** The same for a tree's: `FTreeVisual_quant`, `32768 / 16` (`FTreeVisual.h`), which `consts` carries inverted. */
export const PACKED_TREE_QUANT: number = 2048;

/** The same for a lightmap coordinate: `unpack_tc_lmap` scales by `1 / 32768`. */
export const PACKED_LIGHTMAP_QUANT: number = 32768;

/** Shorts a tree's coordinate takes a vertex: `SHORT4`, its last two the wind terms. */
export const PACKED_TREE_COMPONENTS: number = 4;

/** Where a packed direction carries the low byte of a coordinate: its fourth byte. */
const FRACTION_BYTE: number = 3;

/**
 * One vertex's base coordinate from the engine's packed vertex, as the renderer's shader decodes it: a baked one its
 * shorts plus the low bytes the tangent and binormal carry, over 1024; a tree's its shorts alone, over 2048.
 *
 * @param packed - The packed vertices, carrying a base coordinate.
 * @param components - Shorts the coordinate takes a vertex: two, or a tree's four.
 * @param vertex - Which vertex.
 * @returns Its `u` and `v`.
 */
export function toPackedCoordinate(
  packed: Pick<IRendererPackedVertices, "uv" | "tangent" | "binormal">,
  components: number,
  vertex: number
): [number, number] {
  const uv: Int16Array = packed.uv as Int16Array;

  if (components === PACKED_TREE_COMPONENTS) {
    return [uv[vertex * 4] / PACKED_TREE_QUANT, uv[vertex * 4 + 1] / PACKED_TREE_QUANT];
  }

  const fractionU: number = packed.tangent ? packed.tangent[vertex * 4 + FRACTION_BYTE] / 255 : 0;
  const fractionV: number = packed.binormal ? packed.binormal[vertex * 4 + FRACTION_BYTE] / 255 : 0;

  return [(uv[vertex * 2] + fractionU) / PACKED_BASE_QUANT, (uv[vertex * 2 + 1] + fractionV) / PACKED_BASE_QUANT];
}
