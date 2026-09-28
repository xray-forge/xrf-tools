import { describe, expect, it } from "@jest/globals";
import { CompressedArrayTexture, Texture } from "three/webgpu";

import { mockDdsFile } from "#/dds/dds-fixtures";
import { ITextureCopy } from "#/internals/texture-copy";
import { createRendererTexture } from "#/texture/renderer-texture";
import { TEXTURE_ARRAY_BYTES } from "#/texture/texture-array";
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

  it("copies every level of a claimed texture into its layer, block rows whole, and names its key copied", () => {
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
    // The texture is its key's, which lets it go.
    expect(flush.disposals).toEqual([]);
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

  it("reuses the lowest free layer first, so the top stays what can be given up", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});

    ["a", "b", "c", "d"].forEach((key: string) => arrays.claim(key, createTexture()));
    arrays.release("c");
    arrays.release("a");

    expect((arrays.claim("e", createTexture()) as ITextureLayer).layer).toBe(0);
    expect((arrays.claim("f", createTexture()) as ITextureLayer).layer).toBe(2);
    expect((arrays.claim("g", createTexture()) as ITextureLayer).layer).toBe(4);
  });

  it("gives up the layers let go from the top, so an array fitted afterwards is smaller", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const [first] = ["a", "b", "c", "d", "e"].map(
      (key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)) as ITextureLayer
    );

    arrays.flush();
    // The top two are given up, and the one below them stays free inside what is used.
    arrays.release("d");
    arrays.release("e");
    arrays.release("b");
    arrays.compact(IDLE);

    const { copies }: ITextureArrayFlush = arrays.flush();

    expect((first.array.target.value as CompressedArrayTexture).image.depth).toBe(3);
    expect(copies).toEqual([expect.objectContaining({ layers: 3 })]);
  });

  it("lets an array holding nothing go with the next flush, and says so", () => {
    const replaced: Array<string> = [];
    const arrays: TextureArrays = new TextureArrays((key: string) => replaced.push(key));
    const held: ITextureLayer = arrays.claim("a", createTexture()) as ITextureLayer;
    const texture: Texture = held.array.target.value;

    arrays.flush();
    arrays.release("a");

    const { disposals }: ITextureArrayFlush = arrays.flush();

    expect(disposals).toEqual([texture]);
    expect(replaced).toEqual([held.array.key]);
    expect((arrays.claim("b", createTexture()) as ITextureLayer).array).not.toBe(held.array);
    expect(arrays.flush().disposals).toEqual([]);
  });

  it("holds no more layers in an array than fit its bytes, whatever the device allows", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    // A layer of one level of DXT5 at 2048 a side: a byte a texel.
    const held: ITextureLayer = arrays.claim("a", createTexture("DXT5", 2048, 1)) as ITextureLayer;

    expect(held.array.limit).toBe(TEXTURE_ARRAY_BYTES / (2048 * 2048));
    expect(held.array.limit).toBeLessThan(arrays.layerLimit);
  });

  it("retains a held key for one more surface, counted as a claim, and nothing it does not hold", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const held: ITextureLayer = arrays.claim("a", createTexture()) as ITextureLayer;

    expect(arrays.retain("a")).toEqual(held);
    expect(arrays.retain("b")).toBeNull();

    arrays.release("a");

    expect((arrays.claim("b", createTexture()) as ITextureLayer).layer).not.toBe(held.layer);

    arrays.release("a");

    expect(arrays.retain("a")).toBeNull();
  });

  it("copies nothing again for the texture a layer was copied from, uploaded again", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const texture: Texture = createTexture();

    arrays.claim("a", texture);
    arrays.flush();

    expect(arrays.refresh("a", texture)).toBe(true);
    expect(arrays.flush().copies).toHaveLength(0);
  });

  it("copies what an array held into the last of two replacements made before a flush, and lets both go", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const held: Array<ITextureLayer> = ["a", "b", "c", "d"].map(
      (key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)) as ITextureLayer
    );
    const first: Texture = held[0].array.target.value;

    arrays.flush();
    // Grown to six, then to nine.
    ["e", "f", "g"].forEach((key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)));

    const last: CompressedArrayTexture = held[0].array.target.value as CompressedArrayTexture;
    const { copies, disposals, replaced }: ITextureArrayFlush = arrays.flush();

    expect(last.image.depth).toBe(9);
    expect(copies[0]).toMatchObject({ destination: last, layers: 4, source: first });
    expect(copies.slice(1).every((copy: ITextureCopy) => copy.destination === last)).toBe(true);
    expect(disposals).toHaveLength(2);
    expect(disposals).toContain(first);
    expect(disposals).not.toContain(last);
    expect(replaced).toEqual([held[0].array.key]);
  });

  it("copies no more of an outgrown array than an array fitted since holds", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const held: Array<ITextureLayer> = ["a", "b", "c", "d", "e"].map(
      (key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)) as ITextureLayer
    );

    // Grown to six and not flushed yet, then all but two let go and fitted.
    ["c", "d", "e"].forEach((key: string) => arrays.release(key));
    arrays.compact(IDLE);

    const fitted: CompressedArrayTexture = held[0].array.target.value as CompressedArrayTexture;
    const [outgrown] = arrays.flush().copies;

    expect(fitted.image.depth).toBe(2);
    expect(outgrown).toMatchObject({ destination: fitted, layers: 2 });
  });

  it("copies a key's layer again with the next flush where the frame could not, and nothing else", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const texture: Texture = createTexture("DXT5", 8, 2);

    arrays.claim("lmap", texture);
    arrays.claim("other", createTexture("DXT5", 8, 2));

    const { copies }: ITextureArrayFlush = arrays.flush();
    const skipped: Array<ITextureCopy> = copies.filter((copy: ITextureCopy) => copy.source === texture);

    expect(arrays.retry(skipped)).toEqual(new Set(["lmap"]));

    const again: ITextureArrayFlush = arrays.flush();

    expect(again.copies.map((copy: ITextureCopy) => copy.source)).toEqual([texture, texture]);
    expect(again.evicted).toEqual(["lmap"]);
  });
});
