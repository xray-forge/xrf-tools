import { describe, expect, it } from "@jest/globals";
import { NodeFrame, NodeUpdateType, Texture } from "three/webgpu";

import { ESurfaceSlot, getSurfaceSlotPlaceholder, SURFACE_SLOTS, TSurfaceSlotTargets } from "#/material/surface-slot";
import { SurfaceSlotTextureNode } from "#/material/surface-slot-texture-node";

function toFrame(slots: TSurfaceSlotTargets): NodeFrame {
  const frame: NodeFrame = new NodeFrame();

  frame.material = { surfaceSlots: slots } as unknown as NodeFrame["material"];

  return frame;
}

function toSlots(base: Texture): TSurfaceSlotTargets {
  return Object.fromEntries(
    SURFACE_SLOTS.map((slot: ESurfaceSlot) => [
      slot,
      { value: slot === ESurfaceSlot.BASE ? base : getSurfaceSlotPlaceholder(slot) },
    ])
  ) as TSurfaceSlotTargets;
}

describe("SurfaceSlotTextureNode", () => {
  it("is updated for every object, whatever its build settles", () => {
    const node: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(getSurfaceSlotPlaceholder(ESurfaceSlot.BASE));

    // Three collects update nodes by the property, and a sampler's setup assigns it.
    node.updateType = NodeUpdateType.NONE;

    expect(node.updateType).toBe(NodeUpdateType.OBJECT);
  });

  // Three binds the samplers of one texture once: a base and a lightmap both at their white placeholder as they build
  // would sample one binding, and every lightmap the base.
  it("is a binding of its own a slot, whatever texture it holds as it builds", () => {
    const white: Texture = getSurfaceSlotPlaceholder(ESurfaceSlot.BASE);
    const base: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(white);
    const again: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(white);
    const lightmap: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(white);

    lightmap.slot = ESurfaceSlot.HEMI;

    expect(getSurfaceSlotPlaceholder(ESurfaceSlot.HEMI)).toBe(white);
    expect(lightmap.getUniformHash()).not.toBe(base.getUniformHash());
    expect(again.getUniformHash()).toBe(base.getUniformHash());
  });

  it("samples the slot of whichever material draws", () => {
    const node: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(getSurfaceSlotPlaceholder(ESurfaceSlot.BASE));
    const brick: Texture = new Texture();
    const plaster: Texture = new Texture();

    node.update(toFrame(toSlots(brick)));
    expect(node.value).toBe(brick);

    node.update(toFrame(toSlots(plaster)));
    expect(node.value).toBe(plaster);
  });

  it("keeps its slot through a clone, which writes through to the node it was cloned from", () => {
    const node: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(getSurfaceSlotPlaceholder(ESurfaceSlot.HEMI));
    const lightmap: Texture = new Texture();

    node.slot = ESurfaceSlot.HEMI;

    const biased: SurfaceSlotTextureNode = node.bias(node) as unknown as SurfaceSlotTextureNode;
    const slots: TSurfaceSlotTargets = { ...toSlots(new Texture()), [ESurfaceSlot.HEMI]: { value: lightmap } };

    biased.update(toFrame(slots));

    expect(biased.slot).toBe(ESurfaceSlot.HEMI);
    expect(node.value).toBe(lightmap);
  });
});
