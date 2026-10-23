import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container, EventBus } from "@wirestate/core";

import { createRoots } from "@/core/assets/lib";
import { SESSION_CHANGE_EVENT } from "@/core/ipc/session";
import {
  LEVEL_ENVIRONMENT_PART,
  LEVEL_SPAWN_PART,
  LevelSpawnObjectsDescription,
  SessionChange,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { EXrayEngineChoice } from "@/core/ipc/types/xrf-engine-target";
import { mockLevelSpawnObject, mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { InvokeHandler, mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";

import { LevelLoadService } from "./level-load.service";

const LIST_SPAWN: string = "plugin:levels|open_spawn_objects";

function listing(objects: number): LevelSpawnObjectsDescription {
  return {
    objects: Array.from({ length: objects }, (_, index: number) => mockLevelSpawnObject({ index, visual: 0 })),
    visuals: ["dynamics\\box"],
  };
}

async function flush(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

function countListings(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === LIST_SPAWN).length;
}

/** Reads of the committed level, the restore at provision included. */
function countReads(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === "plugin:levels|get_level").length;
}

/** A spawn listing at `revision`, for the level the call names. */
function listedAt(objects: number, revision: number): InvokeHandler {
  return (args) => mockSessionSnapshot(listing(objects), args?.sessionId as string, revision);
}

interface IOpenedLevel {
  container: Container;
  service: LevelLoadService;
  sessionId: string;
}

/** Provisions the loader, as the level viewer does, and opens a level whose spawn lists one object. */
async function openLevel(): Promise<IOpenedLevel> {
  const container: Container = mockContainer([LevelLoadService]);
  const service: LevelLoadService = container.get(LevelLoadService);

  setMockInvokeResponses({
    ["plugin:levels|get_level"]: null,
    ["plugin:levels|open_level"]: mockSessionResponse(mockSelectedLevelDescription()),
    [LIST_SPAWN]: listedAt(1, 0),
  });

  container.provision();
  await flush();
  await service.load({
    engine: EXrayEngineChoice.VANILLA,
    isDltx: false,
    roots: createRoots(["C:\\game"]),
    source: { kind: "asset", logicalPath: "levels\\zaton" },
  });
  await flush();

  return { container, service, sessionId: service.level.value?.selected.sessionId as string };
}

function notify(container: Container, change: SessionChange): void {
  container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, change);
}

describe("LevelLoadService changes", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("lists the spawn again once, when a change lets it go", async () => {
    const { container, service, sessionId } = await openLevel();
    const before: number = countListings();

    setMockInvokeResponses({ [LIST_SPAWN]: listedAt(3, 1) });
    notify(container, { sessionId, revision: 1, root: { kind: "kept" }, staleParts: [LEVEL_SPAWN_PART] });
    await flush();

    expect(countListings() - before).toBe(1);
    expect(service.spawn?.objects).toHaveLength(3);
    expect(service.spawnRevision).toBe(1);
    expect(service.isStale).toBe(false);
  });

  it("lists nothing again for another level's change, or for parts the panels do not show", async () => {
    const { container, service, sessionId } = await openLevel();
    const before: number = countListings();

    notify(container, { sessionId: "another", revision: 1, root: { kind: "kept" }, staleParts: [LEVEL_SPAWN_PART] });
    notify(container, { sessionId, revision: 1, root: { kind: "kept" }, staleParts: [LEVEL_ENVIRONMENT_PART] });
    await flush();

    expect(countListings()).toBe(before);
    expect(service.spawn?.objects).toHaveLength(1);
  });

  it("drops a listing read at an older revision than the one shown", async () => {
    const { container, service, sessionId } = await openLevel();
    const answers: Array<(snapshot: SessionSnapshot<LevelSpawnObjectsDescription>) => void> = [];

    setMockInvokeResponses({ [LIST_SPAWN]: () => new Promise((resolve) => answers.push(resolve)) });

    notify(container, { sessionId, revision: 1, root: { kind: "kept" }, staleParts: [LEVEL_SPAWN_PART] });
    notify(container, { sessionId, revision: 2, root: { kind: "kept" }, staleParts: [LEVEL_SPAWN_PART] });
    await flush();

    answers[1](mockSessionSnapshot(listing(2), sessionId, 2));
    await flush();
    answers[0](mockSessionSnapshot(listing(5), sessionId, 1));
    await flush();

    expect(service.spawn?.objects).toHaveLength(2);
    expect(service.spawnRevision).toBe(2);
  });

  it("reads a level its session derived anew once, and lists its spawn again", async () => {
    const { container, service, sessionId } = await openLevel();
    const listings: number = countListings();

    setMockInvokeResponses({
      ["plugin:levels|get_level"]: () =>
        mockSessionSnapshot(mockSelectedLevelDescription({ visuals: 7 }), sessionId, 1),
      [LIST_SPAWN]: listedAt(2, 1),
    });
    notify(container, { sessionId, revision: 0, root: { kind: "deriving" }, staleParts: [] });
    await flush();

    expect(service.isStale).toBe(true);
    expect(countReads()).toBe(1);

    notify(container, { sessionId, revision: 1, root: { kind: "derived" }, staleParts: [LEVEL_SPAWN_PART] });
    await flush();

    expect(countReads()).toBe(2);
    expect(service.level.value?.selected.revision).toBe(1);
    expect(service.level.value?.selected.value.visuals).toBe(7);
    expect(service.isStale).toBe(false);
    expect(countListings() - listings).toBe(1);
    expect(service.spawn?.objects).toHaveLength(2);
  });

  it("keeps the level shown where a read again answers another level or an older revision", async () => {
    const { container, service, sessionId } = await openLevel();

    setMockInvokeResponses({
      ["plugin:levels|get_level"]: () => mockSessionSnapshot(mockSelectedLevelDescription(), "another", 1),
    });
    notify(container, { sessionId, revision: 1, root: { kind: "derived" }, staleParts: [] });
    await flush();

    expect(service.level.value?.selected.sessionId).toBe(sessionId);
    expect(service.level.value?.selected.revision).toBe(0);
    expect(service.isStale).toBe(true);
  });

  it("marks a level whose own files moved stale, reading nothing again", async () => {
    const { container, service, sessionId } = await openLevel();
    const before: number = countListings();

    notify(container, { sessionId, revision: 0, root: { kind: "stale" }, staleParts: [LEVEL_SPAWN_PART] });
    await flush();

    expect(service.isStale).toBe(true);
    expect(countListings()).toBe(before);
    expect(service.spawn?.objects).toHaveLength(1);
  });
});
