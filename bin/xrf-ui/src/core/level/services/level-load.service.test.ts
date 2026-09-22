import { beforeEach, describe, expect, it } from "@jest/globals";
import { isObservableProp, reaction } from "@wirestate/mobx";
import { mockDdsFile } from "@xrf/renderer/fixtures";
import { Nullable } from "@xrf/types";

import { createRoots } from "@/core/assets/lib";
import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { SectorDescription, SectorOutline } from "@/core/ipc/types/xrf-visual";
import {
  ILevelSectorDelivery,
  ILevelTextureDelivery,
  ILevelTextureSupplyChange,
} from "@/core/level/lib/render/level-render-protocol";
import { createLevelResidency } from "@/core/level/lib/residency/level-residency";
import {
  mockLevelTextureReference,
  mockSectorDescription,
  mockSectorOutline,
  mockSelectedLevelDescription,
} from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { IDLE_LEVEL_STREAM, LevelLoadService } from "./level-load.service";

const ROOTS: XrayRoots = createRoots(["C:\\game\\db"]);
const ORIGIN = { x: 0, y: 0, z: 0 };

/** A sector sitting at one distance along x, with a unit sphere around it. */
function outlineAt(sector: number, x: number): SectorOutline {
  return mockSectorOutline({
    bounds: {
      boundingBox: { max: { x: x + 1, y: 1, z: 1 }, min: { x: x - 1, y: -1, z: -1 } },
      boundingSphere: { center: { x, y: 0, z: 0 }, radius: 1 },
    },
    sector,
  });
}

/** A level of the given sectors, and one packed sector every read answers with. */
function mockStreamable(sectors: Array<SectorOutline>): {
  level: SelectedLevelDescription;
  description: SectorDescription;
  buffer: ArrayBuffer;
} {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description: SectorDescription = mockSectorDescription(buffer);

  return { buffer: buffer.toArrayBuffer(), description, level: mockSelectedLevelDescription({ sectors }) };
}

/** Arms the three commands a stream makes, answering every sector read with the same pack. */
function armLevel(level: SelectedLevelDescription, description: SectorDescription, buffer: ArrayBuffer): void {
  setMockInvokeResponses({
    ["plugin:levels|open_level"]: mockSessionResponse(level),
    ["plugin:levels|open_sector"]: mockSessionResponse((args?: Record<string, unknown>) => ({
      ...description,
      sector: args?.sector as number,
    })),
    ["plugin:levels|read_sector"]: buffer,
  });
}

function countCalls(command: string): number {
  return mockInvoke.mock.calls.filter(([name]) => name === command).length;
}

function recordSupply(service: LevelLoadService): {
  delivered: Array<ILevelTextureDelivery>;
  retained: Array<Nullable<ReadonlySet<string>>>;
} {
  const delivered: Array<ILevelTextureDelivery> = [];
  const retained: Array<Nullable<ReadonlySet<string>>> = [];

  service.textures.subscribe((change: ILevelTextureSupplyChange) => {
    delivered.push(...change.delivered);
    retained.push(change.retained);
  });

  return { delivered, retained };
}

function createHeldCall(): { held: Promise<void>; release: () => void } {
  const release: Array<() => void> = [];
  const held: Promise<void> = new Promise((resolve) => release.push(resolve));

  return { held, release: () => release[0]() };
}

