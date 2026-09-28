import { Node, TextureNode } from "three/webgpu";

import { ESurfaceSlot } from "#/material/surface-slot";

/**
 * What a surface variant's shader reads of the material drawing, per object: its slots and its numbers.
 */
export interface ISurfaceInputs {
  /**
   * @param slot - The slot sampled.
   * @param coordinates - Where.
   * @param isBiased - Whether the frame's texture bias moves it: a sampler read at a level of its own takes none, since
   *   three prefers a bias to a level.
   * @returns The sampler.
   */
  sample(slot: ESurfaceSlot, coordinates: Node<"vec2">, isBiased?: boolean): TextureNode;
  /** The same inputs without the bias, for shadow views: they draw at their own resolution. */
  unbiased(): ISurfaceInputs;
  tiling: Node<"float">;
  detailScale: Node<"float">;
  alphaReference: Node<"float">;
  slice: Node<"float">;
  color: Node<"vec3">;
}
