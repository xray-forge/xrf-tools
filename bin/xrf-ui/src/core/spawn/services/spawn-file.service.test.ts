import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";

import { SpawnSessionDescriptor } from "@/core/bindings/types/xrf-app";
import { SpawnGraphsChunk } from "@/core/bindings/types/xrf-db";
import { SpawnFileService } from "@/core/spawn/services/spawn-file.service";
import { mockSpawnFile, mockSpawnSession } from "@/fixtures/mocks/spawn.mocks";
import { mockInvoke, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
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

async function mockOpenedService() {
  const { service } = mockInjectedService(SpawnFileService);
  const session = mockSpawnSession();

  setMockInvokeResponses({ "plugin:spawn|open_file": session });
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

    expect(service.chunks.header.isIdle).toBe(true);
    await service.loadGraphs();
    expect(mockInvoke).not.toHaveBeenCalled();

    setMockInvokeResponses({ "plugin:spawn|get_session": session });
    await service.onProvision(1);

    expect(mockInvoke.mock.calls).toEqual([["plugin:spawn|get_session"]]);
    expect(service.chunks.header.value).toEqual(session.header);
    expect(service.path).toBe(session.path);
    expect(service.chunks.graphs.isIdle).toBe(true);
    expect(service.isReady).toBe(true);
  });

  it.each([
    ["alifeSpawn", "loadAlifeSpawn", "get_alife_spawns"],
    ["artefactSpawn", "loadArtefactSpawn", "get_artefact_spawns"],
    ["patrols", "loadPatrols", "get_patrols"],
    ["graphs", "loadGraphs", "get_graphs"],
  ] as const)("caches the addressed %s chunk", async (key, load, command) => {
    const { service, session } = await mockOpenedService();
    const chunk = mockSpawnFile()[key];
    const read = jest.fn(() => chunk);

    setMockInvokeResponses({ ["plugin:spawn|" + command]: read });
    await service[load]();
    await service[load]();

    expect(read).toHaveBeenCalledTimes(1);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:spawn|" + command, { sessionId: session.id });
    expect(service.chunks[key].isReady).toBe(true);
    expect(service.chunks[key].value).toEqual(chunk);
  });

  it("joins a pending read and retries after failure", async () => {
    const { service } = await mockOpenedService();
    const error = new Error("cannot read graphs");

    setMockInvokeResponses({
      "plugin:spawn|get_graphs": () => {
        throw error;
      },
    });
    await service.loadGraphs();
    expect(service.chunks.graphs.error).toBe(error);

    const answer = deferred<SpawnGraphsChunk>();
    const read = jest.fn(() => answer.promise);

    setMockInvokeResponses({ "plugin:spawn|get_graphs": read });

    const first = flowResult(service.loadGraphs());
    const second = flowResult(service.loadGraphs());

    expect(service.chunks.graphs.isLoading).toBe(true);
    expect(service.chunks.graphs.error).toBeNull();
    expect(read).toHaveBeenCalledTimes(1);
    answer.resolve(mockSpawnFile().graphs);
    await Promise.all([first, second]);
    expect(service.chunks.graphs.isReady).toBe(true);
  });

  it("keeps the committed file and caches after a failed replacement", async () => {
    const { service, session } = await mockOpenedService();
    const answer = deferred<SpawnSessionDescriptor>();
    const graphs = mockSpawnFile().graphs;

    setMockInvokeResponses({ "plugin:spawn|get_graphs": graphs });
    await service.loadGraphs();
    service.selectRow("graphs", 1, { name: "selection" });

    const cached = service.chunks.graphs;
    const selection = service.selectedRow;

    setMockInvokeResponses({
      "plugin:spawn|open_file": () => answer.promise,
      "plugin:spawn|get_session": session,
    });

    const opening = flowResult(service.openFile("broken.spawn"));

    expect(service.isOpening).toBe(true);
    expect(service.isBusy).toBe(true);
    expect(service.path).toBe(session.path);
    expect(service.chunks.graphs).toBe(cached);

    answer.reject(new Error("corrupt file"));
    await opening;
    expect(service.isOpening).toBe(false);
    expect(service.isOpen).toBe(true);
    expect(service.chunks.header.value).toEqual(session.header);
    expect(service.chunks.graphs).toBe(cached);
    expect(service.selectedRow).toBe(selection);
  });

  it("ignores an older open that finishes after its replacement", async () => {
    const { service } = await mockOpenedService();
    const older = deferred<SpawnSessionDescriptor>();
    const latest = mockSpawnSession({ id: "latest", path: "latest.spawn" });

    setMockInvokeResponses({ "plugin:spawn|open_file": () => older.promise });

    const oldOpen = flowResult(service.openFile("old.spawn"));

    setMockInvokeResponses({ "plugin:spawn|open_file": latest });
    await service.openFile(latest.path);
    older.resolve(mockSpawnSession({ id: "old", path: "old.spawn" }));
    await oldOpen;
    expect(service.path).toBe(latest.path);
    expect(service.isOpening).toBe(false);
  });

  it("reconciles an unobserved commit when a later open fails", async () => {
    const { service } = await mockOpenedService();
    const older = deferred<SpawnSessionDescriptor>();
    const committed = mockSpawnSession({ id: "committed", path: "committed.spawn" });

    setMockInvokeResponses({ "plugin:spawn|open_file": () => older.promise });

    const oldOpen = flowResult(service.openFile(committed.path));

    setMockInvokeResponses({
      "plugin:spawn|open_file": () => {
        throw new Error("later open failed");
      },
      "plugin:spawn|get_session": committed,
    });
    await service.openFile("broken.spawn");
    older.resolve(committed);
    await oldOpen;
    expect(service.path).toBe(committed.path);
  });

  it("keeps close authoritative over a pending open", async () => {
    const { service } = await mockOpenedService();
    const answer = deferred<SpawnSessionDescriptor>();

    setMockInvokeResponses({ "plugin:spawn|open_file": () => answer.promise });

    const opening = flowResult(service.openFile("late.spawn"));

    await service.closeFile();
    answer.resolve(mockSpawnSession());
    await opening;
    expect(service.isOpen).toBe(false);
    expect(service.path).toBeNull();
    expect(service.chunks.header.isIdle).toBe(true);
    expect(service.isOpening).toBe(false);
  });

  it("keeps a cancelled read retryable and ignores its late result", async () => {
    const { service } = await mockOpenedService();
    const answer = deferred<SpawnGraphsChunk>();

    setMockInvokeResponses({ "plugin:spawn|get_graphs": () => answer.promise });

    const reading = flowResult(service.loadGraphs());

    cancelFlow(service, "loadGraphs");
    await reading;
    expect(service.chunks.graphs.isIdle).toBe(true);

    const graphs = mockSpawnFile().graphs;

    setMockInvokeResponses({ "plugin:spawn|get_graphs": graphs });
    await service.loadGraphs();
    answer.resolve(mockSpawnFile({ graphs: { ...graphs, levels: [] } }).graphs);
    await answer.promise;
    expect(service.chunks.graphs.value).toEqual(graphs);
  });

  it.each(["replace", "close"] as const)("discards a pending chunk after %s", async (action) => {
    const { service } = await mockOpenedService();
    const answer = deferred<SpawnGraphsChunk>();

    setMockInvokeResponses({
      "plugin:spawn|get_graphs": () => answer.promise,
      "plugin:spawn|open_file": mockSpawnSession({ id: "replacement" }),
    });

    const reading = flowResult(service.loadGraphs());

    if (action === "replace") {
      await service.openFile("replacement.spawn");
    } else {
      await service.closeFile();
    }

    answer.resolve(mockSpawnFile().graphs);
    await reading;
    expect(service.chunks.graphs.isIdle).toBe(true);
    expect(service.chunks.graphs.value).toBeNull();
  });
});
