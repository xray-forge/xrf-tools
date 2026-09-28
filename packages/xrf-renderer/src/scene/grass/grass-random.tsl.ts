import { bitAnd, float, shiftRight, uint } from "three/tsl";
import { Node } from "three/webgpu";

/**
 * `CRandom::randI()`: the state advanced, and bits 16 to 30 of it.
 *
 * @param state - The generator, advanced in place.
 * @returns The draw.
 */
export function toGrassRandom(state: Node<"uint">): Node<"uint"> {
  state.assign(state.mul(uint(214013)).add(uint(2531011)));

  return bitAnd(shiftRight(state, uint(16)), uint(0x7fff)).toVar();
}

/**
 * `CRandom::randF()`: `randI() / 32767`.
 *
 * @param state - The generator, advanced in place.
 * @returns The draw, from nought to one.
 */
export function toGrassRandomFloat(state: Node<"uint">): Node<"float"> {
  return float(toGrassRandom(state)).div(32767);
}

/**
 * `CRandom::randFs(range)`: `randF(-range, range)`.
 *
 * @param state - The generator, advanced in place.
 * @param range - How far either way.
 * @returns The draw.
 */
export function toGrassSignedRandom(state: Node<"uint">, range: Node<"float">): Node<"float"> {
  return toGrassRandomFloat(state).mul(range.mul(2)).sub(range);
}
