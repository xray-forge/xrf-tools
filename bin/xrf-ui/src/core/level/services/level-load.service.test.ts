import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";
import { isObservableProp } from "@wirestate/mobx";

import { createRoots } from "@/core/assets/lib";
import { LevelSource, LevelSpawnObjectsDescription, SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { EXrayEngineChoice } from "@/core/ipc/types/xrf-engine-target";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { EMPTY_LEVEL_SPAWN_REPORT, isLevelSpawnReading } from "@/core/level/lib/spawn/level-spawn-report";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockLevelSpawnObject, mockSectorOutline, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { LevelLoadService } from "./level-load.service";

const ROOTS: XrayRoots = createRoots(["C:\\game\\db"]);
const SOURCE: LevelSource = { kind: "asset", logicalPath: "levels\\zaton" };
const OTHER_SOURCE: LevelSource = { kind: "asset", logicalPath: "levels\\jupiter" };

const SPAWN: LevelSpawnObjectsDescription = {
  objects: [mockLevelSpawnObject({ index: 0, visual: 0 }), mockLevelSpawnObject({ index: 1, visual: 0 })],
  visuals: ["dynamics\\box"],
};

function countCalls(command: string): number {
  return mockInvoke.mock.calls.filter(([name]) => name === command).length;
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

/** Arms an open of a one-sector level whose spawn lists the two objects above. */
function armLevel(level: SelectedLevelDescription = mockSelectedLevelDescription({ sectors: [mockSectorOutline()] })) {
  setMockInvokeResponses({
    ["plugin:levels|close_level"]: null,
    ["plugin:levels|open_level"]: mockSessionResponse(level),
    ["plugin:levels|open_spawn_objects"]: mockSessionResponse(SPAWN),
  });
}

describe("LevelLoadService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  // A refresh re-provisions the service while the backend keeps its session, which is the whole reason the backend
  // keeps it: coming back to an empty picker beside a level that is still open reads as having lost it.
  it("takes back the level the backend still has open, without opening it again", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({
      ["plugin:levels|get_level"]: mockSessionResponse(
        mockSelectedLevelDescription({ sectors: [mockSectorOutline()] })
      ),
      ["plugin:levels|open_spawn_objects"]: mockSessionResponse(SPAWN),
    });

    await service.restore();

    expect(service.level.value?.selected.value.sectors).toHaveLength(1);
    expect(countCalls("plugin:levels|open_level")).toBe(0);
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
    expect(isObservableProp(service, "opening")).toBe(true);
    expect(isObservableProp(service, "spawn")).toBe(true);
    expect(isObservableProp(service, "spawnReport")).toBe(true);
  });

  // Named only once open, the level's header would come in with it and shrink the viewport the renderer just sized.
  it("names the level it is opening until it is open", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    armLevel();

    const opened: unknown = service.load({
      engine: EXrayEngineChoice.VANILLA,
      isDltx: false,
      roots: ROOTS,
      source: SOURCE,
    });

    expect(service.opening).toEqual(SOURCE);

    await opened;

    expect(service.opening).toBeNull();
  });

  // The renderer reads the level's geometry, grass, lights and models itself: the page asks for none of it.
  it("opens a level and lists its spawned objects, reading nothing the renderer draws", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    armLevel();

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: SOURCE });
    await flush();

    expect(service.level.value?.selected.value.sectors).toHaveLength(1);
    expect(service.spawn).toEqual(SPAWN);
    expect(service.spawnReport).toEqual({ failure: null, isListed: true, objects: 2, visuals: 1 });
    expect(isLevelSpawnReading(service.spawnReport)).toBe(false);
    // The level itself, then each part the page shows read on its own, side by side.
    expect(mockInvoke.mock.calls.map(([name]) => name)).toEqual([
      "plugin:sessions|bind",
      "plugin:levels|open_level",
      "plugin:levels|describe_start",
      "plugin:levels|describe_level_map",
      "plugin:levels|describe_surfaces",
      "plugin:levels|describe_textures",
      "plugin:levels|open_spawn_objects",
    ]);
  });

  it("says why the spawn could not be listed, and lists nothing", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({
      ["plugin:levels|open_level"]: mockSessionResponse(mockSelectedLevelDescription()),
      ["plugin:levels|open_spawn_objects"]: mockSessionResponse(() => {
        throw new Error("all.spawn names no level graph");
      }),
    });

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: SOURCE });
    await flush();

    expect(service.spawn).toBeNull();
    expect(service.spawnReport).toEqual({
      ...EMPTY_LEVEL_SPAWN_REPORT,
      failure: "all.spawn names no level graph",
      isListed: true,
    });
  });

  it("forgets the level and its spawn once closed", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    armLevel();

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: SOURCE });
    await flush();
    await service.close();

    expect(service.level.value).toBeNull();
    expect(service.spawn).toBeNull();
    expect(service.spawnReport).toEqual(EMPTY_LEVEL_SPAWN_REPORT);
  });

  // The close is fired and forgotten: a failure thrown at it would go unheard and leave the level on screen.
  it("forgets the level even where the backend fails to close it, and releases it again on the next close", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    armLevel();

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: SOURCE });
    await flush();

    setMockInvokeResponses({
      ["plugin:levels|close_level"]: () => Promise.reject(new Error("The level is busy")),
    });

    await expect(service.close()).resolves.toBeUndefined();
    expect(service.level.value).toBeNull();
    expect(service.level.isLoading).toBe(false);

    setMockInvokeResponses({ ["plugin:levels|close_level"]: null });
    await service.close();

    expect(countCalls("plugin:levels|close_level")).toBe(2);
  });

  it("tells a close the backend failed as a notice", async () => {
    const { container, service } = mockInjectedService(LevelLoadService);
    const raised: Array<INotificationPayload> = [];

    container
      .get(EventBus)
      .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
        raised.push(event.payload as INotificationPayload)
      );

    armLevel();

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: SOURCE });
    await flush();

    setMockInvokeResponses({
      ["plugin:levels|close_level"]: () => Promise.reject(new Error("The level is busy")),
    });

    await service.close();

    expect(raised).toEqual([
      expect.objectContaining({ severity: ENotificationSeverity.ERROR, title: "Could not close the level" }),
    ]);
    expect(raised[0].details).toContain("The level is busy");
  });

  // Showing the level it was asked to replace, with no word, reads as the open having worked.
  it("closes the open level when another fails to open, and holds the failure", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    armLevel();

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: SOURCE });
    await flush();

    setMockInvokeResponses({
      ["plugin:levels|open_level"]: mockSessionResponse(() => {
        throw new Error("level carries no visuals chunk, so it draws nothing");
      }),
    });

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: OTHER_SOURCE });

    expect(service.level.value).toBeNull();
    expect(service.level.error?.message).toBe("level carries no visuals chunk, so it draws nothing");
    expect(service.spawn).toBeNull();
    expect(countCalls("plugin:levels|close_level")).toBe(1);
  });

  it("opens the last level asked for again on a retry", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({
      ["plugin:levels|open_level"]: mockSessionResponse(() => {
        throw new Error("The archive is locked");
      }),
    });

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: SOURCE });

    expect(service.level.isFailed).toBe(true);

    armLevel();

    await service.retry();

    expect(service.level.value?.selected.value.sectors).toHaveLength(1);
    expect(mockInvoke.mock.calls.filter(([name]) => name === "plugin:levels|open_level").at(-1)?.[1]).toEqual(
      expect.objectContaining({ request: expect.objectContaining({ source: SOURCE }) })
    );
  });

  it("records a failure as state rather than throwing it at the caller", async () => {
    const { service } = mockInjectedService(LevelLoadService);

    setMockInvokeResponses({
      ["plugin:levels|open_level"]: mockSessionResponse(() => {
        throw new Error("level carries no visuals chunk, so it draws nothing");
      }),
    });

    await service.load({ engine: EXrayEngineChoice.VANILLA, isDltx: false, roots: ROOTS, source: SOURCE });

    expect(service.level.value).toBeNull();
    expect(service.level.error?.message).toBe("level carries no visuals chunk, so it draws nothing");
  });
});
