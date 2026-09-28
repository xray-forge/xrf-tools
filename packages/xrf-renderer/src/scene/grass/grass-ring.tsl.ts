import { abs, int, select, uint } from "three/tsl";
import { Node, StorageBufferNode } from "three/webgpu";

import { GRASS_CACHE_KEY_WORDS } from "#/scene/grass/grass-cache-buffers";
import { IGrassRingSlot } from "#/scene/grass/grass-ring-slot";
import { GrassRingUniforms } from "#/uniforms/grass-ring-uniforms";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

/**
 * @param ring - How far the ring reaches.
 * @returns Cells the ring holds now: `dm_cache_line` squared.
 */
export function toGrassRingCells(ring: GrassRingUniforms): Node<"int"> {
  const line: Node<"int"> = int(ring.reach).mul(2).add(1);

  return line.mul(line);
}

/**
 * `cache_Update`'s ring: of the slots within reach of the camera's, the one a cell holds is the one whose place in the
 * ring, taken round, is the cell's, so a step of the camera changes only the row or column it steps into.
 *
 * @param uniforms - Where the camera stands.
 * @param ring - How far the ring reaches.
 * @param cell - The cell, `z * line + x`.
 * @returns The world slot it holds now.
 */
export function toGrassRingSlot(uniforms: GrassUniforms, ring: GrassRingUniforms, cell: Node<"int">): IGrassRingSlot {
  const reach: Node<"int"> = int(ring.reach).toVar();
  const line: Node<"int"> = reach.mul(2).add(1).toVar();
  const centerX: Node<"int"> = int(uniforms.center.x).toVar();
  const centerZ: Node<"int"> = int(uniforms.center.y).toVar();
  const x: Node<"int"> = toRingAxis(centerX, reach, line, cell.mod(line)).toVar();
  const z: Node<"int"> = toRingAxis(centerZ, reach, line, cell.div(line)).toVar();

  const acrossX: Node<"int"> = abs(x.sub(centerX));
  const acrossZ: Node<"int"> = abs(z.sub(centerZ));

  return { band: select(acrossX.greaterThan(acrossZ), acrossX, acrossZ).toVar(), x, z };
}

/**
 * @param keys - The ring's keys.
 * @param cell - The cell.
 * @param slot - The world slot it holds now.
 * @param generation - The generation the planting runs under.
 * @returns Whether the cell was planted with that slot under that generation, so what it holds stands.
 */
export function isGrassCellCurrent(
  keys: StorageBufferNode<"uint">,
  cell: Node<"int">,
  slot: IGrassRingSlot,
  generation: Node<"uint">
): Node<"bool"> {
  const key: Node<"uint"> = uint(cell).mul(GRASS_CACHE_KEY_WORDS);

  return keys
    .element(key)
    .equal(uint(slot.x))
    .and(keys.element(key.add(1)).equal(uint(slot.z)))
    .and(keys.element(key.add(2)).equal(generation));
}

/** One axis of the ring: the slot `first + ((cell - first) mod line)`, wrapped for a negative `first`. */
function toRingAxis(center: Node<"int">, reach: Node<"int">, line: Node<"int">, cell: Node<"int">): Node<"int"> {
  const first: Node<"int"> = center.sub(reach);

  return first.add(cell.sub(first).mod(line).add(line).mod(line));
}
