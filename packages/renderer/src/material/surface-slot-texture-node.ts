import { Maybe, Nullable } from "@xrf/types";
import { Node, NodeFrame, NodeUpdateType, Texture, TextureNode } from "three/webgpu";

import { ESurfaceSlot, ISurfaceSlotted } from "#/material/surface-slot";

/**
 * A sampler of one slot of whatever material draws: many surfaces' materials share one node graph, and each object
 * drawn samples its own material's texture. Three updates it for every render object refreshed, before its bindings,
 * and a clone - biased, or at a level - writes the texture through to the node it was cloned from.
 */
export class SurfaceSlotTextureNode extends TextureNode {
  public slot: ESurfaceSlot = ESurfaceSlot.BASE;

  public constructor(value?: Texture, uvNode?: Nullable<Node>, levelNode?: Nullable<Node>, biasNode?: Nullable<Node>) {
    super(value, uvNode, levelNode, biasNode);

    // Updated for every object: three reads the property as it builds, and a sampler's own setup settles it to none.
    Object.defineProperty(this, "updateType", { get: () => NodeUpdateType.OBJECT, set: () => {} });
  }

  // Three binds the samplers of one texture once, by its uuid: slots holding the same placeholder as they build would
  // share one binding. A slot is its own binding, which every sampler of it shares.
  public override getUniformHash(): string {
    return `surface-slot:${this.slot}`;
  }

  public override update(frame: NodeFrame): boolean | undefined {
    const material: Maybe<ISurfaceSlotted> = frame.material as unknown as Maybe<ISurfaceSlotted>;

    if (material?.surfaceSlots) {
      this.value = material.surfaceSlots[this.slot].value;
    }

    return super.update(frame);
  }

  public override clone(): this {
    const node: this = super.clone();

    node.slot = this.slot;

    return node;
  }
}
