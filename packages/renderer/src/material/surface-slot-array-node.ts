import { Maybe, Nullable } from "@xrf/types";
import { Node, NodeFrame, NodeUpdateType, Texture, TextureNode } from "three/webgpu";

import { ESurfaceSlot, ISurfaceSlotted } from "#/material/surface-slot";
import { ITextureTarget } from "#/texture/texture-target";

/**
 * A sampler of one array slot of whatever shared material draws: the array its material binds for the slot, sampled at
 * the layer the surface's row names. Three updates it for every render object refreshed, as it does a slot's.
 */
export class SurfaceSlotArrayNode extends TextureNode {
  public slot: ESurfaceSlot = ESurfaceSlot.HEMI;

  public constructor(value?: Texture, uvNode?: Nullable<Node>, levelNode?: Nullable<Node>, biasNode?: Nullable<Node>) {
    super(value, uvNode, levelNode, biasNode);

    // Updated for every object: three reads the property as it builds, and a sampler's own setup settles it to none.
    Object.defineProperty(this, "updateType", { get: () => NodeUpdateType.OBJECT, set: () => {} });
  }

  // A binding a slot, which every sampler of the slot shares, as a slot's own sampler is.
  public override getUniformHash(): string {
    return `surface-array:${this.slot}`;
  }

  public override update(frame: NodeFrame): boolean | undefined {
    const material: Maybe<ISurfaceSlotted> = frame.material as unknown as Maybe<ISurfaceSlotted>;
    const target: Maybe<ITextureTarget> = material?.surfaceArrays?.[this.slot];

    if (target) {
      this.value = target.value;
    }

    return super.update(frame);
  }

  public override clone(): this {
    const node: this = super.clone();

    node.slot = this.slot;

    return node;
  }
}
