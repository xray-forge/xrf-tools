import { Maybe, Nullable } from "@xrf/types";
import { Node, NodeBuilder, NodeFrame, NodeUpdateType, Texture, TextureNode } from "three/webgpu";

import { ESurfaceSlot, ISurfaceSlotted } from "#/material/surface-slot";
import { SurfaceSlotNodes } from "#/material/surface-slot-nodes";
import { ITextureTarget } from "#/texture/texture-target";

/**
 * A sampler of one array slot of whatever shared material draws: the array its material binds for the slot, sampled at
 * the layer the surface's row names. Three updates it for every render object refreshed, as it does a slot's.
 */
export class SurfaceSlotArrayNode extends TextureNode {
  public slot: ESurfaceSlot = ESurfaceSlot.HEMI;
  /** What it samples while no object has pointed it at anything. */
  public placeholder: Nullable<Texture> = null;

  public constructor(value?: Texture, uvNode?: Nullable<Node>, levelNode?: Nullable<Node>, biasNode?: Nullable<Node>) {
    super(value, uvNode, levelNode, biasNode);

    // A clone is made empty, then given what it was cloned from, placeholder and all.
    if (value) {
      this.placeholder = value;
      SurfaceSlotNodes.add(this, value);
    }

    // Updated for every object: three reads the property as it builds, and a sampler's own setup settles it to none.
    Object.defineProperty(this, "updateType", { get: () => NodeUpdateType.OBJECT, set: () => {} });
  }

  // A binding a slot, which every sampler of the slot shares, as a slot's own sampler is.
  public override getUniformHash(): string {
    return `surface-array:${this.slot}`;
  }

  // Built holding its placeholder: a program's bindings keep what its samplers held as it was built, and copy it into
  // every render object made from them, which three uploads before updating the sampler for the object. A texture let
  // go since would be uploaded again from whatever of it is left.
  public override setup(builder: NodeBuilder): ReturnType<TextureNode["setup"]> {
    if (this.placeholder) {
      this.value = this.placeholder;
    }

    return super.setup(builder);
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
    node.placeholder = this.placeholder;

    if (this.placeholder) {
      SurfaceSlotNodes.add(node, this.placeholder);
    }

    return node;
  }
}
