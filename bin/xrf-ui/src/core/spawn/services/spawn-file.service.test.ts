import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";

import { ESpawnChunkKind, SessionSnapshot, SpawnChunk, SpawnSessionDescriptor } from "@/core/ipc/types/xrf-app";
import { SpawnGraphsChunk } from "@/core/ipc/types/xrf-spawn";
import { SpawnFileService } from "@/core/spawn/services/spawn-file.service";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockSpawnFile, mockSpawnSession } from "@/fixtures/mocks/spawn.mocks";
import { InvokeHandler, mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";
import { cancelFlow } from "@/lib/mobx";

function deferred<T>() {
  let resolve: (value: T) => void = noop;
  let reject: (error: Error) => void = noop;
  const promise = new Promise<T>((settle, fail) => {
    resolve = settle;
    reject = fail;
  });

  return { promise, resolve, reject };
}

/** Answers `read_chunk` with the graphs a test hands it, tagged and under the opening that asked. */
function answerGraphs(read: () => SpawnGraphsChunk | Promise<SpawnGraphsChunk>): InvokeHandler {
  return async (args) =>
    mockSessionSnapshot<SpawnChunk>({ kind: ESpawnChunkKind.GRAPHS, ...(await read()) }, String(args?.sessionId));
}

async function mockOpenedService() {
  const { service } = mockInjectedService(SpawnFileService);
  const session = mockSpawnSession();

  setMockInvokeResponses({ "plugin:spawn|open_file": mockSessionResponse(session) });
  await service.openFile(session.path);

  return { service, session };
}

describe("SpawnFileService sessions", () => {
  beforeEach(() => {
    resetMockInvoke();
    mockInvoke.mockClear();
  });

  it("restores one coherent descriptor without eagerly reading chunks", async () => {
    const { service } = mockInjectedService(SpawnFileService);
    const session = mockSpawnSession();

    expect(service.chunks.header.value).toBeNull();
    await service.loadGraphs();
    expect(mockInvoke).not.toHaveBeenCalled();

    setMockInvokeResponses({ "plugin:spawn|restore_file": mockSessionSnapshot(session, "restored") });
    await service.onProvision(1);

    expect(mockInvoke.mock.calls).toEqual([["plugin:spawn|restore_file", { tool: "shell" }]]);
    expect(service.sessionId).toBe("restored");
    expect(service.chunks.header.value).toEqual(session.header);
    expect(service.path).toBe(session.path);
    expect(service.chunks.graphs.value).toBeNull();
    expect(service.isReady).toBe(true);
  });

  it.each([
    [ESpawnChunkKind.ALIFE_SPAWN, "loadAlifeSpawn"],
    [ESpawnChunkKind.ARTEFACT_SPAWN, "loadArtefactSpawn"],
    [ESpawnChunkKind.PATROLS, "loadPatrols"],
    [ESpawnChunkKind.GRAPHS, "loadGraphs"],
  ] as const)("caches the addressed %s chunk", async (kind, load) => {
    const { service } = await mockOpenedService();
    const chunk = { kind, ...mockSpawnFile()[kind] } as SpawnChunk;
    const read = jest.fn((args?: Record<string, unknown>) => mockSessionSnapshot(chunk, String(args?.sessionId)));

    setMockInvokeResponses({ "plugin:spawn|read_chunk": read });
    await service[load]();
    await service[load]();

    expect(read).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:spawn|read_chunk", { sessionId: service.sessionId, chunk: kind });
    expect(service.chunks[kind].value).toEqual(chunk);
  });

  it("refuses a chunk of another kind than it asked for", async () => {
    const { service } = await mockOpenedService();

    setMockInvokeResponses({
      "plugin:spawn|read_chunk": mockSessionSnapshot<SpawnChunk>({
        kind: ESpawnChunkKind.PATROLS,
        ...mockSpawnFile().patrols,
      }),
    });
    await service.loadGraphs();

    expect(service.chunks.graphs.value).toBeNull();
    expect(service.chunks.graphs.error?.message).toBe("Asked for the graphs chunk, read the patrols chunk");
  });

  it("joins a pending read and retries after failure", async () => {
    const { service } = await mockOpenedService();
    const error = new Error("cannot read graphs");

    setMockInvokeResponses({
      "plugin:spawn|read_chunk": () => {
        throw error;
      },
    });
    await service.loadGraphs();
    expect(service.chunks.graphs.error).toBe(error);

    const answer = deferred<SpawnGraphsChunk>();
    const read = jest.fn(answerGraphs(() => answer.promise));

    setMockInvokeResponses({ "plugin:spawn|read_chunk": read });

    const first = flowResult(service.loadGraphs());
    const second = flowResult(service.loadGraphs());

    expect(service.chunks.graphs.isLoading).toBe(true);
    expect(service.chunks.graphs.error).toBeNull();
    expect(read).toHaveBeenCalledTimes(1);
    answer.resolve(mockSpawnFile().graphs);
    await Promise.all([first, second]);
    expect(service.chunks.graphs.value).toEqual({ kind: ESpawnChunkKind.GRAPHS, ...mockSpawnFile().graphs });
    expect(service.chunks.graphs.isLoading).toBe(false);
  });

  it("keeps the committed file and caches after a failed replacement", async () => {
    const { service, session } = await mockOpenedService();
    const answer = deferred<SessionSnapshot<SpawnSessionDescriptor>>();
    const graphs = mockSpawnFile().graphs;

    setMockInvokeResponses({ "plugin:spawn|read_chunk": answerGraphs(() => graphs) });
    await service.loadGraphs();
    service.selectRow("graphs", 1, { name: "selection" });

    const cached = service.chunks.graphs.value;
    const selection = service.selectedRow;

    setMockInvokeResponses({
      "plugin:spawn|open_file": () => answer.promise,
      "plugin:spawn|restore_file": mockSessionSnapshot(session),
    });

    const opening = flowResult(service.openFile("broken.spawn"));

    expect(service.isOpening).toBe(true);
    expect(service.isBusy).toBe(true);
    expect(service.path).toBe(session.path);
    expect(service.chunks.graphs.value).toBe(cached);

    answer.reject(new Error("corrupt file"));
    await opening;
    expect(service.isOpening).toBe(false);
    expect(service.isOpen).toBe(true);
    expect(service.chunks.header.value).toEqual(session.header);
    expect(service.chunks.graphs.value).toBe(cached);
    expect(service.selectedRow).toBe(selection);
  });

  it("ignores an older open that finishes after its replacement", async () => {
    const { service } = await mockOpenedService();
    const older = deferred<SessionSnapshot<SpawnSessionDescriptor>>();
    const latest = mockSpawnSession({ path: "latest.spawn" });

    setMockInvokeResponses({ "plugin:spawn|open_file": () => older.promise });

    const oldOpen = flowResult(service.openFile("old.spawn"));

    setMockInvokeResponses({ "plugin:spawn|open_file": mockSessionResponse(latest) });

    await service.openFile(latest.path);

    older.resolve(mockSessionSnapshot(mockSpawnSession({ path: "old.spawn" }), "old"));

    await oldOpen;

    expect(service.path).toBe(latest.path);
    expect(service.isOpening).toBe(false);
  });

  it("retains the previous document when a replacement fails and a superseded open answers late", async () => {
    const { service, session } = await mockOpenedService();
    const older = deferred<SessionSnapshot<SpawnSessionDescriptor>>();
    const committed = mockSessionSnapshot(mockSpawnSession({ path: "committed.spawn" }), "committed");

    setMockInvokeResponses({ "plugin:spawn|open_file": () => older.promise });

    const oldOpen = flowResult(service.openFile(committed.value.path));

    setMockInvokeResponses({
      "plugin:spawn|open_file": () => {
        throw new Error("later open failed");
      },
      "plugin:spawn|restore_file": committed,
    });

    await service.openFile("broken.spawn");

    older.resolve(committed);
    await oldOpen;

    expect(service.path).toBe(session.path);
  });

  it("keeps close authoritative over a pending open", async () => {
    const { service } = await mockOpenedService();
    const answer = deferred<SessionSnapshot<SpawnSessionDescriptor>>();

    setMockInvokeResponses({ "plugin:spawn|open_file": () => answer.promise });

    const opening = flowResult(service.openFile("late.spawn"));

    await service.closeFile();
    answer.resolve(mockSessionSnapshot(mockSpawnSession()));
    await opening;
    expect(service.isOpen).toBe(false);
    expect(service.path).toBeNull();
    expect(service.chunks.header.value).toBeNull();
    expect(service.isOpening).toBe(false);
  });

  it("keeps a cancelled read retryable and ignores its late result", async () => {
    const { service } = await mockOpenedService();
    const answer = deferred<SpawnGraphsChunk>();

    setMockInvokeResponses({ "plugin:spawn|read_chunk": answerGraphs(() => answer.promise) });

    const reading = flowResult(service.loadGraphs());

    cancelFlow(service, "loadGraphs");
    await reading;
    expect(service.chunks.graphs.value).toBeNull();

    const graphs = mockSpawnFile().graphs;

    setMockInvokeResponses({ "plugin:spawn|read_chunk": answerGraphs(() => graphs) });
    await service.loadGraphs();
    answer.resolve(mockSpawnFile({ graphs: { ...graphs, levels: [] } }).graphs);
    await answer.promise;
    expect(service.chunks.graphs.value).toEqual({ kind: ESpawnChunkKind.GRAPHS, ...graphs });
  });

  it.each(["replace", "close"] as const)("discards a pending chunk after %s", async (action) => {
    const { service } = await mockOpenedService();
    const answer = deferred<SpawnGraphsChunk>();

    setMockInvokeResponses({
      "plugin:spawn|read_chunk": answerGraphs(() => answer.promise),
      "plugin:spawn|open_file": mockSessionResponse(mockSpawnSession({ path: "replacement.spawn" })),
    });

    const reading = flowResult(service.loadGraphs());

    if (action === "replace") {
      await service.openFile("replacement.spawn");
    } else {
      await service.closeFile();
    }

    answer.resolve(mockSpawnFile().graphs);
    await reading;
    expect(service.chunks.graphs.value).toBeNull();
  });
});
