import { beforeEach, describe, expect, it } from "@jest/globals";

import { createRoots } from "@/core/assets/lib";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { ISectorTextureRequest } from "@/core/level/lib/sector/level-sector-textures";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelTexture, LevelTextureSet } from "@/core/level/lib/texture/level-texture-set";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import { mockLevelTextureReference } from "@/fixtures/mocks/level.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { Nullable } from "@/lib/types/general";

const ROOTS: XrayRoots = createRoots(["C:/game/db"]);

function countReads(): number {
  return mockInvoke.mock.calls.filter(([command]) => command === "plugin:assets|read_asset").length;
}

/** References as a sector names them, none of them read for alpha unless a case says otherwise. */
function requests(...references: Array<string>): Array<ISectorTextureRequest> {
  return references.map((reference: string) => ({ isAlphaRead: false, isMipped: true, reference }));
}

describe("LevelTextureSet", () => {
  beforeEach(() => {
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:assets|read_asset"]: mockDdsFile() });
  });

  it("loads the textures a sector names", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(requests("stone"));

    expect(set.get("stone")?.texture).not.toBeNull();
    expect(set.size).toBe(1);
  });

  // One ground texture dresses dozens of sectors. Uploading it per sector would spend the memory the streaming budget
  // exists to save.
  it("uploads a texture two sectors share only once", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(requests("stone"));
    await set.load(requests("stone"));

    expect(countReads()).toBe(1);
    expect(set.size).toBe(1);
  });

  it("joins a read already in flight rather than starting a second", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await Promise.all([set.load(requests("stone")), set.load(requests("stone"))]);

    expect(countReads()).toBe(1);
  });

  it("keeps a texture the resident sectors still name", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(requests("stone"));
    set.retain(new Set(["stone"]));

    expect(set.get("stone")?.texture).not.toBeNull();
  });

  it("disposes a texture nothing resident names any more", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(requests("stone"));

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

    const reading: Promise<void> = set.load(requests("stone"));

    // The camera moved on: nothing is resident by the time the read lands.
    await reading;
    set.retain(new Set());

    expect(set.size).toBe(0);
  });

  it("is safe to reconcile against the same set twice", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(requests("stone"));

    set.retain(new Set(["stone"]));
    set.retain(new Set(["stone"]));

    expect(set.size).toBe(1);
  });

  // A surface whose texture the roots do not hold should say so rather than looking like one never asked for. Left
  // untextured it takes its material's tint and reads as a plain surface; dressed in the checker it reads as a fault.
  it("draws a checker for a reference the roots hold nothing for, and says why", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("missing", false)]);

    await set.load(requests("missing"));

    expect(set.get("missing")?.texture).toBeTruthy();
    expect(set.get("missing")?.reason).toContain("missing");
  });

  // A file is uploaded once for the whole level, and whether its alpha survives is its callers' answer rather than
  // its own. Uploaded first for a sector of opaque surfaces, a cut-out file reached the surfaces that do test its
  // alpha with no alpha channel bound, which draws its transparent black as solid black.
  it("uploads a texture again when a later surface reads the alpha the first one did not", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("leaf")]);

    await set.load([{ isAlphaRead: false, isMipped: true, reference: "leaf" }]);

    const opaque: Nullable<ILevelTexture> = set.get("leaf");

    await set.load([{ isAlphaRead: true, isMipped: true, reference: "leaf" }]);

    expect(set.get("leaf")?.isAlphaRead).toBe(true);
    expect(set.get("leaf")?.texture).not.toBe(opaque?.texture);
    expect(set.size).toBe(1);
  });

  it("keeps a texture already uploaded with its alpha", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("leaf")]);

    await set.load([{ isAlphaRead: true, isMipped: true, reference: "leaf" }]);

    const held: Nullable<ILevelTexture> = set.get("leaf");

    await set.load([{ isAlphaRead: false, isMipped: true, reference: "leaf" }]);

    expect(set.get("leaf")?.texture).toBe(held?.texture);
  });

  it("reports nothing about a texture that is a picture", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(requests("stone"));

    expect(set.describe().problems).toEqual([]);
  });

  it("releases everything when the level is swapped", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(requests("stone"));

    set.open(ROOTS, [mockLevelTextureReference("other")]);

    expect(set.size).toBe(0);
  });

  it("says what became of each reference, without handing any of them over", async () => {
    const set: LevelTextureSet = new LevelTextureSet();

    set.open(ROOTS, [mockLevelTextureReference("stone")]);

    await set.load(requests("stone", "missing"));

    const report = set.describe();

    expect(report.uploaded).toBe(2);
    expect(report.dressing.get("stone")?.state).toBe(ELevelSurfaceDressing.UPLOADED);
    expect(report.dressing.get("stone")?.upload).toBe("1 level · linear · wrapped · aniso 8");
    expect(report.dressing.get("missing")?.state).toBe(ELevelSurfaceDressing.STOOD_IN);
    expect(report.problems.map((it) => it.reference)).toEqual(["missing"]);
  });
});