describe("LevelLoadService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  // A refresh re-provisions the service while the backend keeps its session, which is the whole reason the backend
  // keeps it: coming back to an empty picker beside a level that is still open reads as having lost it.
  it("takes back the level the backend still has open, without opening it again", async () => {
    const { level } = mockStreamable([outlineAt(0, 5)]);
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({ ["plugin:levels|get_level"]: mockSessionResponse(level) });

    await service.restore();

    expect(service.level.value?.selected.value.sectors).toHaveLength(1);
    expect(countCalls("plugin:levels|open_level")).toBe(0);
  });

  // The defect a restore first shipped with: it took the description and not the textures, so every surface of a
  // restored level was drawn flat grey. A restore has to arrive at the same state an open does.
  it("dresses a restored level from the roots the open searched", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 5)]);
    const { service } = mockInjectedService(LevelLoadService);
    const textures = [mockLevelTextureReference("stone")];

    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: mockDdsFile(),
      ["plugin:levels|get_level"]: mockSessionResponse({
        ...level,
        bounds: { ...level.bounds!, boundingSphere: { center: { x: 0, y: 0, z: 0 }, radius: 2000 } },
        textures,
      }),
      ["plugin:levels|open_sector"]: mockSessionResponse((args?: Record<string, unknown>) => ({
        ...description,
        sector: args?.sector as number,
      })),
      ["plugin:levels|read_sector"]: buffer,
    });

    const supply = recordSupply(service);

    await service.restore();
    await service.stream(ORIGIN);

    // Read on this side and handed on as bytes: uploading it belongs to whichever side draws.
    expect(supply.delivered.map((it) => it.reference)).toContain("stone");
    expect(supply.delivered[0].bytes.byteLength).toBeGreaterThan(0);
    // Sized from the description's own extent, which is the only place a restore can learn it from.
    expect(service.residency).toEqual(createLevelResidency(2000));
  });

  // The same defect, one thread over: a level handed to a renderer on another thread is read again from scratch,
  // and the first attempt forgot where the texture files were along with what had been supplied. Every surface of
  // the level came back in the missing-texture checkerboard.
  it("dresses the level again when it is read for a second renderer", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 5)]);
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: mockDdsFile(),
      ["plugin:levels|get_level"]: mockSessionResponse({ ...level, textures: [mockLevelTextureReference("stone")] }),
      ["plugin:levels|open_sector"]: mockSessionResponse((args?: Record<string, unknown>) => ({
        ...description,
        sector: args?.sector as number,
      })),
      ["plugin:levels|read_sector"]: buffer,
    });

    await service.restore();
    await service.stream(ORIGIN);

    const supply = recordSupply(service);

    await service.restream();

    expect(supply.delivered.map((it) => it.reference)).toContain("stone");
    expect(supply.delivered.every((it) => it.reason === null)).toBe(true);
    expect(supply.delivered[0].bytes.byteLength).toBeGreaterThan(0);
  });

  // Nothing is held any more, so whatever draws now is told so before it is given anything.
  it("tells whatever draws that everything it held is gone", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 5)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "level", path: "levels\\zaton" } as never, ROOTS);
    await service.stream(ORIGIN);

    const released: Array<Nullable<ReadonlyArray<number>>> = [];

    service.sectors.subscribe((change) => released.push(change.released));

    await service.restream();

    expect(released[0]).toBeNull();
    expect(service.sectorReport.held).toEqual([0]);
  });

  // Adopted with its session id, so the sector reads that follow a restore belong to the opening the backend holds
  // rather than to nothing.
  it("reads sectors against the session it restored", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 5)]);
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({
      ["plugin:levels|get_level"]: mockSessionResponse(level),
      ["plugin:levels|open_sector"]: mockSessionResponse((args?: Record<string, unknown>) => ({
        ...description,
        sector: args?.sector as number,
      })),
      ["plugin:levels|read_sector"]: buffer,
    });

    await service.restore();
    await service.stream(ORIGIN);

    const opened = mockInvoke.mock.calls.find(([name]) => name === "plugin:levels|open_sector");

    expect((opened?.[1] as { sessionId: string }).sessionId).toBe(service.level.value?.selected.sessionId);
    expect(service.sectorReport.held).toEqual([0]);
  });

  it("reports itself ready even when there is nothing to restore", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({ ["plugin:levels|get_level"]: null });

    await service.onProvision();

    expect(service.isReady).toBe(true);
    expect(service.level.value).toBeNull();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(LevelLoadService);

    expect(isObservableProp(service, "level")).toBe(true);
    expect(isObservableProp(service, "sectorReport")).toBe(true);
    expect(isObservableProp(service, "residency")).toBe(true);
  });

  // The whole point of the open: a quarter of a gigabyte of geometry stays on disk until a camera asks for a piece.
  it("opens a level without reading any geometry", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 5)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    expect(service.level.value?.selected.value.sectors).toHaveLength(1);
    expect(service.sectorReport.held).toHaveLength(0);
    expect(countCalls("plugin:levels|open_sector")).toBe(0);
    expect(countCalls("plugin:levels|read_sector")).toBe(0);
  });

  it("brings the sectors near the camera into residency", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 5), outlineAt(1, 1000)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    // Without the floor, so this is about the distances rather than about never drawing nothing.
    service.residency = { ...service.residency, minSectors: 0 };

    const delivered: Array<ILevelSectorDelivery> = [];

    service.sectors.subscribe((change) => delivered.push(...change.delivered));

    await service.stream(ORIGIN);

    expect(service.sectorReport.held).toEqual([0]);
    // Handed on as the pack and its bytes. Building a geometry from them belongs to whichever side draws, which
    // is why nothing here holds one.
    expect(delivered.map((it) => it.sector)).toEqual([0]);
    expect(delivered[0].buffer.byteLength).toBe(description.bufferLength);
  });

  // Framing a level puts the camera outside it, so a policy that loaded only what was within a fixed distance opened
  // every level larger than that distance on an empty screen. That is what shipped, and this is the guard.
  it("draws something even when the camera is outside the whole level", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 100_000), outlineAt(1, 200_000)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);
    await service.stream(ORIGIN);

    expect(service.sectorReport.held.length).toBeGreaterThan(0);
    expect(service.sectorReport.held).toContain(0);
  });

  // The geometry is device memory, and it is held by whichever side draws. What this side owes that side is to
  // say which sector went, so it can dispose it.
  it("says which sector the camera has left", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 0), outlineAt(1, 20_000)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    service.residency = { ...service.residency, maxSectors: 1, minSectors: 1 };

    await service.stream(ORIGIN);

    const released: Array<number> = [];

    service.sectors.subscribe((change) => released.push(...(change.released ?? [])));

    // Flown to the far sector, which is now the nearest and takes the only place in the budget.
    await service.stream({ x: 20_000, y: 0, z: 0 });

    expect(service.sectorReport.held).toEqual([1]);
    expect(released).toEqual([0]);
  });

  // A camera that has not moved far enough to change what is resident should cost nothing at all.
  it("reads nothing when the plan asks for no change", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 5)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);
    await service.stream(ORIGIN);
    await service.stream(ORIGIN);

    expect(countCalls("plugin:levels|open_sector")).toBe(1);
  });

  it("holds no more sectors than its budget allows", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 2), outlineAt(1, 3), outlineAt(2, 4)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    service.residency = { ...service.residency, maxSectors: 2 };

    await service.stream(ORIGIN);

    expect(service.sectorReport.held).toHaveLength(2);
  });

  // Streaming is a latest-wins flow, so a camera that keeps moving cancels reads midway. What those reads had
  // already uploaded must not outlive the sectors that never arrived.
  it("keeps only the textures its resident sectors name", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 0), outlineAt(1, 20_000)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);
    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: mockDdsFile(),
      ["plugin:levels|open_level"]: mockSessionResponse(level),
      ["plugin:levels|open_sector"]: mockSessionResponse((args?: Record<string, unknown>) => ({
        ...description,
        sector: args?.sector as number,
      })),
      ["plugin:levels|read_sector"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    service.residency = { ...service.residency, maxSectors: 1, minSectors: 1 };

    const supply = recordSupply(service);

    await service.stream(ORIGIN);

    expect(supply.delivered.map((it) => it.reference)).toEqual(["stone"]);

    await service.stream({ x: 20_000, y: 0, z: 0 });

    // The second sector names the same fixture surface, so what it says to keep is what that one names rather
    // than what the first one left behind.
    expect(Array.from(supply.retained.at(-1) ?? [])).toEqual(["stone"]);
  });

  it("releases every texture when the level is closed", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 0)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);
    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: mockDdsFile(),
      ["plugin:levels|open_level"]: mockSessionResponse(level),
      ["plugin:levels|open_sector"]: mockSessionResponse(description),
      ["plugin:levels|read_sector"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);
    await service.stream(ORIGIN);

    const supply = recordSupply(service);

    service.clear();

    // Null rather than an empty set: the whole set went, which is a different thing from keeping none of it.
    expect(supply.retained).toEqual([null]);
    expect(service.sectorReport.held).toHaveLength(0);
  });

  it("streams nothing when no level is open", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    await service.stream(ORIGIN);

    expect(service.sectorReport.held).toHaveLength(0);
    expect(countCalls("plugin:levels|open_sector")).toBe(0);
  });

  it("records a failure as state rather than throwing it at the caller", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({
      ["plugin:levels|open_level"]: mockSessionResponse(() => {
        throw new Error("level carries no visuals chunk, so it draws nothing");
      }),
    });

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    expect(service.level.value).toBeNull();
    expect(service.level.error?.message).toBe("level carries no visuals chunk, so it draws nothing");
  });
});

