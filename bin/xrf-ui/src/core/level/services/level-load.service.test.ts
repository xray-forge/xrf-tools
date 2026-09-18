import { beforeEach, describe, expect, it } from "@jest/globals";
import { isObservableProp, reaction } from "@wirestate/mobx";

import { createRoots } from "@/core/assets/lib";
import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { SectorDescription, SectorOutline } from "@/core/ipc/types/xrf-visual";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import { mockSectorDescription, mockSectorOutline, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
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

describe("LevelLoadService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(LevelLoadService);

    expect(isObservableProp(service, "level")).toBe(true);
    expect(isObservableProp(service, "sectors")).toBe(true);
    expect(isObservableProp(service, "residency")).toBe(true);
  });

  // The whole point of the open: a quarter of a gigabyte of geometry stays on disk until a camera asks for a piece.
  it("opens a level without reading any geometry", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 5)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    expect(service.level.value?.selected.value.sectors).toHaveLength(1);
    expect(service.sectors.size).toBe(0);
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

    await service.stream(ORIGIN);

    expect(Array.from(service.sectors.keys())).toEqual([0]);
    expect(service.sectors.get(0)?.geometry.getAttribute("position").count).toBe(3);
  });

  // Framing a level puts the camera outside it, so a policy that loaded only what was within a fixed distance opened
  // every level larger than that distance on an empty screen. That is what shipped, and this is the guard.
  it("draws something even when the camera is outside the whole level", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 100_000), outlineAt(1, 200_000)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);
    await service.stream(ORIGIN);

    expect(service.sectors.size).toBeGreaterThan(0);
    expect(Array.from(service.sectors.keys())).toContain(0);
  });

  // The geometry is device memory, so dropping the reference is not enough: what the camera leaves has to be disposed.
  it("disposes the geometry of a sector the camera has left", async () => {
    const { level, description, buffer } = mockStreamable([outlineAt(0, 0), outlineAt(1, 20_000)]);
    const { service } = mockInjectedService(LevelLoadService);

    armLevel(level, description, buffer);

    await service.load({ kind: "asset", logicalPath: "levels\\zaton" }, ROOTS);

    service.residency = { ...service.residency, maxSectors: 1, minSectors: 1 };

    await service.stream(ORIGIN);

    const geometry = service.sectors.get(0)?.geometry;

    let disposed: boolean = false;

    geometry?.addEventListener("dispose", () => {
      disposed = true;
    });

    // Flown to the far sector, which is now the nearest and takes the only place in the budget.
    await service.stream({ x: 20_000, y: 0, z: 0 });

    expect(Array.from(service.sectors.keys())).toEqual([1]);
    expect(disposed).toBe(true);
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

    expect(service.sectors.size).toBe(2);
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

    await service.stream(ORIGIN);

    expect(service.textures.size).toBeGreaterThan(0);

    await service.stream({ x: 20_000, y: 0, z: 0 });

    // The second sector names the same fixture surface, so what survives is what it names rather than what the first
    // one left behind.
    expect(service.textures.size).toBe(1);
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

    service.clear();

    expect(service.textures.size).toBe(0);
    expect(service.sectors.size).toBe(0);
  });

  it("streams nothing when no level is open", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    await service.stream(ORIGIN);

    expect(service.sectors.size).toBe(0);
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

    await expect(service.stream(ORIGIN)).rejects.toThrow();

    expect(service.streaming).toEqual(IDLE_LEVEL_STREAM);
  });
});
