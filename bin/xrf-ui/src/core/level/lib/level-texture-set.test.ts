import { beforeEach, describe, expect, it } from "@jest/globals";

import { createRoots } from "@/core/assets/lib";
import { LevelTextureReference } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTexture, LevelTextureSet } from "@/core/level/lib/level-texture-set";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";

const ROOTS: XrayRoots = createRoots(["C:/game/db"]);

function reference(name: string, isPresent: boolean = true): LevelTextureReference {
  return { logicalPath: isPresent ? `textures/${name}.dds` : null, reference: name };
}

function countReads(): number {
  return mockInvoke.mock.calls.filter(([command]) => command === "plugin:assets|read_asset").length;
}

describe("LevelTextureSet", () => {
  beforeEach(() => {
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:assets|read_asset"]: mockDdsFile() });
  });

  it("loads the textures a sector names", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [reference("stone")]);

    await set.acquire(0, ["stone"]);

    expect(set.get("stone")?.texture).not.toBeNull();
    expect(set.size).toBe(1);
  });

  // One ground texture dresses dozens of sectors. Uploading it per sector would spend the memory the streaming budget
  // exists to save.
  it("uploads a texture two sectors share only once", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [reference("stone")]);

    await set.acquire(0, ["stone"]);
    await set.acquire(1, ["stone"]);

    expect(countReads()).toBe(1);
    expect(set.size).toBe(1);
  });

  it("joins a read already in flight rather than starting a second", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [reference("stone")]);

    await Promise.all([set.acquire(0, ["stone"]), set.acquire(1, ["stone"])]);

    expect(countReads()).toBe(1);
  });

  // The texture is device memory, so it has to go when nothing names it - but not while another sector still does.
  it("keeps a texture while another sector still names it", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [reference("stone")]);

    await set.acquire(0, ["stone"]);
    await set.acquire(1, ["stone"]);

    set.release(0);

    expect(set.get("stone")?.texture).not.toBeNull();

    set.release(1);

    expect(set.get("stone")).toBeNull();
  });

  it("disposes the texture of the last sector that named it", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [reference("stone")]);

    await set.acquire(0, ["stone"]);

    const loaded: ILevelTexture | null = set.get("stone");

    let disposed: boolean = false;

    loaded?.texture?.addEventListener("dispose", () => {
      disposed = true;
    });

    set.release(0);

    expect(disposed).toBe(true);
  });

  // A surface whose texture the roots do not hold should say so rather than looking like one that was never asked for.
  it("says why a reference the roots hold nothing for has no texture", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [reference("missing", false)]);

    await set.acquire(0, ["missing"]);

    expect(set.get("missing")?.texture).toBeNull();
    expect(set.get("missing")?.reason).toContain("missing");
  });

  it("releases everything when the level is swapped", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [reference("stone")]);

    await set.acquire(0, ["stone"]);

    set.open(ROOTS, [reference("other")]);

    expect(set.size).toBe(0);
  });
});
