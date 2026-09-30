import { describe, expect, it, jest } from "@jest/globals";

import { LevelSpawnModelDescription, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { ILevelSpawnDelivery } from "@/core/level/lib/render/level-render-protocol";
import { LEVEL_SPAWN_BATCH, LevelSpawnReader } from "@/core/level/lib/spawn/level-spawn-reader";
import { ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { setMockBulkResponses } from "@/fixtures/mocks/bulk.mocks";
import { mockLevelSpawnModel, mockLevelSpawnObject, mockLevelTextureReference } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";

interface IMockHost {
  reader: LevelSpawnReader;
  delivered: Array<ILevelSpawnDelivery>;
  notes: Array<ILevelSpawnReport>;
  supplied: Array<Array<string>>;
  close: () => void;
}

function mockHost(): IMockHost {
  const delivered: Array<ILevelSpawnDelivery> = [];
  const notes: Array<ILevelSpawnReport> = [];
  const supplied: Array<Array<string>> = [];
  const state: { isOpen: boolean } = { isOpen: true };
  const reader: LevelSpawnReader = new LevelSpawnReader({
    deliver: (delivery: ILevelSpawnDelivery) => delivered.push(delivery),
    isOpen: () => state.isOpen,
    note: (report: ILevelSpawnReport) => notes.push(report),
    supply: async (_, textures) => {
      supplied.push(textures.map((it) => it.reference));
    },
  });

  return { close: () => (state.isOpen = false), delivered, notes, reader, supplied };
}

/** One object standing as each of `count` visuals, named `visual-<index>`. */
function mockObjects(count: number): LevelSpawnObjectsDescription {
  const visuals: Array<string> = Array.from({ length: count }, (_, index: number) => `visual-${index}`);

  return {
    objects: visuals.map((name: string, index: number) => mockLevelSpawnObject({ index, name, visual: index })),
    visuals,
  };
}

/** Describes every visual asked for, each binding a texture named as it, but those named in `failing`. */
function armModels(objects: LevelSpawnObjectsDescription, failing: ReadonlyArray<string> = []): void {
  setMockInvokeResponses({
    ["plugin:levels|describe_spawn_models"]: mockSessionResponse((args?: Record<string, unknown>) => {
      const names: Array<string> = args?.names as Array<string>;

      return {
        failures: names.filter((it) => failing.includes(it)).map((name) => ({ name, reason: "Malformed" })),
        hemi: names.map((name) => ({ cube: [1, 1, 1, 1, 1, 1], index: Number(name.split("-")[1]) })),
        models: names
          .filter((it) => !failing.includes(it))
          .map((name): LevelSpawnModelDescription => ({
            ...mockLevelSpawnModel(name).description,
            textures: [mockLevelTextureReference(name)],
          })),
      };
    }),
    ["plugin:levels|open_spawn_objects"]: mockSessionResponse(objects),
  });
  setMockBulkResponses({
    "levels/read_spawn_model": (args: Record<string, unknown>) => mockLevelSpawnModel(String(args.name)).buffer,
  });
}

function countDescribes(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === "plugin:levels|describe_spawn_models").length;
}

describe("LevelSpawnReader", () => {
  it("hands the objects on with every model read so far, a batch at a time", async () => {
    const objects: LevelSpawnObjectsDescription = mockObjects(LEVEL_SPAWN_BATCH + 2);
    const host: IMockHost = mockHost();

    armModels(objects);

    const report: ILevelSpawnReport | null = await host.reader.read("session");

    expect(countDescribes()).toBe(2);
    expect(host.delivered.map((it) => it.models.size)).toEqual([LEVEL_SPAWN_BATCH, LEVEL_SPAWN_BATCH + 2]);
    expect(host.delivered.every((it) => it.objects === objects)).toBe(true);
    expect(host.delivered.at(-1)?.models.get(LEVEL_SPAWN_BATCH + 1)?.description.name).toBe(
      `visual-${LEVEL_SPAWN_BATCH + 1}`
    );
    // Each supply claims everything read so far, so the first batch's textures stay claimed through the second.
    expect(host.supplied.map((it) => it.length)).toEqual([LEVEL_SPAWN_BATCH, LEVEL_SPAWN_BATCH + 2]);
    expect(host.notes.map((it) => it.read)).toEqual([0, LEVEL_SPAWN_BATCH, LEVEL_SPAWN_BATCH + 2]);
    expect(report).toEqual({
      failure: null,
      failures: [],
      objects: LEVEL_SPAWN_BATCH + 2,
      read: LEVEL_SPAWN_BATCH + 2,
      visuals: LEVEL_SPAWN_BATCH + 2,
    });
    // Each object's cube arrives with its visual's batch, and is kept with every later one.
    expect(host.delivered[0].hemi.size).toBe(LEVEL_SPAWN_BATCH);
    expect(host.delivered.at(-1)?.hemi.get(LEVEL_SPAWN_BATCH + 1)).toEqual([1, 1, 1, 1, 1, 1]);
  });

  it("lists a visual the backend could not read, and holds the others", async () => {
    const objects: LevelSpawnObjectsDescription = mockObjects(3);
    const host: IMockHost = mockHost();

    armModels(objects, ["visual-1"]);

    const report: ILevelSpawnReport | null = await host.reader.read("session");

    expect(Array.from(host.delivered.at(-1)?.models.keys() ?? [])).toEqual([0, 2]);
    expect(report?.failures).toEqual([{ name: "visual-1", reason: "Malformed" }]);
  });

  // A pack read against another description would draw from offsets into the wrong bytes.
  it("refuses a model whose pack is not the one its description covers", async () => {
    const objects: LevelSpawnObjectsDescription = mockObjects(1);
    const host: IMockHost = mockHost();

    armModels(objects);
    setMockBulkResponses({ "levels/read_spawn_model": new ArrayBuffer(4) });

    const report: ILevelSpawnReport | null = await host.reader.read("session");

    expect(host.delivered.at(-1)?.models.size).toBe(0);
    expect(report?.failures.map((it) => it.name)).toEqual(["visual-0"]);
  });

  it("lists every visual of a batch that could not be read at all, and reads on", async () => {
    const objects: LevelSpawnObjectsDescription = mockObjects(LEVEL_SPAWN_BATCH + 1);
    const host: IMockHost = mockHost();
    const describe = jest.fn(async (args?: Record<string, unknown>) => {
      if ((args?.names as Array<string>).includes("visual-0")) {
        throw new Error("The level's spawned visuals are unavailable");
      }

      return { failures: [], hemi: [], models: [mockLevelSpawnModel(`visual-${LEVEL_SPAWN_BATCH}`).description] };
    });

    armModels(objects);
    setMockInvokeResponses({
      ["plugin:levels|describe_spawn_models"]: mockSessionResponse(describe),
      ["plugin:levels|open_spawn_objects"]: mockSessionResponse(objects),
    });

    const report: ILevelSpawnReport | null = await host.reader.read("session");

    expect(report?.failures).toHaveLength(LEVEL_SPAWN_BATCH);
    expect(report?.failures[0].reason).toContain("unavailable");
    expect(Array.from(host.delivered.at(-1)?.models.keys() ?? [])).toEqual([LEVEL_SPAWN_BATCH]);
  });

  it("stops at the first step after the level closes, handing nothing on", async () => {
    const objects: LevelSpawnObjectsDescription = mockObjects(LEVEL_SPAWN_BATCH + 1);
    const host: IMockHost = mockHost();

    armModels(objects);
    setMockBulkResponses({
      "levels/read_spawn_model": (args: Record<string, unknown>) => {
        host.close();

        return mockLevelSpawnModel(String(args.name)).buffer;
      },
    });

    expect(await host.reader.read("session")).toBeNull();
    expect(host.delivered).toEqual([]);
    expect(countDescribes()).toBe(1);
  });

  // A spawn the backend cannot read leaves the level without any object, which is noted rather than only logged.
  it("notes why the spawn could not be read, and reads no model", async () => {
    const host: IMockHost = mockHost();

    setMockInvokeResponses({
      ["plugin:levels|open_spawn_objects"]: () => {
        throw new Error("Failed to read 'spawns\\all.spawn': truncated chunk");
      },
    });

    expect(await host.reader.read("session")).toBeNull();
    expect(host.notes).toEqual([
      expect.objectContaining({ failure: "Failed to read 'spawns\\all.spawn': truncated chunk" }),
    ]);
    expect(countDescribes()).toBe(0);
  });

  it("reads nothing for a level drawing no object", async () => {
    const host: IMockHost = mockHost();

    armModels(mockObjects(0));

    expect(await host.reader.read("session")).toBeNull();
    expect(countDescribes()).toBe(0);
    expect(host.notes).toEqual([]);
  });
});
