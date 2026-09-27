import { describe, expect, it } from "@jest/globals";
import { Texture, WebGPURenderer } from "three/webgpu";

import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { mockCubeDdsFile, mockDdsFile } from "#/dds/dds-fixtures";
import { getPlaceholderSkyTexture, getWhiteTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { ITextureTarget } from "#/texture/texture-target";

/** A renderer that uploads nothing, which is all the queue asks of it here. */
const RENDERER: WebGPURenderer = { initTexture: () => {} } as unknown as WebGPURenderer;

function createTarget(placeholder: Texture): ITextureTarget {
  return { value: placeholder };
}

describe("RendererTextures", () => {
  // A flat sampler bound to a cube, or a cube's to a picture, is a pipeline that does not compile.
  it("draws a key's texture only into targets sampling its kind, the others keeping their placeholder", () => {
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {}
    );
    const flat: ITextureTarget = createTarget(getWhiteTexture());
    const sky: ITextureTarget = createTarget(getPlaceholderSkyTexture());
    const cubeFlat: ITextureTarget = createTarget(getWhiteTexture());

    textures.target("sky", getPlaceholderSkyTexture(), sky);
    textures.target("sky", getWhiteTexture(), cubeFlat);
    textures.target("brick", getWhiteTexture(), flat);
    textures.put("sky", { bytes: mockCubeDdsFile(), encoding: ERendererTextureEncoding.DDS });
    textures.put("brick", { bytes: mockDdsFile(), encoding: ERendererTextureEncoding.DDS });
    textures.upload(RENDERER, Infinity);

    expect((sky.value as { isCubeTexture?: boolean }).isCubeTexture).toBe(true);
    expect(sky.value).not.toBe(getPlaceholderSkyTexture());
    expect(cubeFlat.value).toBe(getWhiteTexture());
    expect(flat.value).not.toBe(getWhiteTexture());
  });
});