describe("LevelLoadService streaming progress", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("reports nothing in flight before anything is asked for", () => {
    const { service } = mockInjectedService(LevelLoadService);

    expect(service.streaming).toEqual(IDLE_LEVEL_STREAM);
    expect(service.isStreaming).toBe(false);
  });

  it("counts the sectors of a plan as they arrive", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1), outlineAt(1, 2), outlineAt(2, 3)]);
    const { service } = mockInjectedService(LevelLoadService);
    const seen: Array<string> = [];

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    const stop = reaction(
      () => service.streaming,
      (progress) => seen.push(`${progress.loaded}/${progress.total}`)
    );

    await service.stream(ORIGIN);

    stop();

    // The last increment and the reset land in one tick, so the count runs out at the sector before the last and
    // then goes idle rather than showing a full bar nobody sees.
    expect(seen).toEqual(["0/3", "1/3", "2/3", "0/0"]);
  });

  it("stops reporting a read that is not coming once the plan is done", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);
    await service.stream(ORIGIN);

    expect(service.streaming).toEqual(IDLE_LEVEL_STREAM);
    expect(service.isStreaming).toBe(false);
  });

  // A failed read would otherwise leave a viewer showing progress towards sectors that will never land.
  it("stops reporting when a read fails", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    setMockInvokeResponses({
      ["plugin:levels|open_sector"]: mockSessionResponse(() => {
        throw new Error("sector geometry was not read");
      }),
    });

    await service.stream(ORIGIN);

    expect(service.streaming).toEqual(IDLE_LEVEL_STREAM);
    expect(service.sectorReport.held).toHaveLength(0);
  });

  // One sector that cannot be read is one sector missing, not a reason to abandon the rest of what the camera is
  // standing in.
  it("reads the rest of a plan past a sector that fails", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1), outlineAt(1, 2)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    setMockInvokeResponses({
      ["plugin:levels|open_sector"]: mockSessionResponse((args?: Record<string, unknown>) => {
        if (args?.sector === 0) {
          throw new Error("sector geometry was not read");
        }

        return { ...description, sector: args?.sector as number };
      }),
      ["plugin:levels|read_sector"]: buffer,
    });

    await service.stream(ORIGIN);

    expect(service.sectorReport.held).toEqual([1]);
  });

  // Stage 2's win, and the reason the flow became a scheduler: pack, transfer and adoption of different sectors
  // now overlap. Read one at a time they came to 48 ms each, which capped streaming at about twenty sectors a
  // second however fast the camera moved.
  it("reads several sectors at once", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1), outlineAt(1, 2), outlineAt(2, 3)]);
    const { service } = mockInjectedService(LevelLoadService);
    const { held, release } = createHeldCall();

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    setMockInvokeResponses({
      ["plugin:levels|open_sector"]: mockSessionResponse(async (args?: Record<string, unknown>) => {
        await held;

        return { ...description, sector: args?.sector as number };
      }),
      ["plugin:levels|read_sector"]: buffer,
    });

    const streaming: Promise<void> = service.stream(ORIGIN);

    await Promise.resolve();

    // All three are packing before any of them has come back, which is what the default concurrency is for.
    expect(countCalls("plugin:levels|open_sector")).toBe(3);

    release();

    await streaming;

    expect([...service.sectorReport.held].sort()).toEqual([0, 1, 2]);
  });

  // The defect that broke opening a level once one had already been open. The reads of the last level were still
  // in flight, holding its sector numbers - and the new level's sectors carry the same numbers, so they were
  // filtered out of its own queue and then waited for reads the reader had already dropped. Nothing near the
  // camera arrived; flying somewhere with different numbers was the only thing that looked like it worked.
  it("reads the sectors of a level opened while the last one was still arriving", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1), outlineAt(1, 2)]);
    const { service } = mockInjectedService(LevelLoadService);
    const { held, release } = createHeldCall();

    setMockInvokeResponses({
      ["plugin:levels|open_level"]: mockSessionResponse(level),
      ["plugin:levels|open_sector"]: mockSessionResponse(async (args?: Record<string, unknown>) => {
        await held;

        return { ...description, sector: args?.sector as number };
      }),
      ["plugin:levels|read_sector"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    // Left running: its reads are still packing when the next level opens, which is what a person clicking
    // through the picker does.
    void service.stream(ORIGIN);

    await Promise.resolve();

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\jupiter" }, ROOTS);
    await service.stream(ORIGIN);

    release();

    expect([...service.sectorReport.held].sort()).toEqual([0, 1]);
  });

  // What the owner asked for: a camera at full boost crosses 2400 metres a second, so the only way to have a
  // sector when it arrives is to have read it beforehand. Once what the camera asked for has landed, the rest of
  // the level follows, nearest first.
  it("reads the rest of the level once the camera has what it asked for", async () => {
    const { level, description, buffer } = mockStreamable([
      outlineAt(0, 1),
      outlineAt(1, 20_000),
      outlineAt(2, 40_000),
    ]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    service.residency = { ...service.residency, maxSectors: 1, minSectors: 1 };

    await service.stream(ORIGIN);

    expect(service.sectorReport.held).toEqual([0]);

    // The two far sectors are nowhere near the camera and are read anyway, because the level fits and a sector
    // already held is one the camera never waits for.
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect([...service.sectorReport.held].sort()).toEqual([0, 1, 2]);
  });

  it("does not fill the level in when it is told not to", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1), outlineAt(1, 20_000)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    service.residency = { ...service.residency, isPreloaded: false, maxSectors: 1, minSectors: 1 };

    await service.stream(ORIGIN);
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(service.sectorReport.held).toEqual([0]);
  });

  // A report no longer ends in the level's housekeeping, so one that changes nothing costs a plan and nothing else.
  it("does nothing at all for a camera report that changes what is wanted not at all", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);
    await service.stream(ORIGIN);

    const reads: number = countCalls("plugin:levels|open_sector");
    const seen: Array<unknown> = [];
    const stop = reaction(
      () => service.streaming,
      (progress) => seen.push(progress)
    );

    await service.stream(ORIGIN);

    stop();

    expect(countCalls("plugin:levels|open_sector")).toBe(reads);
    expect(seen).toEqual([]);
  });

  // The defect this exists for: a camera moving while a big sector packs used to cancel the read it was waiting on,
  // leave the backend packing anyway, and ask again on the next move - so the queue grew faster than it drained and
  // the level never appeared.
  it("joins the read already in flight rather than asking for the same sector again", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1)]);
    const { service } = mockInjectedService(LevelLoadService);
    const { held, release } = createHeldCall();

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    setMockInvokeResponses({
      ["plugin:levels|open_sector"]: mockSessionResponse(async (args?: Record<string, unknown>) => {
        await held;

        return { ...description, sector: args?.sector as number };
      }),
      ["plugin:levels|read_sector"]: buffer,
    });

    // Three camera reports while the first pack is still in flight, which is what flying through a level does.
    const streams: Array<unknown> = [service.stream(ORIGIN), service.stream(ORIGIN), service.stream(ORIGIN)];

    release();

    await Promise.all(streams);

    expect(countCalls("plugin:levels|open_sector")).toBe(1);
    expect(service.sectorReport.held).toEqual([0]);
  });

  // The defect: a read uploads its textures one at a time and adopts its sector only once every one of them has
  // landed, so between the first upload and the adoption the sector is in no residency. Any other flow settling in
  // that window computed what to keep without it and disposed exactly what it had just uploaded, and the sector then
  // arrived dressed in disposed textures and drew white. A refresh hid it, because a restore reads it all again.
  it("keeps the textures of a read still in flight, which are not resident yet", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 0), outlineAt(1, 20_000)]);
    const { service } = mockInjectedService(LevelLoadService);
    const { held, release } = createHeldCall();
    const { held: reached, release: reach } = createHeldCall();

    // Sector 0 names two, because the window only exists while one of a sector's textures is up and another is not.
    // Sector 1 names neither, so what it keeps resident cannot be what sector 0 is waiting on.
    function textured(sector: number): Array<SectorDescription["sections"][number]> {
      return (sector === 0 ? ["stone", "grass"] : ["rock"]).map((textureName) => ({
        ...description.sections[0],
        surface: { ...description.sections[0].surface, textureName },
      }));
    }

    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: async (args?: Record<string, unknown>) => {
        if (String(args?.logicalPath).includes("grass")) {
          reach();

          await held;
        }

        return mockDdsFile();
      },
      ["plugin:levels|open_level"]: mockSessionResponse({
        ...level,
        textures: ["stone", "grass", "rock"].map((it) => mockLevelTextureReference(it)),
      }),
      ["plugin:levels|open_sector"]: mockSessionResponse((args?: Record<string, unknown>) => ({
        ...description,
        sections: textured(args?.sector as number),
        sector: args?.sector as number,
      })),
      ["plugin:levels|read_sector"]: buffer,
    });

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    service.residency = { ...service.residency, maxSectors: 1, minSectors: 1 };

    const supply = recordSupply(service);

    // Left running, and never awaited: the move below supersedes this flow, so its own promise is not what settles
    // when the read it started finishes.
    void service.stream(ORIGIN);

    await reached;
    // Two turns past the ask, so `stone` has resolved and been recorded while `grass` is still outstanding.
    await Promise.resolve();
    await Promise.resolve();

    // A second move settles while that read is still in the window, and takes everything not resident with it.
    await service.stream({ x: 20_000, y: 0, z: 0 });

    release();

    await new Promise((resolve) => setTimeout(resolve, 0));

    // The file was read before the move and is still supplied, so the sector arrives with something to draw
    // with rather than with a texture released out from under it.
    expect(supply.delivered.map((it) => it.reference)).toContain("stone");
    expect(supply.retained.some((it) => it?.has("stone"))).toBe(true);
  });
});

