import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";
import { cancelFlows } from "@xrf/flow";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { EJobsCommand } from "@xrf/ipc/commands/jobs";
import { mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { ARCHIVES_JOB_PACK, ARCHIVES_JOB_UNPACK } from "@xrf/ipc/types/xrf-app-archives";
import { ArchivePackConfig, ArchivePackResult } from "@xrf/ipc/types/xrf-pack";

import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { PackerService } from "@/applications/archives-packer/services/packer";
import { mockArchivePackResult } from "@/core/archive/fixtures/archive.mocks";
import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

const CONFIG: ArchivePackConfig = { ...FALLBACK_PACK_CONFIG, source: "C:\\in", destination: "C:\\out" };
const RESULT: ArchivePackResult = mockArchivePackResult();

function mockDeferred<T>() {
  let resolve: (value: T) => void = noop;
  const promise = new Promise<T>((settle) => {
    resolve = settle;
  });

  return { promise, resolve };
}

function mockPacker() {
  const descriptor = mockInjectedService(PackerService);

  descriptor.service.config = CONFIG;

  return descriptor;
}

describe("PackerService operation", () => {
  beforeEach(resetMockInvoke);

  it("locks configuration I/O for a locally started pack", async () => {
    const response = mockDeferred<ArchivePackResult>();

    mockIpcCommands({ [EArchivesCommand.PACK_DIRECTORY]: () => response.promise });

    const { service } = mockPacker();
    const packing = service.pack(CONFIG, true);

    expect(service.isBusy).toBe(true);
    expect(service.operation.isRunning).toBe(true);

    const importing = service.importConfig("pack.ltx");
    const exporting = service.exportConfig("pack.ltx");

    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.IMPORT_PACK_CONFIG, expect.anything());
    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.EXPORT_PACK_CONFIG, expect.anything());
    expect(mockInvoke).toHaveBeenCalledWith(
      EArchivesCommand.PACK_DIRECTORY,
      expect.objectContaining({ request: { config: CONFIG, isForced: true } })
    );

    response.resolve(RESULT);
    await Promise.all([packing, importing, exporting]);

    expect(service.isBusy).toBe(false);
    expect(service.operation.result).toEqual(RESULT);
  });

  it("keeps configuration editable while a rediscovered pack blocks commands", async () => {
    const { service, container } = mockPacker();
    const jobs = container.get(JobsService);

    jobs.jobs = [
      {
        id: "adopted-pack",
        kind: ARCHIVES_JOB_PACK,
        request: null,
        progress: null,
        isAdopted: true,
        isCancelRequested: false,
      },
    ];

    expect(service.isBusy).toBe(false);
    expect(service.operation.isRunning).toBe(true);

    service.patchConfig({ name: "next-pack" });
    await service.importConfig("pack.ltx");
    await service.exportConfig("pack.ltx");
    await service.pack(CONFIG, false);

    expect(service.config?.name).toBe("next-pack");
    expect(mockInvoke).not.toHaveBeenCalled();

    service.operation.cancel();

    expect(mockInvoke).toHaveBeenCalledWith(EJobsCommand.CANCEL, { id: "adopted-pack" });
  });

  it("keeps the eventual partial result after requesting pack cancellation", async () => {
    const response = mockDeferred<ArchivePackResult>();

    mockIpcCommands({ [EArchivesCommand.PACK_DIRECTORY]: () => response.promise });

    const { service } = mockPacker();
    const packing = service.pack(CONFIG, false);
    const id = service.operation.job?.id;

    service.operation.cancel();

    expect(mockInvoke).toHaveBeenCalledWith(EJobsCommand.CANCEL, { id });
    expect(service.isBusy).toBe(true);

    const partial: ArchivePackResult = mockArchivePackResult({ outcome: "cancelled", volumes: [], volumesOpened: [] });

    response.resolve(partial);
    await packing;

    expect(service.isBusy).toBe(false);
    expect(service.operation.result).toEqual(partial);
  });

  it("unlocks an abandoned view without cancelling the backend pack or publishing its response", async () => {
    const response = mockDeferred<ArchivePackResult>();

    mockIpcCommands({ [EArchivesCommand.PACK_DIRECTORY]: () => response.promise });

    const { service, container } = mockPacker();
    const packing = service.pack(CONFIG, false);

    cancelFlows(service);
    await packing;

    expect(service.isBusy).toBe(false);
    expect(container.get(JobsService).jobs).toHaveLength(1);
    expect(mockInvoke).not.toHaveBeenCalledWith(EJobsCommand.CANCEL, expect.anything());

    response.resolve(RESULT);
    await response.promise;

    expect(service.operation.result).toBeNull();
  });

  it("keeps the old configuration and pack result until an import succeeds", async () => {
    const response = mockDeferred<ArchivePackConfig>();

    mockIpcCommands({
      [EArchivesCommand.PACK_DIRECTORY]: RESULT,
      [EArchivesCommand.IMPORT_PACK_CONFIG]: () => response.promise,
    });

    const { service } = mockPacker();

    await service.pack(CONFIG, false);

    const importing = service.importConfig("pack.ltx");

    expect(service.isBusy).toBe(true);
    expect(service.config).toEqual(CONFIG);
    expect(service.operation.result).toEqual(RESULT);

    const imported = { ...CONFIG, name: "imported" };

    response.resolve(imported);
    await importing;

    expect(service.isBusy).toBe(false);
    expect(service.config).toEqual(imported);
    expect(service.operation.result).toBeNull();
  });

  it("retains the previous pack result when importing fails", async () => {
    mockIpcCommands({
      [EArchivesCommand.PACK_DIRECTORY]: RESULT,
      [EArchivesCommand.IMPORT_PACK_CONFIG]: () => {
        throw new Error("Invalid configuration");
      },
    });

    const { service } = mockPacker();

    await service.pack(CONFIG, false);
    await service.importConfig("pack.ltx");

    expect(service.config).toEqual(CONFIG);
    expect(service.operation.result).toEqual(RESULT);
    expect(service.error).toBe("Invalid configuration");
    expect(service.isBusy).toBe(false);
  });

  it("clears a previous pack error while exporting without discarding its partial result", async () => {
    const response = mockDeferred<null>();

    mockIpcCommands({ [EArchivesCommand.EXPORT_PACK_CONFIG]: () => response.promise });

    const { service } = mockPacker();

    service.operation.adopt({
      id: "failed-pack",
      kind: ARCHIVES_JOB_PACK,
      conclusion: "failed",
      error: "Could not finish",
      result: RESULT,
    });

    const exporting = service.exportConfig("pack.ltx");

    expect(service.error).toBeNull();
    expect(service.operation.result).toEqual(RESULT);
    expect(service.isBusy).toBe(true);

    response.resolve(null);
    await exporting;

    expect(service.operation.result).toEqual(RESULT);
    expect(service.isBusy).toBe(false);
  });

  it("clears a configuration error when a retained pack outcome is accepted", async () => {
    mockIpcCommands({
      [EArchivesCommand.DEFAULT_PACK_CONFIG]: CONFIG,
      [EArchivesCommand.IMPORT_PACK_CONFIG]: () => {
        throw new Error("Invalid configuration");
      },
    });

    const { service, container } = mockPacker();

    container.provision();
    await service.onProvision();
    await service.importConfig("pack.ltx");

    expect(service.error).toBe("Invalid configuration");

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "adopted-pack",
      kind: ARCHIVES_JOB_PACK,
      conclusion: "completed",
      error: null,
      result: RESULT,
    });

    expect(service.error).toBeNull();
    expect(service.operation.result).toEqual(RESULT);
  });

  it("keeps configuration errors when another application's outcome arrives", async () => {
    mockIpcCommands({
      [EArchivesCommand.DEFAULT_PACK_CONFIG]: CONFIG,
      [EArchivesCommand.EXPORT_PACK_CONFIG]: () => {
        throw new Error("Cannot write configuration");
      },
    });

    const { service, container } = mockPacker();

    container.provision();
    await service.onProvision();
    await service.exportConfig("pack.ltx");

    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "adopted-unpack",
      kind: ARCHIVES_JOB_UNPACK,
      conclusion: "completed",
      error: null,
      result: null,
    });

    expect(service.error).toBe("Cannot write configuration");
  });
});
