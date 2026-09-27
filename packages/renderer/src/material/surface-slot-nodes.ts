import { Texture, TextureNode } from "three/webgpu";

/** A shared slot sampler, and what it samples while no object has pointed it anywhere. */
interface ISurfaceSlotNode {
  node: TextureNode;
  placeholder: Texture;
}

/**
 * Every shared slot sampler built, which holds the texture of the last object it was updated for between objects.
 * Three builds a render object's bindings from what its samplers hold before updating them for it, so a sampler still
 * holding a texture let go would have three upload that texture again, from whatever of it is left.
 */
export class SurfaceSlotNodes {
  private static readonly nodes: Set<ISurfaceSlotNode> = new Set();

  /**
   * @param node - A shared slot sampler.
   * @param placeholder - What it samples while it holds nothing an object pointed it at.
   */
  public static add(node: TextureNode, placeholder: Texture): void {
    SurfaceSlotNodes.nodes.add({ node, placeholder });
  }

  /**
   * Points every shared sampler holding a texture about to be let go back at its placeholder.
   *
   * @param texture - The texture.
   */
  public static forget(texture: Texture): void {
    for (const { node, placeholder } of SurfaceSlotNodes.nodes) {
      if (node.value === texture) {
        node.value = placeholder;
      }
    }
  }
}
