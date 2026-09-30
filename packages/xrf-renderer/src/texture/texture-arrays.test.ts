import { describe, expect, it } from "@jest/globals";
import { mockDdsFile } from "@xrf/dds/fixtures";
import { CompressedArrayTexture, Texture } from "three/webgpu";

import { ITextureDeviceFixture, mockTextureDevice } from "#/internals/device-fixtures";
import { ITextureCopy } from "#/internals/texture-copy";
import { createRendererTexture } from "#/texture/renderer-texture";
import { TEXTURE_ARRAY_BYTES } from "#/texture/texture-array";
import { toTextureArrayClass } from "#/texture/texture-array-class";
import { ITextureArrayFlush } from "#/texture/texture-array-flush";
import { TextureArrays } from "#/texture/texture-arrays";
import { hasTextureData, listTextureData, releaseTextureData } from "#/texture/texture-data";
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

    arrays.release("lmap#1", first);

    expect((arrays.claim("lmap#3", createTexture()) as ITextureLayer).layer).toBe(2);

    arrays.release("lmap#1", first);

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

  it("fills a key's own texture from its layer, every level, only once the layer holds a copy of that texture", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const brick: Texture = createTexture("DXT5", 8, 2);

    arrays.claim("a", createTexture("DXT5", 8, 2));

    const held: ITextureLayer = arrays.claim("brick", brick) as ITextureLayer;

    // Still to be copied: the layer holds nothing of it yet.
    expect(arrays.toRestoreCopies("brick", brick)).toBeNull();

    arrays.flush();

    expect(arrays.toRestoreCopies("brick", brick)).toEqual([
      {
        destination: brick,
        destinationLayer: 0,
        height: 8,
        layers: 1,
        level: 0,
        source: held.array.target.value,
        sourceLayer: 1,
        width: 8,
      },
      {
        destination: brick,
        destinationLayer: 0,
        height: 4,
        layers: 1,
        level: 1,
        source: held.array.target.value,
        sourceLayer: 1,
        width: 4,
      },
    ]);
    // Another texture, or a key no array holds.
    expect(arrays.toRestoreCopies("brick", createTexture("DXT5", 8, 2))).toBeNull();
    expect(arrays.toRestoreCopies("plaster", brick)).toBeNull();

    arrays.release("brick", held);

    expect(arrays.toRestoreCopies("brick", brick)).toBeNull();
  });

  it("fills a key's own texture from the array it outgrew until a flush moves its layer", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const brick: Texture = createTexture("DXT5", 8, 1);
    const held: ITextureLayer = arrays.claim("brick", brick) as ITextureLayer;

    ["b", "c", "d"].forEach((key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)));
    arrays.flush();

    const outgrown: Texture = held.array.target.value;

    arrays.claim("e", createTexture("DXT5", 8, 1));

    expect(arrays.toRestoreCopies("brick", brick)?.map((copy: ITextureCopy) => copy.source)).toEqual([outgrown]);

    arrays.flush();

    expect(arrays.toRestoreCopies("brick", brick)?.map((copy: ITextureCopy) => copy.source)).toEqual([
      held.array.target.value,
    ]);
  });

  it("reuses the lowest free layer first, so the top stays what can be given up", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});

    const held: Array<ITextureLayer> = ["a", "b", "c", "d"].map(
      (key: string) => arrays.claim(key, createTexture()) as ITextureLayer
    );

    arrays.release("c", held[2]);
    arrays.release("a", held[0]);

    expect((arrays.claim("e", createTexture()) as ITextureLayer).layer).toBe(0);
    expect((arrays.claim("f", createTexture()) as ITextureLayer).layer).toBe(2);
    expect((arrays.claim("g", createTexture()) as ITextureLayer).layer).toBe(4);
  });

  it("gives up the layers let go from the top, so an array fitted afterwards is smaller", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const [first, second, , fourth, fifth] = ["a", "b", "c", "d", "e"].map(
      (key: string) => arrays.claim(key, createTexture("DXT5", 8, 1)) as ITextureLayer
    );

    arrays.flush();
    // The top two are given up, and the one below them stays free inside what is used.
    arrays.release("d", fourth);
    arrays.release("e", fifth);
    arrays.release("b", second);
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
    arrays.release("a", held);

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

    arrays.release("a", held);

    expect((arrays.claim("b", createTexture()) as ITextureLayer).layer).not.toBe(held.layer);

    arrays.release("a", held);

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
    ["c", "d", "e"].forEach((key: string, index: number) => arrays.release(key, held[index + 2]));
    arrays.compact(IDLE);

    const fitted: CompressedArrayTexture = held[0].array.target.value as CompressedArrayTexture;
    const [outgrown] = arrays.flush().copies;

    expect(fitted.image.depth).toBe(2);
    expect(outgrown).toMatchObject({ destination: fitted, layers: 2 });
  });

  it("holds a detached key anew as its new class, the layer held before kept until the views holding it let go", () => {
    const replaced: Array<string> = [];
    const arrays: TextureArrays = new TextureArrays((key: string) => replaced.push(key));
    const before: ITextureLayer = arrays.claim("a", createTexture()) as ITextureLayer;
    const drawn: Texture = before.array.target.value;

    arrays.flush();
    arrays.detach("a");

    expect(arrays.holds("a")).toBe(false);

    const after: ITextureLayer = arrays.claim("a", createTexture("DXT1")) as ITextureLayer;

    expect(after.array).not.toBe(before.array);
    expect(arrays.retain("a")).toEqual(after);
    expect(arrays.flush().disposals).not.toContain(drawn);

    arrays.release("a", before);

    expect(arrays.holds("a")).toBe(true);
    expect(arrays.flush().disposals).toContain(drawn);
    expect(replaced).toEqual([before.array.key]);
  });

  it("copies nothing more into a detached key's layer, and again into it for a key claimed back as its class", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const first: Texture = createTexture();
    const before: ITextureLayer = arrays.claim("a", first) as ITextureLayer;

    arrays.detach("a");

    expect(arrays.flush().copies).toHaveLength(0);

    const back: Texture = createTexture();

    arrays.claim("a", createTexture("DXT1"));
    arrays.detach("a");

    expect(arrays.claim("a", back)).toEqual(before);
    expect(
      arrays
        .flush()
        .copies.filter((copy: ITextureCopy) => copy.destination === before.array.target.value)
        .map((copy: ITextureCopy) => copy.source)
    ).toEqual([back, back]);
  });

  it("copies nothing again with the next flush for a key detached since the copy was skipped", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});

    arrays.claim("lmap", createTexture());

    const { copies }: ITextureArrayFlush = arrays.flush();

    arrays.detach("lmap");

    expect(arrays.retry(copies)).toEqual(new Set());
    expect(arrays.flush().copies).toHaveLength(0);
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

  // A key's own texture lets its bytes go once it is up, and the class may first be seen in such a one.
  it("makes an array of a class from a texture whose bytes went, each level's layer sized by the format", () => {
    const arrays: TextureArrays = new TextureArrays(() => {});
    const [block, colour]: Array<Texture> = [createTexture("DXT5", 8, 3), createTexture("DXT1", 8, 3)];

    [block, colour].forEach((texture: Texture) => releaseTextureData(texture));

    const sizes: Array<Array<number>> = [
      arrays.claim("bump", block) as ITextureLayer,
      arrays.claim("base", colour) as ITextureLayer,
    ].map((held: ITextureLayer) =>
      listTextureData(held.array.target.value).map((data: ArrayBufferView) => data.byteLength)
    );

    // Two blocks a side, one, then one for the smallest level: sixteen bytes a block, and eight for DXT1's.
    expect(sizes).toEqual([
      [64, 16, 16],
      [32, 8, 8],
    ]);
  });

  it("lets go of an array's zeroed layer once it is on the GPU, and an array made later makes its own", () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const arrays: TextureArrays = new TextureArrays(() => {});

    // An array a layer, so a key of a class held already opens another.
    arrays.layerLimit = 1;

    const up: ITextureLayer = arrays.claim("a", createTexture()) as ITextureLayer;
    const waiting: ITextureLayer = arrays.claim("b", createTexture("DXT1")) as ITextureLayer;
    const zeros: ArrayBufferView = listTextureData(waiting.array.target.value)[0];

    device.renderer.initTexture(up.array.target.value);
    arrays.releaseZeros(device.renderer);

    // The other, not up yet, keeps what it is written from.
    expect(hasTextureData(up.array.target.value)).toBe(false);
    expect(listTextureData(waiting.array.target.value)[0]).toBe(zeros);

    const later: ITextureLayer = arrays.claim("c", createTexture("DXT1")) as ITextureLayer;

    expect(later.array).not.toBe(waiting.array);
    expect(listTextureData(later.array.target.value)[0]).not.toBe(zeros);
    expect(listTextureData(later.array.target.value)[0].byteLength).toBe(zeros.byteLength);
  });
});
