import { describe, expect, it } from "@jest/globals";
import { NodeFrame, Texture } from "three/webgpu";

import { ESurfaceSlot, getSurfaceSlotPlaceholder } from "#/material/surface-slot";
import { SurfaceSlotArrayNode } from "#/material/surface-slot-array-node";
import { SurfaceSlotNodes } from "#/material/surface-slot-nodes";
import { SurfaceSlotTextureNode } from "#/material/surface-slot-texture-node";
import { getPlaceholderArrayTexture } from "#/texture/placeholder-textures";

describe("SurfaceSlotNodes", () => {
  it("points a sampler holding a texture let go back at its placeholder, and its clones with it", () => {
    const nodes: SurfaceSlotNodes = new SurfaceSlotNodes();
    const placeholder: Texture = getSurfaceSlotPlaceholder(ESurfaceSlot.BASE);
    const node: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(placeholder);
    const kept: SurfaceSlotTextureNode = new SurfaceSlotTextureNode(placeholder);
    const brick: Texture = new Texture();
    const plaster: Texture = new Texture();

    nodes.add(node);
    nodes.add(kept);

    const biased: SurfaceSlotTextureNode = node.bias(node) as unknown as SurfaceSlotTextureNode;

    node.value = brick;
    kept.value = plaster;
    nodes.forget(new Set([brick]));

    expect(node.value).toBe(placeholder);
    expect(biased.value).toBe(placeholder);
    expect(kept.value).toBe(plaster);

    nodes.clear();
    kept.value = brick;
    nodes.forget(new Set([brick]));

    expect(kept.value).toBe(brick);
  });
});

describe("SurfaceSlotArrayNode", () => {
  it("samples the array whichever shared material draws binds for its slot, and keeps it for one binding none", () => {
    const node: SurfaceSlotArrayNode = new SurfaceSlotArrayNode(getPlaceholderArrayTexture());
    const array: Texture = new Texture();
    const frame: NodeFrame = new NodeFrame();

    node.slot = ESurfaceSlot.HEMI;
    frame.material = { surfaceArrays: { hemi: { value: array } } } as unknown as NodeFrame["material"];
    node.update(frame);

    expect(node.value).toBe(array);

    frame.material = { surfaceArrays: {} } as unknown as NodeFrame["material"];
    node.update(frame);

    expect(node.value).toBe(array);
    expect(node.getUniformHash()).toBe("surface-array:hemi");
  });
});
