import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";

import { JobDescription } from "@/core/bindings/types/xrf-app";
import { IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs/jobs.service";
import { EMIT_NOTIFICATION_EVENT, INotificationPayload } from "@/core/notifications/lib";
import {
  emitMockChannelMessage,
  resetMockChannels,
  resetMockInvoke,
  setMockInvokeResponses,
} from "@/fixtures/mocks/tauri.mocks";
import { IInjectedServiceMockDescriptor, mockInjectedService } from "@/fixtures/utils/container";

const LIST_COMMAND: string = "plugin:jobs|list";
const ATTACH_COMMAND: string = "plugin:jobs|attach";

/** How long the service waits between asking an adopted job where it has got to. */
const POLL_INTERVAL: number = 1000;

function described(patch: Partial<JobDescription> = {}): JobDescription {
  return {
    id: "b8f0",
    kind: "archives.pack",
    leaseKeys: ["archives.pack:c:\\out|gamedata"],
    isCancelRequested: false,
    progress: null,
    conclusion: null,
    error: null,
    result: null,
    duration: 1000,
    ...patch,
  } as JobDescription;
}

/**
 * A provisioned service, plus whatever it announced.
 *
 * @returns The service and the notifications it raised.
 */
function watched(): {
  service: JobsService;
  raised: Array<INotificationPayload>;
  settled: Array<IJobSettledPayload>;
} {
  const { service, container }: IInjectedServiceMockDescriptor<JobsService> = mockInjectedService(JobsService);
  const raised: Array<INotificationPayload> = [];
  const settled: Array<IJobSettledPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  container
    .get(EventBus)
    .subscribe(JOB_SETTLED_EVENT, (event: WireEvent<IJobSettledPayload>) =>
      settled.push(event.payload as IJobSettledPayload)
    );

  return { service, raised, settled };
}

describe("JobsService adoption", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("takes over a job the backend was already running", async () => {
    // The reload case. The backend never stopped: its command future lives in the application runtime, so the run kept
    // writing while this window forgot everything it knew about it.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const { service } = watched();

    await service.onProvision(1);

    expect(service.jobs).toHaveLength(1);
    expect(service.jobs[0].id).toBe("b8f0");
    expect(service.jobs[0].isAdopted).toBe(true);
  });

  it("watches an adopted job over a channel of its own", async () => {
    // The whole point of attaching: the run is still reporting to the channel of the page that reloaded, which is a
    // callback the webview can no longer find. A channel of this window's own replaces it and the bar moves again.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()], [ATTACH_COMMAND]: true });

    const { service } = watched();

    await service.onProvision(1);

    emitMockChannelMessage({
      levels: [{ id: "write", label: null, completed: 4, total: 10, unit: "items" }],
      duration: 1200,
      detail: null,
    });

    expect(service.getJob("b8f0")?.progress?.levels[0].completed).toBe(4);
  });

  it("keeps the listing from overwriting what the channel is reporting", async () => {
    // A listing lags the channel by up to one emission interval, so applying both would let a running bar go backwards
    // — which reads as work being redone.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()], [ATTACH_COMMAND]: true });

    const { service } = watched();

    await service.onProvision(1);

    emitMockChannelMessage({
      levels: [{ id: "write", label: null, completed: 9, total: 10, unit: "items" }],
      duration: 3000,
      detail: null,
    });

    setMockInvokeResponses({
      [LIST_COMMAND]: [
        described({
          progress: {
            levels: [{ id: "write", label: null, completed: 2, total: 10, unit: "items" }],
            duration: 1000,
            detail: null,
          },
        }),
      ],
      [ATTACH_COMMAND]: true,
    });

    await jest.advanceTimersByTimeAsync(POLL_INTERVAL);

    expect(service.getJob("b8f0")?.progress?.levels[0].completed).toBe(9);
  });

  it("still follows a job by asking when it cannot be attached to", async () => {
    // The job finished between the listing and the attach, or the command failed. Polling is slower, not broken.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()], [ATTACH_COMMAND]: false });

    const { service } = watched();

    await service.onProvision(1);

    setMockInvokeResponses({
      [LIST_COMMAND]: [
        described({
          progress: {
            levels: [{ id: "write", label: null, completed: 7, total: 10, unit: "items" }],
            duration: 2000,
            detail: null,
          },
        }),
      ],
      [ATTACH_COMMAND]: false,
    });

    await jest.advanceTimersByTimeAsync(POLL_INTERVAL);

    expect(service.getJob("b8f0")?.progress?.levels[0].completed).toBe(7);
  });

  it("hands the answer of a settled adopted job to whoever owns that kind", async () => {
    // Nothing here awaited this run: the command answered a page that no longer exists. The copy the backend retained
    // is the only way the tool can render what the pack produced.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()], [ATTACH_COMMAND]: true });

    const { service, settled } = watched();

    await service.onProvision(1);

    setMockInvokeResponses({
      [LIST_COMMAND]: [described({ conclusion: "completed", result: { volumes: ["textures.db0"] } })],
      [ATTACH_COMMAND]: true,
    });

    await jest.advanceTimersByTimeAsync(POLL_INTERVAL);

    expect(settled).toHaveLength(1);
    expect(settled[0].kind).toBe("archives.pack");
    expect(settled[0].conclusion).toBe("completed");
    expect(settled[0].result).toEqual({ volumes: ["textures.db0"] });
  });

  it("announces a settled adopted job that answered nothing", async () => {
    setMockInvokeResponses({ [LIST_COMMAND]: [described()], [ATTACH_COMMAND]: true });

    const { service, settled } = watched();

    await service.onProvision(1);

    setMockInvokeResponses({
      [LIST_COMMAND]: [described({ conclusion: "failed", error: "volume cap refuses particles.xr" })],
      [ATTACH_COMMAND]: true,
    });

    await jest.advanceTimersByTimeAsync(POLL_INTERVAL);

    expect(settled[0].result).toBeNull();
    expect(settled[0].error).toBe("volume cap refuses particles.xr");
  });

  it("adopts one job once when the same service is provisioned twice", async () => {
    // What React strict mode does on every mount in development: the provider deprovisions and provisions again, and
    // the container it retains hands back the same service. Two entries for one run would draw two bars over one pack.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const { service } = watched();

    await service.onProvision(1);
    service.onDeprovision(1);
    await service.onProvision(2);

    expect(service.jobs).toHaveLength(1);
  });

  it("adopts nothing when the backend has already finished everything", async () => {
    setMockInvokeResponses({ [LIST_COMMAND]: [described({ conclusion: "completed" })] });

    const { service } = watched();

    await service.onProvision(1);

    expect(service.jobs).toHaveLength(0);
  });

  it("lets a tool re-attach to an adopted job by kind", async () => {
    // What stops a tool showing an idle form over a pack that is still writing volumes, and stops the user starting a
    // second one that the lease would refuse by naming a job they cannot see.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const { service } = watched();

    await service.onProvision(1);

    expect(service.getJobOfKind("archives.pack")?.id).toBe("b8f0");
  });

  it("follows an adopted job by asking, because its channel died with the page", async () => {
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const { service } = watched();

    await service.onProvision(1);

    setMockInvokeResponses({
      [LIST_COMMAND]: [
        described({
          progress: {
            levels: [{ id: "write", label: null, completed: 7, total: 10, unit: "items" }],
            duration: 2000,
            detail: null,
          },
        }),
      ],
    });

    await jest.advanceTimersByTimeAsync(POLL_INTERVAL);

    expect(service.getJob("b8f0")?.progress?.levels[0].completed).toBe(7);
  });

  it("announces an adopted job that ended, and forgets it", async () => {
    // Nobody is awaiting its answer, so the polled conclusion is the only way the user learns their pack finished.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const { service, raised } = watched();

    await service.onProvision(1);

    expect(raised).toHaveLength(0);

    setMockInvokeResponses({ [LIST_COMMAND]: [described({ conclusion: "completed" })] });

    await jest.advanceTimersByTimeAsync(POLL_INTERVAL);

    expect(service.jobs).toHaveLength(0);
    expect(raised).toHaveLength(1);
    expect(raised[0].title).toContain("reloading");
  });

  it("stops asking once nothing is adopted", async () => {
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const { service } = watched();

    await service.onProvision(1);

    setMockInvokeResponses({ [LIST_COMMAND]: [described({ conclusion: "completed" })] });

    await jest.advanceTimersByTimeAsync(POLL_INTERVAL);

    expect(jest.getTimerCount()).toBe(0);
  });

  it("treats a job that left the listing as ended with an unrecorded outcome", async () => {
    // Twenty finished jobs push it out of the retained ring. It ended; how is no longer knowable.
    setMockInvokeResponses({ [LIST_COMMAND]: [described()] });

    const { service, raised } = watched();

    await service.onProvision(1);

    setMockInvokeResponses({ [LIST_COMMAND]: [] });

    await jest.advanceTimersByTimeAsync(POLL_INTERVAL);

    expect(service.jobs).toHaveLength(0);
    expect(raised[0].details).toContain("no longer recorded");
  });

  it("keeps working when the listing cannot be read", async () => {
    // A start-up is a bad moment to fail loudly, and the lease still prevents a duplicate from doing any damage.
    setMockInvokeResponses({
      [LIST_COMMAND]: () => {
        throw new Error("state unavailable");
      },
    });

    const { service } = watched();

    await expect(service.onProvision(1)).resolves.toBeUndefined();
    expect(service.jobs).toHaveLength(0);
  });
});
