import { describe, expect, it } from "@jest/globals";

import { ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { LevelTextureSet } from "@/core/level/lib/texture/level-texture-set";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";

function delivered(
  references: Array<string>,
  overrides: Partial<ILevelTextureDelivery> = {}
): Array<ILevelTextureDelivery> {
  return references.map((reference: string) => ({
    bytes: mockDdsFile(),
    isAlphaRead: false,
    isDecoded: false,
    isMipped: true,
    reason: null,
    reference,
    ...overrides,
  }));
}

describe("LevelTextureSet", () => {
  it("uploads what it is handed", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"]));

    expect(set.get("stone")?.texture).not.toBeNull();
    expect(set.size).toBe(1);
  });

  // One ground texture dresses dozens of sectors. Uploading it per sector would spend the memory the streaming
  // budget exists to save.
  it("uploads a texture two sectors share only once", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"]));

    const first = set.get("stone")?.texture;

    await set.take(delivered(["stone"]));

    expect(set.get("stone")?.texture).toBe(first);
    expect(set.size).toBe(1);
  });

  it("joins an upload already in flight rather than starting a second", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await Promise.all([set.take(delivered(["stone"])), set.take(delivered(["stone"]))]);

    expect(set.size).toBe(1);
  });

  // Whether a file keeps its alpha is decided by the surfaces drawn with it, and it is uploaded once for the whole
  // level by whichever sector asked first. A sector of opaque surfaces uploading a cut-out file as `RGB_S3TC_DXT1`
  // left every cut-out surface reached later testing an alpha channel that is not there, which draws the file's
  // transparent black as solid black.
  it("uploads a texture again when a later surface reads the alpha the first one did not", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"]));

    const opaque = set.get("stone")?.texture;

    await set.take(delivered(["stone"], { isAlphaRead: true }));

    expect(set.get("stone")?.texture).not.toBe(opaque);
    expect(set.get("stone")?.isAlphaRead).toBe(true);
  });

  it("keeps a texture already uploaded with its alpha", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"], { isAlphaRead: true }));

    const held = set.get("stone")?.texture;

    await set.take(delivered(["stone"]));

    expect(set.get("stone")?.texture).toBe(held);
  });

  it("says whether what it holds already satisfies a surface", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"]));

    expect(set.holds("stone", false, true)).toBe(true);
    // The same file, but a surface that reads its alpha needs it uploaded differently.
    expect(set.holds("stone", true, true)).toBe(false);
    expect(set.holds("grass", false, true)).toBe(false);
  });

  it("keeps a texture the resident sectors still name", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone", "grass"]));

    set.retain(new Set(["stone"]));

    expect(set.get("stone")?.texture).not.toBeNull();
    expect(set.get("grass")).toBeNull();
    expect(set.size).toBe(1);
  });

  it("is safe to reconcile against the same set twice", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"]));

    set.retain(new Set(["stone"]));
    set.retain(new Set(["stone"]));

    expect(set.size).toBe(1);
  });

  // A checker stands in so the surface is drawn as something rather than as nothing, and the reason rides with it:
  // a checker multiplied into a wall looks like a blending fault and is not one.
  it("draws a checker for a file that could not be read, and says why", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"], { reason: "Nothing in the mounted roots answers to it" }));

    expect(set.get("stone")?.texture).not.toBeNull();
    expect(set.describe().problems).toEqual([
      { reason: "Nothing in the mounted roots answers to it", reference: "stone" },
    ]);
    expect(set.describe().dressing.get("stone")?.state).toBe(ELevelSurfaceDressing.STOOD_IN);
  });

  it("reports nothing about a texture that uploaded cleanly", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"]));

    expect(set.describe().problems).toEqual([]);
    expect(set.describe().dressing.get("stone")?.state).toBe(ELevelSurfaceDressing.UPLOADED);
    expect(set.describe().uploaded).toBe(1);
  });

  it("releases everything when the level is swapped", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    await set.take(delivered(["stone"]));

    set.open();

    expect(set.size).toBe(0);
  });

  it("says what changed, so what draws from it re-dresses only that", async () => {
    const set: LevelTextureSet = new LevelTextureSet();
    const changes: Array<unknown> = [];

    set.subscribe((changed) => changes.push(changed));

    await set.take(delivered(["stone"]));

    set.retain(new Set());

    expect(changes).toEqual([new Set(["stone"]), new Set(["stone"])]);
  });
});
