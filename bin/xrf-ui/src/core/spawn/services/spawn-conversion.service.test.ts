import { beforeEach, describe, expect, it } from "@jest/globals";
import { EventBus } from "@wirestate/core";

import { EJobKind, SpawnConversionResult } from "@/core/ipc/types/xrf-app";
import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity } from "@/core/notifications/lib";
import { SpawnConversionService } from "@/core/spawn/services/spawn-conversion.service";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";

const RESULT: SpawnConversionResult = { operation: "pack", destination: "C:\\out\\all.spawn", outcome: "completed" };

function deferred() {
  let resolve: (result: SpawnConversionResult) => void = noop;
  const promise = new Promise<SpawnConversionResult>((settle) => {
    resolve = settle;
  });

  return { promise, resolve };
}

describe("SpawnConversionService", () => {
  beforeEach(() => setMockInvokeResponses({}));

  it.each(["pack", "unpack"] as const)("runs %s with a job identity and progress channel", async (operation) => {
    const result: SpawnConversionResult = { ...RESULT, operation };

    setMockInvokeResponses({ [`plugin:spawn|${operation}_file`]: result });

    const { service, container } = mockInjectedService(SpawnConversionService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    await service[operation]("C:\\source", RESULT.destination);

    expect(mockInvoke).toHaveBeenCalledWith(`plugin:spawn|${operation}_file`, {
      request: { source: "C:\\source", destination: RESULT.destination },
      jobId: expect.any(String),
      progress: expect.anything(),
    });
    expect(service.operation.result).toEqual(result);
    expect(service.operation.isRunning).toBe(false);
    expect(notices).toHaveLength(1);
  });

  it("ignores repeated and opposite actions while the shared flow is running", async () => {
    const response = deferred();

    setMockInvokeResponses({ "plugin:spawn|pack_file": () => response.promise });

    const { service } = mockInjectedService(SpawnConversionService);
    const first = service.pack("source", "output");
    const repeated = service.pack("other", "other-output");
    const opposite = service.unpack("output", "chunks");

    expect(service.operation.isRunning).toBe(true);
    expect(mockInvoke.mock.calls.filter(([command]) => command.startsWith("plugin:spawn|"))).toHaveLength(1);

    response.resolve(RESULT);
    await Promise.all([first, repeated, opposite]);

    expect(service.operation.result).toEqual(RESULT);
  });

  it("also refuses a conversion in a new tool scope while the root jobs service tracks one", async () => {
    const response = deferred();

    setMockInvokeResponses({ "plugin:spawn|pack_file": () => response.promise });

    const { service, container } = mockInjectedService(SpawnConversionService);
    const nextTool = new SpawnConversionService(container.get(JobsService));
    const running = service.pack("source", "output");

    await nextTool.unpack("output", "chunks");

    expect(nextTool.operation.isRunning).toBe(true);
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:spawn|unpack_file", expect.anything());

    response.resolve(RESULT);
    await running;
  });

  it("keeps a cancelled job exclusive until the backend settles and reports completed writes as success", async () => {
    const response = deferred();

    setMockInvokeResponses({ "plugin:spawn|pack_file": () => response.promise });

    const { service, container } = mockInjectedService(SpawnConversionService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    const running = service.pack("source", "output");
    const id = service.operation.job?.id;

    service.operation.cancel();

    expect(service.operation.isRunning).toBe(true);
    expect(mockInvoke).toHaveBeenCalledWith("plugin:jobs|cancel", { id });

    response.resolve(RESULT);
    await running;

    expect(notices).toEqual([expect.objectContaining({ severity: ENotificationSeverity.SUCCESS })]);
  });

  it("adopts running work after reload and publishes its retained result", async () => {
    const { service, container } = mockInjectedService(SpawnConversionService);

    await container.provision();

    const jobs = container.get(JobsService);

    jobs.jobs = [
      {
        id: "adopted",
        kind: EJobKind.SPAWN_UNPACK,
        progress: null,
        request: null,
        isCancelRequested: false,
        isAdopted: true,
      },
    ];

    await service.pack("source", "output");

    expect(service.operation.isRunning).toBe(true);
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:spawn|pack_file", expect.anything());

    jobs.jobs = [];
    container.get(EventBus).emit<IJobSettledPayload>(JOB_SETTLED_EVENT, {
      id: "adopted",
      kind: EJobKind.SPAWN_UNPACK,
      conclusion: "completed",
      error: null,
      result: { ...RESULT, operation: "unpack" },
    });

    expect(service.operation.result?.operation).toBe("unpack");

    container.deprovision();
    container.unbindAll();
  });

  it("normalizes backend failures and permits a later retry", async () => {
    setMockInvokeResponses({ "plugin:spawn|pack_file": () => Promise.reject("Spawn output is busy") });

    const { service } = mockInjectedService(SpawnConversionService);

    await service.pack("source", "output");

    expect(service.operation.error).toBe("Spawn output is busy");
    expect(service.operation.isRunning).toBe(false);

    setMockInvokeResponses({ "plugin:spawn|pack_file": RESULT });
    await service.pack("source", "output");

    expect(service.operation.error).toBeNull();
    expect(service.operation.result).toEqual(RESULT);
  });

  it("reports accepted cancellation without claiming output was written", async () => {
    setMockInvokeResponses({ "plugin:spawn|pack_file": { ...RESULT, outcome: "cancelled" } });

    const { service, container } = mockInjectedService(SpawnConversionService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));
    await service.pack("source", "output");

    expect(service.operation.result?.outcome).toBe("cancelled");
    expect(notices).toEqual([
      expect.objectContaining({
        severity: ENotificationSeverity.INFO,
        details: "Stopped before writing. Output was left unchanged.",
      }),
    ]);
  });

  it("stops publishing after deactivation while the job continues and notifies once", async () => {
    const response = deferred();

    setMockInvokeResponses({ "plugin:spawn|pack_file": () => response.promise });

    const { service, container } = mockInjectedService(SpawnConversionService);
    const jobs = container.get(JobsService);
    const notices: Array<unknown> = [];

    container.get(EventBus).subscribe(EMIT_NOTIFICATION_EVENT, (event) => notices.push(event.payload));

    const running = service.pack("source", "output");

    container.unbind(SpawnConversionService);

    expect(jobs.jobs).toHaveLength(1);

    response.resolve(RESULT);
    await running;
    await response.promise;

    expect(service.operation.result).toBeNull();
    expect(jobs.jobs).toHaveLength(0);
    expect(notices).toHaveLength(1);
  });
});
