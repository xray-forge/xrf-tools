import { Texture } from "three/webgpu";

import { SurfaceSlotSamplerNode } from "#/material/surface-slot-sampler-node";

/**
 * One renderer's shared slot samplers, each holding the texture of the last object it was updated for between objects.
 * Three builds a render object's bindings from what its samplers hold before updating them for it, so a sampler still
 * holding a texture let go would have three upload that texture again, from whatever of it is left.
 */
export class SurfaceSlotNodes {
  /** The samplers built, never their clones: a clone reads and writes the node it was cloned from. */
  private readonly nodes: Set<SurfaceSlotSamplerNode> = new Set();

  /**
   * @param node - A shared slot sampler, just built.
   */
  public add(node: SurfaceSlotSamplerNode): void {
    this.nodes.add(node);
  }

  /**
   * Points every shared sampler holding a texture about to be let go back at its placeholder.
   *
   * @param textures - The textures.
   */
  public forget(textures: ReadonlySet<Texture>): void {
    if (!textures.size) {
      return;
    }

    for (const node of this.nodes) {
      if (node.placeholder && textures.has(node.value)) {
        node.value = node.placeholder;
      }
    }
  }

  public clear(): void {
    this.nodes.clear();
  }
}
