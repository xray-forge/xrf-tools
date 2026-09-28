import { Maybe } from "@xrf/types";
import { Texture } from "three/webgpu";

import { SurfaceSlotSamplerNode } from "#/material/surface-slot-sampler-node";
import { ISurfaceSlotted } from "#/material/surface-slotted";

/**
 * A sampler of one array slot of whatever shared material draws: the array its material binds for the slot, sampled at
 * the layer the surface's row names.
 */
export class SurfaceSlotArrayNode extends SurfaceSlotSamplerNode {
  // A binding a slot, which every sampler of the slot shares, as a slot's own sampler is.
  public override getUniformHash(): string {
    return `surface-array:${this.slot}`;
  }

  protected override resolve(material: ISurfaceSlotted): Maybe<Texture> {
    return material.surfaceArrays?.[this.slot]?.value;
  }
}
