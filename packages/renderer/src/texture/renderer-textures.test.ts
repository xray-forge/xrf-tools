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

  it("lets an evicted texture go once, and uploads it again within the budget once it is asked for", () => {
    const rebound: Array<string> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      (key: string) => rebound.push(key)
    );
    const target: ITextureTarget = createTarget(getWhiteTexture());
    let disposals: number = 0;

    textures.target("brick", getWhiteTexture(), target);
    textures.put("brick", { bytes: mockDdsFile(), encoding: ERendererTextureEncoding.DDS });
    textures.upload(RENDERER, Infinity);

    const texture: Texture = textures.getUploaded("brick") as Texture;

    texture.addEventListener("dispose", () => (disposals += 1));
    rebound.length = 0;

    expect(textures.evict("brick")).toBe(texture);
    expect(textures.evict("brick")).toBeNull();
    expect(disposals).toBe(1);
    expect(textures.hasQueued).toBe(false);
    expect(textures.isUploaded("brick")).toBe(false);
    expect(textures.getUploaded("brick")).toBeNull();
    expect(textures.hasQueued).toBe(true);

    textures.upload(RENDERER, Infinity);

    // The same texture, which its targets never stopped drawing: what waited for it is told it can draw.
    expect(textures.getUploaded("brick")).toBe(texture);
    expect(target.value).toBe(texture);
    expect(rebound).toEqual(["brick"]);

    textures.evict("brick");
    textures.release("brick");

    expect(disposals).toBe(2);
    expect(target.value).toBe(getWhiteTexture());
  });

  it("evicts nothing of a key whose latest texture is not up yet", () => {
    const textures: RendererTextures = new RendererTextures(
      () => {},
      () => {}
    );

    textures.put("brick", { bytes: mockDdsFile(), encoding: ERendererTextureEncoding.DDS });

    expect(textures.evict("brick")).toBeNull();
    expect(textures.evict("plaster")).toBeNull();
  });
});
