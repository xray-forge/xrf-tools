import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container, EventBus } from "@wirestate/core";
import { flowResult } from "@wirestate/mobx";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { IpcCommandAnswer, mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { SessionChange, SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import {
  LEVEL_ENVIRONMENT_PART,
  LEVEL_GEOMETRY_PART,
  LEVEL_MAP_PART,
  LEVEL_SPAWN_PART,
  LevelSpawnObjectsDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { createRoots } from "@/core/assets/lib";
import {
  mockLevelPartResponses,
  mockLevelSpawnObject,
  mockSelectedLevelDescription,
} from "@/core/level/fixtures/level.mocks";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";

import { LevelLoadService } from "./level-load.service";

const LIST_SPAWN: string = ELevelsCommand.DESCRIBE_SPAWN_OBJECTS;

function listing(objects: number): LevelSpawnObjectsDescription {
  return {
    objects: Array.from({ length: objects }, (_, index: number) => mockLevelSpawnObject({ index, visual: 0 })),
    visuals: ["dynamics\\box"],
  };
}

function countListings(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === LIST_SPAWN).length;
}

/** Reads of the committed level, the restore at provision included. */
function countReads(): number {
  return mockInvoke.mock.calls.filter(([name]) => name === ELevelsCommand.RESTORE_LEVEL).length;
}

/** A spawn listing at `revision`, for the level the call names. */
function listedAt(objects: number, revision: number): IpcCommandAnswer<ELevelsCommand.DESCRIBE_SPAWN_OBJECTS> {
  return (args) => mockSessionSnapshot(listing(objects), args.sessionId, revision);
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

  mockIpcCommands({
    [ELevelsCommand.RESTORE_LEVEL]: null,
    [ELevelsCommand.OPEN_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
    ...mockLevelPartResponses(),
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

interface IHeldOpen {
  container: Container;
  service: LevelLoadService;
  /** The id the open asked for. */
  getSessionId: () => string;
  /** Answers the open, then waits for its start to be asked for. */
  answerOpen: () => Promise<void>;
  /** Answers the start, then waits for the open to settle. */
  answerStart: () => Promise<void>;
  /**
   * Answers a read of the level again at \`revision\`, with \`visuals\`; asked once the start is, since it replaces every
   * answer the open is still to ask for but the start's own.
   */
  readsAgainAt: (revision: number, visuals: number) => void;
}

/** Provisions the loader and opens a level whose answers wait to be given, the open's and then its start's. */
async function openHeld(): Promise<IHeldOpen> {
  const container: Container = mockContainer([LevelLoadService]);
  const service: LevelLoadService = container.get(LevelLoadService);
  const opens: Array<() => void> = [];
  const starts: Array<() => void> = [];
  let sessionId: string = "";

  mockIpcCommands({
    [ELevelsCommand.RESTORE_LEVEL]: null,
    ...mockLevelPartResponses(),
    [LIST_SPAWN]: listedAt(1, 0),
    [ELevelsCommand.OPEN_LEVEL]: mockSessionResponse((args?: Record<string, unknown>) => {
      const request = args?.request as Record<string, unknown> | undefined;

      sessionId = (args?.sessionId ?? request?.sessionId) as string;

      return new Promise((resolve) => opens.push(() => resolve(mockSelectedLevelDescription())));
    }),
    [ELevelsCommand.DESCRIBE_START]: () =>
      new Promise((resolve) => starts.push(() => resolve(mockSessionSnapshot(null, sessionId, 0)))),
  });

  container.provision();
  await flush();

  const loading: Promise<unknown> = flowResult(
    service.load({
      engine: EXrayEngineChoice.VANILLA,
      isDltx: false,
      roots: createRoots(["C:\\game"]),
      source: { kind: "asset", logicalPath: "levels\\zaton" },
    })
  );

  await waitFor(() => expect(opens).toHaveLength(1));

  return {
    answerOpen: async () => {
      opens[0]();
      await waitFor(() => expect(starts).toHaveLength(1));
    },
    answerStart: async () => {
      starts[0]();
      await loading;
      await flush();
    },
    container,
    getSessionId: () => sessionId,
    readsAgainAt: (revision: number, visuals: number) =>
      mockIpcCommands({
        ...mockLevelPartResponses(),
        [LIST_SPAWN]: listedAt(1, 0),
        [ELevelsCommand.RESTORE_LEVEL]: () =>
          mockSessionSnapshot(mockSelectedLevelDescription({ visuals }), sessionId, revision),
      }),
    service,
  };
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

    mockIpcCommands({ [LIST_SPAWN]: listedAt(3, 1) });
    notify(container, {
      sessionId,
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: false,
    });
    await flush();

    expect(countListings() - before).toBe(1);
    expect(service.spawn?.objects).toHaveLength(3);
    expect(service.spawnRevision).toBe(1);
    expect(service.isStale).toBe(false);
  });

  it("lists nothing again for another level's change, or for parts the panels do not show", async () => {
    const { container, service, sessionId } = await openLevel();
    const before: number = countListings();

    notify(container, {
      sessionId: "another",
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: false,
    });
    notify(container, {
      sessionId,
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_ENVIRONMENT_PART],
      isViewRenewed: false,
    });
    await flush();

    expect(countListings()).toBe(before);
    expect(service.spawn?.objects).toHaveLength(1);
  });

  it("drops a listing read at an older revision than the one shown", async () => {
    const { container, service, sessionId } = await openLevel();
    const answers: Array<(snapshot: SessionSnapshot<LevelSpawnObjectsDescription>) => void> = [];

    mockIpcCommands({ [LIST_SPAWN]: () => new Promise((resolve) => answers.push(resolve)) });

    notify(container, {
      sessionId,
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: false,
    });
    notify(container, {
      sessionId,
      revision: 2,
      root: { kind: "kept" },
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: false,
    });
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

    mockIpcCommands({
      [ELevelsCommand.RESTORE_LEVEL]: () =>
        mockSessionSnapshot(mockSelectedLevelDescription({ visuals: 7 }), sessionId, 1),
      [LIST_SPAWN]: listedAt(2, 1),
    });
    notify(container, { sessionId, revision: 0, root: { kind: "deriving" }, staleParts: [], isViewRenewed: false });
    await flush();

    expect(service.isStale).toBe(true);
    expect(countReads()).toBe(1);

    notify(container, {
      sessionId,
      revision: 1,
      root: { kind: "derived" },
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: true,
    });
    await flush();

    expect(countReads()).toBe(2);
    expect(service.level.value?.selected.revision).toBe(1);
    expect(service.level.value?.selected.value.visuals).toBe(7);
    expect(service.isStale).toBe(false);
    expect(countListings() - listings).toBe(1);
    expect(service.spawn?.objects).toHaveLength(2);
  });

  it("reads a part named stale alone, the level itself kept, and moves the scene for what its sectors are drawn from", async () => {
    const { container, service, sessionId } = await openLevel();
    const reads: number = countReads();
    const opened = service.scene;

    expect(opened).toEqual({ revision: 0, sessionId });

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_LEVEL_MAP]: (args?: Record<string, unknown>) =>
        mockSessionSnapshot(
          { area: { maxX: 1, maxZ: 1, minX: 0, minZ: 0 }, texture: null },
          args?.sessionId as string,
          1
        ),
    });
    notify(container, {
      sessionId,
      revision: 1,
      root: { kind: "kept" },
      staleParts: [LEVEL_MAP_PART],
      isViewRenewed: false,
    });
    await flush();

    expect(service.levelMap.value?.area.maxX).toBe(1);
    expect(countReads()).toBe(reads);
    expect(service.scene).toBe(opened);

    notify(container, {
      sessionId,
      revision: 2,
      root: { kind: "kept" },
      staleParts: [LEVEL_GEOMETRY_PART],
      isViewRenewed: true,
    });
    await flush();

    expect(service.scene).toEqual({ revision: 2, sessionId });
    expect(countReads()).toBe(reads);

    // A notice of a change the scene streams as already moves it nowhere.
    const moved = service.scene;

    notify(container, {
      sessionId,
      revision: 2,
      root: { kind: "kept" },
      staleParts: [LEVEL_GEOMETRY_PART],
      isViewRenewed: true,
    });
    await flush();

    expect(service.scene).toBe(moved);
  });

  // A change heard while a level is opened is not lost to the wait: the level derived anew meanwhile is read again once
  // it is adopted, and its scene moves on.
  it("hears a change to a level opening while its start is read, once the level is adopted", async () => {
    const held: IHeldOpen = await openHeld();

    await held.answerOpen();
    held.readsAgainAt(1, 7);
    notify(held.container, {
      sessionId: held.getSessionId(),
      revision: 1,
      root: { kind: "derived" },
      staleParts: [],
      isViewRenewed: true,
    });

    expect(held.service.level.value).toBeNull();

    await held.answerStart();

    expect(held.service.level.value?.selected.revision).toBe(1);
    expect(held.service.level.value?.selected.value.visuals).toBe(7);
    expect(held.service.scene).toEqual({ revision: 1, sessionId: held.getSessionId() });
    expect(held.service.isStale).toBe(false);
  });

  // A change that cannot move the level comes at the revision it was opened at, and still marks it stale once adopted.
  it("marks a level stale for a change heard while it was opened, at the revision it was opened at", async () => {
    for (const root of [{ kind: "deriving" }, { kind: "failed", reason: "cannot read" }] as const) {
      const held: IHeldOpen = await openHeld();

      await held.answerOpen();
      notify(held.container, {
        sessionId: held.getSessionId(),
        revision: 0,
        root,
        staleParts: [],
        isViewRenewed: false,
      });
      await held.answerStart();

      expect(held.service.level.value?.selected.revision).toBe(0);
      expect(held.service.isStale).toBe(true);
    }
  });

  // The change event can come before the open's own answer resumes its flow.
  it("hears a change heard before the open answered, once the level is adopted", async () => {
    const held: IHeldOpen = await openHeld();

    notify(held.container, {
      sessionId: held.getSessionId(),
      revision: 1,
      root: { kind: "derived" },
      staleParts: [],
      isViewRenewed: true,
    });

    await held.answerOpen();
    held.readsAgainAt(1, 0);
    await held.answerStart();

    expect(held.service.level.value?.selected.revision).toBe(1);
    expect(held.service.scene).toEqual({ revision: 1, sessionId: held.getSessionId() });
  });

  // The next level is shown once its start is read, its other parts after it: nothing shows one level's parts beside
  // another's.
  it("lets the parts of the level held go as the next one shows, before its own are read", async () => {
    const { service, sessionId } = await openLevel();
    let answer!: () => void;

    expect(service.spawn?.objects).toHaveLength(1);

    mockIpcCommands({
      ...mockLevelPartResponses(),
      [ELevelsCommand.OPEN_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
      [LIST_SPAWN]: (args?: Record<string, unknown>) =>
        new Promise((resolve) => (answer = () => resolve(mockSessionSnapshot(listing(4), args?.sessionId as string)))),
    });

    const loading: Promise<unknown> = flowResult(
      service.load({
        engine: EXrayEngineChoice.VANILLA,
        isDltx: false,
        roots: createRoots(["C:\\game"]),
        source: { kind: "asset", logicalPath: "levels\\jupiter" },
      })
    );

    await waitFor(() => expect(service.level.value?.selected.sessionId).not.toBe(sessionId));
    expect(service.spawn).toBeNull();

    answer();
    await loading;

    expect(service.spawn?.objects).toHaveLength(4);
  });

  it("closes the level it held once another opens in its place", async () => {
    const { service, sessionId } = await openLevel();

    await service.load({
      engine: EXrayEngineChoice.VANILLA,
      isDltx: false,
      roots: createRoots(["C:\\game"]),
      source: { kind: "asset", logicalPath: "levels\\jupiter" },
    });
    await flush();

    expect(service.level.value?.selected.sessionId).not.toBe(sessionId);
    expect(
      mockInvoke.mock.calls.filter(([name]) => name === ELevelsCommand.CLOSE_LEVEL).map(([, args]) => args)
    ).toEqual([{ sessionIds: [sessionId] }]);
  });

  it("keeps the level shown where a read again answers another level or an older revision", async () => {
    const { container, service, sessionId } = await openLevel();

    mockIpcCommands({
      [ELevelsCommand.RESTORE_LEVEL]: () => mockSessionSnapshot(mockSelectedLevelDescription(), "another", 1),
    });
    notify(container, { sessionId, revision: 1, root: { kind: "derived" }, staleParts: [], isViewRenewed: true });
    await flush();

    expect(service.level.value?.selected.sessionId).toBe(sessionId);
    expect(service.level.value?.selected.revision).toBe(0);
    expect(service.isStale).toBe(true);
  });

  it("marks a level whose own files moved stale, reading nothing again", async () => {
    const { container, service, sessionId } = await openLevel();
    const before: number = countListings();

    notify(container, {
      sessionId,
      revision: 0,
      root: { kind: "stale" },
      staleParts: [LEVEL_SPAWN_PART],
      isViewRenewed: false,
    });
    await flush();

    expect(service.isStale).toBe(true);
    expect(countListings()).toBe(before);
    expect(service.spawn?.objects).toHaveLength(1);
  });
});
