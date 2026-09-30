import { DataUtils } from "three/webgpu";

import { RENDERER_HEMI_CUBE_FLOATS_PER_INSTANCE } from "#/contract/scene/renderer-instances";

/** Words a place's hemisphere cube packs into: its six faces as half floats, two a word. */
export const STATIC_HEMI_CUBE_WORDS: number = RENDERER_HEMI_CUBE_FLOATS_PER_INSTANCE / 2;

/**
 * Packs one place's hemisphere cube as `unpackHalf2x16` reads it back: `(+x, +y)`, `(+z, -x)`, `(-y, -z)`.
 *
 * @param cube - Six floats an instance.
 * @param index - The instance.
 * @param into - Where the three words are written.
 * @param at - Where they start.
 */
export function packStaticHemiCube(cube: Float32Array, index: number, into: Uint32Array, at: number): void {
  const first: number = index * RENDERER_HEMI_CUBE_FLOATS_PER_INSTANCE;

  for (let word = 0; word < STATIC_HEMI_CUBE_WORDS; word += 1) {
    const low: number = DataUtils.toHalfFloat(cube[first + word * 2]);
    const high: number = DataUtils.toHalfFloat(cube[first + word * 2 + 1]);

    into[at + word] = (low | (high << 16)) >>> 0;
  }
}
