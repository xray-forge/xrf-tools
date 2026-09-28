import { describe, expect, it } from "@jest/globals";
import { uv } from "three/tsl";
import { Texture, TextureNode, WebGPURenderer } from "three/webgpu";

import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { mockCubeDdsFile, mockDdsFile } from "#/dds/dds-fixtures";
import { getPlaceholderSkyTexture, getWhiteTexture } from "#/texture/placeholder-textures";
import { RendererTextures } from "#/texture/renderer-textures";
import { ITextureTarget } from "#/texture/texture-target";

/** A renderer that uploads nothing, which is all the queue asks of it here. */
const RENDERER: WebGPURenderer = { initTexture: () => {} } as unknown as WebGPURenderer;

function createTextures(): RendererTextures {
  return new RendererTextures(
    () => {},
    () => {}
  );
}

function createTarget(placeholder: Texture): ITextureTarget {
  return { value: placeholder };
}

function upload(textures: RendererTextures, key: string): void {
  textures.put(key, { bytes: mockDdsFile(), encoding: ERendererTextureEncoding.DDS });
  textures.upload(RENDERER, Infinity);
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

  // Three uploads again whatever a sampler still holds once it went, from the bytes left on the CPU, and a pipeline
  // compiled over such a target is one such sampler.
  it("lets an evicted texture go, its targets drawing their placeholders, and uploads it again once held", () => {
    const rebound: Array<string> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      (key: string) => rebound.push(key)
    );
    const target: ITextureTarget = createTarget(getWhiteTexture());
    let isDisposed: boolean = false;

    textures.target("brick", getWhiteTexture(), target);
    upload(textures, "brick");

    const texture: Texture = textures.getUploaded("brick") as Texture;

    texture.addEventListener("dispose", () => (isDisposed = true));
    rebound.length = 0;

    expect(textures.evict("brick")).toBe(texture);
    expect(textures.evict("brick")).toBeNull();
    expect(isDisposed).toBe(true);
    expect(target.value).toBe(getWhiteTexture());
    expect(textures.isEvicted("brick")).toBe(true);
    expect(textures.isUploaded("brick")).toBe(false);
    expect(textures.getUploaded("brick")).toBeNull();
    // Asking is no need: only a hold brings it back.
    expect(textures.hasQueued).toBe(false);

    textures.hold(["brick"]);

    expect(textures.hasQueued).toBe(true);

    textures.upload(RENDERER, Infinity);

    // The same texture, up again: its targets draw it, and what waited for it is told it can draw.
    expect(textures.getUploaded("brick")).toBe(texture);
    expect(textures.isEvicted("brick")).toBe(false);
    expect(target.value).toBe(texture);
    expect(rebound).toEqual(["brick"]);
  });

  it("disposes an evicted texture again as its key is released, for whatever of it three brought back", () => {
    const textures: RendererTextures = createTextures();
    const target: ITextureTarget = createTarget(getWhiteTexture());

    textures.target("brick", getWhiteTexture(), target);
    upload(textures, "brick");

    const texture: Texture = textures.evict("brick") as Texture;
    let isDisposed: boolean = false;

    texture.addEventListener("dispose", () => (isDisposed = true));
    textures.release("brick");

    expect(isDisposed).toBe(true);
    expect(target.value).toBe(getWhiteTexture());
  });

  it("binds a target to an evicted key as its placeholder, never to the texture let go", () => {
    const textures: RendererTextures = createTextures();
    const target: ITextureTarget = createTarget(getWhiteTexture());

    upload(textures, "brick");
    textures.target("brick", getWhiteTexture(), createTarget(getWhiteTexture()));
    textures.evict("brick");
    textures.target("brick", getWhiteTexture(), target);

    expect(target.value).toBe(getWhiteTexture());
  });

  it("evicts nothing something holds until the last hold lets go, and brings back what is held once evicted", () => {
    const textures: RendererTextures = createTextures();

    upload(textures, "brick");
    textures.hold(["brick", "brick"]);

    expect(textures.evict("brick")).toBeNull();

    textures.letGo(["brick"]);

    expect(textures.evict("brick")).toBeNull();

    textures.letGo(["brick"]);

    expect(textures.evict("brick")).not.toBeNull();
    expect(textures.hasQueued).toBe(false);

    textures.hold(["brick"]);

    expect(textures.hasQueued).toBe(true);
  });

  it("evicts nothing a sampler of its own binds, since whatever builds with it draws the key plainly", () => {
    const textures: RendererTextures = createTextures();
    const sampler: TextureNode = textures.bind("brick", getWhiteTexture(), uv());

    upload(textures, "brick");

    expect(textures.evict("brick")).toBeNull();

    textures.unbind("brick", sampler);

    expect(textures.evict("brick")).not.toBeNull();
  });

  it("keeps a key something holds while nothing else names it", () => {
    const textures: RendererTextures = createTextures();

    textures.hold(["brick"]);
    upload(textures, "brick");
    textures.release("brick");
    textures.put("brick", { bytes: mockDdsFile(), encoding: ERendererTextureEncoding.DDS });
    textures.upload(RENDERER, Infinity);

    expect(textures.evict("brick")).toBeNull();
  });

  // Something readying a batched surface asks about its keys every frame: asking brought the key back, its layer was
  // copied again and the key evicted again, every frame, and the queue never emptied.
  it("brings nothing back for being asked about, however often", () => {
    const rebound: Array<string> = [];
    const textures: RendererTextures = new RendererTextures(
      () => {},
      (key: string) => rebound.push(key)
    );

    textures.target("brick", getWhiteTexture(), createTarget(getWhiteTexture()));
    upload(textures, "brick");
    textures.evict("brick");
    rebound.length = 0;

    for (let frame: number = 0; frame < 3; frame += 1) {
      expect(textures.isUploaded("brick")).toBe(false);
      expect(textures.getUploaded("brick")).toBeNull();
      textures.upload(RENDERER, Infinity);
    }

    expect(textures.hasQueued).toBe(false);
    expect(textures.isEvicted("brick")).toBe(true);
    expect(rebound).toEqual([]);
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
