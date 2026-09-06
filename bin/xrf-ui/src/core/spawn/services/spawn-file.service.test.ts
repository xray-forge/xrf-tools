import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { flowResult } from "@wirestate/mobx";

import { SpawnGraphsChunk } from "@/core/bindings/types/xrf-db";
import { SpawnFileService } from "@/core/spawn/services/spawn-file.service";
import { mockSpawnFile } from "@/fixtures/mocks/spawn.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";
import { cancelFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

function deferred<T>() {
  let resolve: (value: T) => void = noop;
  const promise = new Promise<T>((settle) => {
    resolve = settle;
  });

  return { promise, resolve };
}

describe("SpawnFileService chunk lifecycle", () => {
  beforeEach(resetMockInvoke);

  it.each([
    ["header", "loadHeader", "get_header"],
    ["alifeSpawn", "loadAlifeSpawn", "get_alife_spawns"],
    ["artefactSpawn", "loadArtefactSpawn", "get_artefact_spawns"],
    ["patrols", "loadPatrols", "get_patrols"],
    ["graphs", "loadGraphs", "get_graphs"],
  ] as const)("caches a successful empty %s chunk", async (key, load, command) => {
    const { service } = mockInjectedService(SpawnFileService);
    const read = jest.fn(() => null);

    setMockInvokeResponses({ [`plugin:spawn|${command}`]: read });

    expect(service[key].isIdle).toBe(true);

    await service[load]();
    await service[load]();

    expect(read).toHaveBeenCalledTimes(1);
    expect(service[key].isReady).toBe(true);
    expect(service[key].value).toBeNull();
  });

  it("joins a pending chunk read and permits a retry after failure", async () => {
    const { service } = mockInjectedService(SpawnFileService);
    const error = new Error("cannot read graphs");
    const failed = jest.fn(() => {
      throw error;
    });

    setMockInvokeResponses({ "plugin:spawn|get_graphs": failed });

    await service.loadGraphs();

    expect(service.graphs.isFailed).toBe(true);
    expect(service.graphs.error).toBe(error);

    const answer = deferred<Nullable<SpawnGraphsChunk>>();
    const read = jest.fn(() => answer.promise);

    setMockInvokeResponses({ "plugin:spawn|get_graphs": read });

    const first = flowResult(service.loadGraphs());
    const second = flowResult(service.loadGraphs());

    expect(service.graphs.isLoading).toBe(true);
    expect(service.graphs.error).toBeNull();
    expect(read).toHaveBeenCalledTimes(1);

    answer.resolve(null);
    await Promise.all([first, second]);

    expect(service.graphs.isReady).toBe(true);
  });

  it("invalidates an empty chunk when another file opens and when the file closes", async () => {
    const { service } = mockInjectedService(SpawnFileService);
    const file = mockSpawnFile();

    await service.loadGraphs();

    setMockInvokeResponses({
      "plugin:spawn|open_file": file.header,
      "plugin:spawn|get_graphs": file.graphs,
    });

    await service.openFile("next.spawn");

    expect(service.graphs.isIdle).toBe(true);

    await service.loadGraphs();

    expect(service.graphs.isReady).toBe(true);
    expect(service.graphs.value).toEqual(file.graphs);

    await service.closeFile();

    expect(service.header.isIdle).toBe(true);
    expect(service.graphs.isIdle).toBe(true);
    expect(service.graphs.value).toBeNull();
  });

  it("invalidates earlier empty reads when restoring the backend's open file", async () => {
    const { service } = mockInjectedService(SpawnFileService);
    const file = mockSpawnFile();

    await service.loadHeader();
    await service.loadGraphs();

    setMockInvokeResponses({
      "plugin:spawn|has_file": true,
      "plugin:spawn|get_header": file.header,
      "plugin:spawn|get_graphs": file.graphs,
      "plugin:spawn|get_path": "restored.spawn",
    });

    await service.onProvision(1);
    await service.loadGraphs();

    expect(service.header.isReady).toBe(true);
    expect(service.header.value).toEqual(file.header);
    expect(service.graphs.value).toEqual(file.graphs);
    expect(service.path).toBe("restored.spawn");
  });

  it("keeps a cancelled read retryable and ignores its late result", async () => {
    const { service } = mockInjectedService(SpawnFileService);
    const oldAnswer = deferred<Nullable<SpawnGraphsChunk>>();

    setMockInvokeResponses({ "plugin:spawn|get_graphs": () => oldAnswer.promise });

    const oldRead = flowResult(service.loadGraphs());

    cancelFlow(service, "loadGraphs");
    await oldRead;

    expect(service.graphs.isIdle).toBe(true);

    setMockInvokeResponses({ "plugin:spawn|get_graphs": null });
    await service.loadGraphs();

    oldAnswer.resolve(mockSpawnFile().graphs);
    await oldAnswer.promise;

    expect(service.graphs.isReady).toBe(true);
    expect(service.graphs.value).toBeNull();
  });

  it("abandons an old file's pending chunk before opening its replacement", async () => {
    const { service } = mockInjectedService(SpawnFileService);
    const oldAnswer = deferred<Nullable<SpawnGraphsChunk>>();
    const file = mockSpawnFile();

    setMockInvokeResponses({
      "plugin:spawn|get_graphs": () => oldAnswer.promise,
      "plugin:spawn|open_file": file.header,
    });

    const oldRead = flowResult(service.loadGraphs());

    await service.openFile("replacement.spawn");
    await oldRead;

    expect(service.graphs.isIdle).toBe(true);

    oldAnswer.resolve(file.graphs);
    await oldAnswer.promise;

    expect(service.graphs.isIdle).toBe(true);
    expect(service.graphs.value).toBeNull();
  });
});
