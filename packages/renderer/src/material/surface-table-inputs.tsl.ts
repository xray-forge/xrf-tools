import { Nullable } from "@xrf/types";
import { bitcast, nodeObject, vec3 } from "three/tsl";
import { Node, TextureNode } from "three/webgpu";

import { ISurfaceInputs } from "#/material/surface-inputs";
import { toSurfaceInputs } from "#/material/surface-inputs.tsl";
import { ESurfaceSlot, SURFACE_SLOTS } from "#/material/surface-slot";
import { SurfaceSlotArrayNode } from "#/material/surface-slot-array-node";
import { SurfaceSlotNodes } from "#/material/surface-slot-nodes";
import { toSurfaceRow } from "#/shader/surface-row.tsl";
import { getPlaceholderArrayTexture } from "#/texture/placeholder-textures";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { SURFACE_TABLE_LAYER_WORD, SURFACE_TABLE_WORDS, SurfaceTable } from "#/uniforms/surface-table";

/** Components of one of the table's elements, by a word's place in it. */
const COMPONENTS: ReadonlyArray<"x" | "y" | "z" | "w"> = ["x", "y", "z", "w"];

/**
 * What a static batch's shared shader reads per surface: its numbers from the surface table's row its draw's slot
 * names, the slots in `arrayed` from the arrays its material binds at the row's layers, and every other slot as a
 * surface's own material samples it.
 *
 * @param bias - The mip levels every sample is moved by, or null for a material drawing no screen pixels.
 * @param table - The surface table.
 * @param buffers - What static draws are placed by, whose slots name their rows.
 * @param arrayed - The slots sampled from arrays.
 * @param nodes - The renderer's shared samplers, which each sampler built joins.
 * @returns The inputs.
 */
export function toTabledSurfaceInputs(
  bias: Nullable<Node<"float">>,
  table: SurfaceTable,
  buffers: StaticDrawBuffers,
  arrayed: ReadonlyArray<ESurfaceSlot>,
  nodes: SurfaceSlotNodes
): ISurfaceInputs {
  const own: ISurfaceInputs = toSurfaceInputs(bias, nodes);
  const row: Node<"uint"> = toSurfaceRow(buffers);

  function toWord(word: number): Node<"uint"> {
    const element = table.words.element(row.mul(SURFACE_TABLE_WORDS / 4).add(Math.floor(word / 4)));

    return (element as unknown as Record<string, Node<"uint">>)[COMPONENTS[word % 4]];
  }

  function toNumber(word: number): Node<"float"> {
    return bitcast(toWord(word), "float") as unknown as Node<"float">;
  }

  return {
    alphaReference: toNumber(2),
    color: vec3(toNumber(4), toNumber(5), toNumber(6)),
    detailScale: toNumber(1),
    sample: (slot: ESurfaceSlot, coordinates: Node<"vec2">, isBiased: boolean = true): TextureNode => {
      if (!arrayed.includes(slot)) {
        return own.sample(slot, coordinates, isBiased);
      }

      const sampler: SurfaceSlotArrayNode = new SurfaceSlotArrayNode(getPlaceholderArrayTexture(), coordinates);

      sampler.slot = slot;
      nodes.add(sampler);

      const layered: TextureNode = (nodeObject(sampler) as unknown as TextureNode).depth(
        toWord(SURFACE_TABLE_LAYER_WORD + SURFACE_SLOTS.indexOf(slot))
      );

      return bias && isBiased ? layered.bias(bias) : layered;
    },
    slice: toNumber(3),
    tiling: toNumber(0),
    unbiased: (): ISurfaceInputs => toTabledSurfaceInputs(null, table, buffers, arrayed, nodes),
  };
}
