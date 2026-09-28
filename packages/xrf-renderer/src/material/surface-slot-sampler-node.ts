import { Maybe, Nullable } from "@xrf/types";
import { Node, NodeBuilder, NodeFrame, NodeUpdateType, Texture, TextureNode } from "three/webgpu";

import { ESurfaceSlot } from "#/material/surface-slot";
import { ISurfaceSlotted } from "#/material/surface-slotted";

/**
 * A sampler of one slot of whatever material draws: many materials share one node graph, and each object drawn samples
 * what its own material holds for the slot. Three updates it for every render object refreshed, before its bindings,
 * and a clone - biased, or at a level - writes the texture through to the node it was cloned from.
 */
export abstract class SurfaceSlotSamplerNode extends TextureNode {
  public slot: ESurfaceSlot = ESurfaceSlot.BASE;
  /** What it samples while no object has pointed it at anything. */
  public placeholder: Nullable<Texture> = null;

  public constructor(value?: Texture, uvNode?: Nullable<Node>, levelNode?: Nullable<Node>, biasNode?: Nullable<Node>) {
    super(value, uvNode, levelNode, biasNode);

    this.placeholder = value ?? null;

    // Updated for every object: three reads the property as it builds, and a sampler's own setup settles it to none.
    Object.defineProperty(this, "updateType", { get: () => NodeUpdateType.OBJECT, set: () => {} });
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
    const texture: Maybe<Texture> = material ? this.resolve(material) : undefined;

    if (texture) {
      this.value = texture;
    }

    return super.update(frame);
  }

  public override clone(): this {
    const node: this = super.clone();

    node.slot = this.slot;
    node.placeholder = this.placeholder;

    return node;
  }

  /**
   * @param material - The material of the object drawn.
   * @returns What it holds for the slot, or nothing where it holds nothing this sampler reads.
   */
  protected abstract resolve(material: ISurfaceSlotted): Maybe<Texture>;
}
