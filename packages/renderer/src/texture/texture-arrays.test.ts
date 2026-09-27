import { describe, expect, it, jest } from "@jest/globals";
import { Box2, Box3, CompressedArrayTexture, Texture, Vector3, WebGPURenderer } from "three/webgpu";

import { mockDdsFile } from "#/dds/dds-fixtures";
import { createRendererTexture } from "#/texture/renderer-texture";
import { toTextureArrayClass } from "#/texture/texture-array-class";
import { ITextureLayer, TextureArrays } from "#/texture/texture-arrays";

/** A copy three was asked for: what from, what to, the region, where to, and the levels. */
type TCopy = [Texture, Texture, Box2 | Box3, Vector3, number, number];

function createRenderer(): { renderer: WebGPURenderer; copies: Array<TCopy> } {
  const copies: Array<TCopy> = [];
  const renderer = {
    copyTextureToTexture: jest.fn((...copy: TCopy) => copies.push(copy)),
    initTexture: () => {},
  } as unknown as WebGPURenderer;

  return { copies, renderer };
}

function createTexture(fourCC: string = "DXT5", size: number = 8, mipmapCount: number = 2): Texture {
  return createRendererTexture(mockDdsFile({ fourCC, height: size, mipmapCount, width: size })).texture as Texture;
}

describe("toTextureArrayClass", () => {
  it("is shared by block textures of one format, size and chain, and by nothing else", () => {
    expect(toTextureArrayClass(createTexture())).toBe(toTextureArrayClass(createTexture()));
    expect(toTextureArrayClass(createTexture("DXT1"))).not.toBe(toTextureArrayClass(createTexture()));
    expect(toTextureArrayClass(createTexture("DXT5", 16))).not.toBe(toTextureArrayClass(createTexture()));
    expect(toTextureArrayClass(createTexture("DXT5", 8, 1))).not.toBe(toTextureArrayClass(createTexture()));
    expect(toTextureArrayClass(new Texture())).toBeNull();
  });
});

describe("TextureArrays", () => {
  it("holds a key once however many surfaces claim it, and its layer frees with the last", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const texture: Texture = createTexture();
    const first: ITextureLayer = arrays.claim("lmap#1", texture) as ITextureLayer;

    expect(arrays.claim("lmap#1", texture)).toEqual(first);
    expect((arrays.claim("lmap#2", createTexture()) as ITextureLayer).layer).toBe(1);

    arrays.release("lmap#1");

    expect((arrays.claim("lmap#3", createTexture()) as ITextureLayer).layer).toBe(2);

    arrays.release("lmap#1");

    expect((arrays.claim("lmap#4", createTexture()) as ITextureLayer).layer).toBe(first.layer);
  });

  it("keeps each class in arrays of its own, a class opening another at the device's limit", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});

    arrays.layerLimit = 2;

    const [a, b, c] = ["a", "b", "c"].map((key: string) => arrays.claim(key, createTexture()) as ITextureLayer);
    const other: ITextureLayer = arrays.claim("d", createTexture("DXT1")) as ITextureLayer;

    expect(b.array).toBe(a.array);
    expect(c.array).not.toBe(a.array);
    expect(c.layer).toBe(0);
    expect(other.array).not.toBe(a.array);
    expect(other.array).not.toBe(c.array);
  });

  it("copies a layer again for a texture of its class, and holds a key of another class nowhere", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});

    arrays.claim("lmap", createTexture());

    expect(arrays.refresh("lmap", createTexture())).toBe(true);
    expect(arrays.refresh("lmap", createTexture("DXT1"))).toBe(false);
    expect(arrays.refresh("lmap", null)).toBe(false);
    expect(arrays.claim("lmap", createTexture("DXT1"))).toBeNull();
  });

  it("copies every level of a claimed texture into its layer, block rows whole", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const { copies, renderer } = createRenderer();
    const texture: Texture = createTexture("DXT5", 8, 3);
    const held: ITextureLayer = arrays.claim("lmap", texture) as ITextureLayer;

    arrays.flush(renderer);

    // Eight, four and two texels a side: the last is a block of four.
    expect(copies.map(([source, , region, at, level]) => [source, (region as Box2).max.x, at.z, level])).toEqual([
      [texture, 8, held.layer, 0],
      [texture, 4, held.layer, 1],
      [texture, 4, held.layer, 2],
    ]);

    arrays.flush(renderer);

    expect(copies).toHaveLength(3);
  });

  it("grows an array by copying every layer it held into a larger one, and says so", () => {
    const replaced: Array<string> = [];
    const arrays: TextureArrays = new TextureArrays((key: string) => replaced.push(key));
    const { copies, renderer } = createRenderer();
    const held: Array<ITextureLayer> = ["a", "b", "c", "d"].map(
      (key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)) as ITextureLayer
    );
    const before: Texture = held[0].array.target.value;

    arrays.flush(renderer);
    copies.length = 0;
    arrays.claim("e", createTexture("DXT5", 8, 1));

    const after: CompressedArrayTexture = held[0].array.target.value as CompressedArrayTexture;

    arrays.flush(renderer);

    expect(after).not.toBe(before);
    expect(after.image.depth).toBe(8);
    expect(replaced).toEqual([held[0].array.key]);
    expect(copies[0][0]).toBe(before);
    expect((copies[0][2] as Box3).max.z).toBe(4);
    expect(copies[1][3].z).toBe(4);
  });
});
