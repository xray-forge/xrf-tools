import { describe, expect, it } from "@jest/globals";
import { Texture } from "three/webgpu";

import { createRendererRawTexture } from "#/texture/renderer-texture";
import { markRendererTextureNew } from "#/texture/renderer-texture-version";

describe("markRendererTextureNew", () => {
  // Three rebuilds a sampler's bind group for another texture only where the two versions differ.
  it("gives every texture made a version of its own, as a first upload asks", () => {
    const first: Texture = createRendererRawTexture(new ArrayBuffer(4), 1, 1);
    const second: Texture = createRendererRawTexture(new ArrayBuffer(4), 1, 1);
    const third: Texture = new Texture();

    markRendererTextureNew(third);

    expect(first.version).toBeGreaterThan(0);
    expect(new Set([first.version, second.version, third.version]).size).toBe(3);
  });
});
