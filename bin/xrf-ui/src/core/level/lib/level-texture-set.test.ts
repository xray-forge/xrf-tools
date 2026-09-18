import { beforeEach, describe, expect, it } from "@jest/globals";

import { createRoots } from "@/core/assets/lib";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ILevelTexture, LevelTextureSet } from "@/core/level/lib/level-texture-set";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import { mockLevelTextureReference } from "@/fixtures/mocks/level.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";

const ROOTS: XrayRoots = createRoots(["C:/game/db"]);

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

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(["stone"]);

    expect(set.get("stone")?.texture).not.toBeNull();
    expect(set.size).toBe(1);
  });

  // One ground texture dresses dozens of sectors. Uploading it per sector would spend the memory the streaming budget
  // exists to save.
  it("uploads a texture two sectors share only once", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(["stone"]);
    await set.load(["stone"]);

    expect(countReads()).toBe(1);
    expect(set.size).toBe(1);
  });

  it("joins a read already in flight rather than starting a second", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await Promise.all([set.load(["stone"]), set.load(["stone"])]);

    expect(countReads()).toBe(1);
  });

  it("keeps a texture the resident sectors still name", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(["stone"]);
    set.retain(new Set(["stone"]));

    expect(set.get("stone")?.texture).not.toBeNull();
  });

  it("disposes a texture nothing resident names any more", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(["stone"]);

    const loaded: ILevelTexture | null = set.get("stone");

    let disposed: boolean = false;

    loaded?.texture?.addEventListener("dispose", () => {
      disposed = true;
    });

    set.retain(new Set());

    expect(disposed).toBe(true);
    expect(set.get("stone")).toBeNull();
  });

  // The two ways counting claims per sector leaked: a sector evicted while its read was in flight left the finished
  // texture owned by nobody, and a cancelled stream left a claim for a sector that never arrived. Deriving what to
  // keep from what is resident makes both unreachable - whatever the reads did, the next reconcile is the truth.
  it("disposes a texture whose sector was gone before the read finished", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    const reading: Promise<void> = set.load(["stone"]);

    // The camera moved on: nothing is resident by the time the read lands.
    await reading;
    set.retain(new Set());

    expect(set.size).toBe(0);
  });

  it("is safe to reconcile against the same set twice", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(["stone"]);

    set.retain(new Set(["stone"]));
    set.retain(new Set(["stone"]));

    expect(set.size).toBe(1);
  });

  // A surface whose texture the roots do not hold should say so rather than looking like one never asked for.
  it("says why a reference the roots hold nothing for has no texture", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("missing", false)]);

    await set.load(["missing"]);

    expect(set.get("missing")?.texture).toBeNull();
    expect(set.get("missing")?.reason).toContain("missing");
  });

  it("releases everything when the level is swapped", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(["stone"]);

    set.open(ROOTS, [mockLevelTextureReference("other")]);

    expect(set.size).toBe(0);
  });
});
