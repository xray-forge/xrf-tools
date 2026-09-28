import { Maybe } from "@xrf/types";
import { Texture } from "three/webgpu";

import { SurfaceSlotSamplerNode } from "#/material/surface-slot-sampler-node";
import { ISurfaceSlotted } from "#/material/surface-slotted";

/**
 * A sampler of one slot of whatever material draws, from the texture the material's own slot holds.
 */
export class SurfaceSlotTextureNode extends SurfaceSlotSamplerNode {
  // Three binds the samplers of one texture once, by its uuid: slots holding the same placeholder as they build would
  // share one binding. A slot is its own binding, which every sampler of it shares.
  public override getUniformHash(): string {
    return `surface-slot:${this.slot}`;
  }

  protected override resolve(material: ISurfaceSlotted): Maybe<Texture> {
    return material.surfaceSlots?.[this.slot].value;
  }
}
