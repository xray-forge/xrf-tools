import { describe, expect, it } from "@jest/globals";
import { CompressedArrayTexture, Texture } from "three/webgpu";

import { mockDdsFile } from "#/dds/dds-fixtures";
import { ITextureCopy } from "#/internals/texture-copy";
import { createRendererTexture } from "#/texture/renderer-texture";
import { toTextureArrayClass } from "#/texture/texture-array-class";
import { ITextureArrayFlush } from "#/texture/texture-array-flush";
import { TextureArrays } from "#/texture/texture-arrays";
import { ITextureLayer } from "#/texture/texture-layer";

/** A clock reading long after every claim a test makes. */
const IDLE: number = performance.now() + 60_000;

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

  it("copies every level of a claimed texture into its layer, block rows whole, then lets the texture go", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const texture: Texture = createTexture("DXT5", 8, 3);
    const held: ITextureLayer = arrays.claim("lmap", texture) as ITextureLayer;
    const flush: ITextureArrayFlush = arrays.flush();

    // Eight, four and two texels a side: the last is a block of four.
    expect(
      flush.copies.map((copy: ITextureCopy) => [copy.source, copy.width, copy.destinationLayer, copy.level])
    ).toEqual([
      [texture, 8, held.layer, 0],
      [texture, 4, held.layer, 1],
      [texture, 4, held.layer, 2],
    ]);
    expect(flush.disposals).toEqual([texture]);
    expect(flush.evicted).toEqual(["lmap"]);
    expect(arrays.flush().copies).toHaveLength(0);
  });

  it("fits an array to what it holds once it has gone unchanged a while, copying only the layers it used", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const [first] = ["a", "b", "c", "d", "e"].map(
      (key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)) as ITextureLayer
    );

    arrays.flush();

    const before: CompressedArrayTexture = first.array.target.value as CompressedArrayTexture;

    // Five of six, but only just claimed: a level streaming in is left alone.
    expect(before.image.depth).toBe(6);

    arrays.compact(performance.now());

    expect(arrays.flush().copies).toHaveLength(0);

    arrays.compact(IDLE);

    const { copies, disposals }: ITextureArrayFlush = arrays.flush();
    const after: CompressedArrayTexture = first.array.target.value as CompressedArrayTexture;

    expect(after.image.depth).toBe(5);
    expect(copies).toEqual([expect.objectContaining({ destination: after, layers: 5, source: before })]);
    expect(disposals).toEqual([before]);
  });

  it("grows an array by half again, copying every layer it held into the larger one, and says so", () => {
    const replaced: Array<string> = [];
    const arrays: TextureArrays = new TextureArrays((key: string) => replaced.push(key));
    const held: Array<ITextureLayer> = ["a", "b", "c", "d"].map(
      (key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)) as ITextureLayer
    );
    const before: Texture = held[0].array.target.value;

    arrays.flush();
    arrays.claim("e", createTexture("DXT5", 8, 1));

    const after: CompressedArrayTexture = held[0].array.target.value as CompressedArrayTexture;
    const { copies, disposals }: ITextureArrayFlush = arrays.flush();

    expect(after).not.toBe(before);
    expect(after.image.depth).toBe(6);
    expect(replaced).toEqual([held[0].array.key]);
    expect(copies[0]).toMatchObject({ destination: after, destinationLayer: 0, layers: 4, source: before });
    expect(copies[1]).toMatchObject({ destinationLayer: 4, layers: 1 });
    expect(disposals).toContain(before);
  });
});