describe("LevelLoadService texture supply", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  // The files are read here and uploaded wherever the level is drawn, so what crosses is which references moved
  // and their bytes - never a texture, and never a count that means only that something changed.
  it("names the files a sector needed rather than saying that something changed", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    const supply = recordSupply(service);

    await service.stream(ORIGIN);

    expect(supply.delivered.map((it) => it.reference)).toEqual(["stone"]);
  });

  // There is no reference to name for a level opening: the answer is the whole set.
  it("says the whole set went when a level opens", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    const supply = recordSupply(service);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    expect(supply.retained).toEqual([null]);
    expect(supply.delivered).toEqual([]);
  });

  // A file is read once for the level. Reading it again for every sector that names it would spend the round trip
  // the whole ledger exists to avoid.
  it("reads a file once however many sectors name it", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1), outlineAt(1, 2)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    const supply = recordSupply(service);

    await service.stream(ORIGIN);

    // One delivery for two sectors that name the same reference, which is the read that did not happen twice.
    expect(supply.delivered.map((it) => it.reference)).toEqual(["stone"]);
  });

  // Nothing arrived and nothing was released, so nothing is re-dressed.
  it("supplies nothing when a camera has not moved far enough to change anything", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 1)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);
    await service.stream(ORIGIN);

    const supply = recordSupply(service);

    await service.stream(ORIGIN);

    expect(supply.delivered).toEqual([]);
    expect(supply.retained).toEqual([]);
  });
});
