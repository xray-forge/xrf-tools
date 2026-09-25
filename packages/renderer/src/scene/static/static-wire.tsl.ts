import { Fn, instanceIndex, storage } from "three/tsl";
import { ComputeNode, IndirectStorageBufferAttribute } from "three/webgpu";

import { STATIC_DRAW_ARGUMENTS } from "#/uniforms/static-draw-buffers";

/**
 * Rewrites a phase's arguments as its wireframe draw's, one invocation a slot: the count and the first index doubled
 * into the arenas' line indices, which hold two indices for every triangle index, and the rest as the cull left them.
 *
 * @param from - The phase's arguments, as its cull wrote them.
 * @param to - Its wireframe draw's.
 * @param slots - Slots the buffers hold.
 * @returns The compute pass.
 */
export function createWireArgumentsShader(
  from: IndirectStorageBufferAttribute,
  to: IndirectStorageBufferAttribute,
  slots: number
): ComputeNode {
  const source = storage(from, "uint", slots * STATIC_DRAW_ARGUMENTS).toReadOnly();
  const target = storage(to, "uint", slots * STATIC_DRAW_ARGUMENTS);

  return Fn(() => {
    const at = instanceIndex.mul(STATIC_DRAW_ARGUMENTS);

    target.element(at).assign(source.element(at).mul(2));
    target.element(at.add(1)).assign(source.element(at.add(1)));
    target.element(at.add(2)).assign(source.element(at.add(2)).mul(2));
    target.element(at.add(3)).assign(source.element(at.add(3)));
    target.element(at.add(4)).assign(source.element(at.add(4)));
  })().compute(slots);
}
