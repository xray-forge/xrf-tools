import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { emitMockChannelMessage, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { ARCHIVES_JOB_PACK } from "@xrf/ipc/types/xrf-app-archives";

import { EApplicationId } from "@/core/routing/application";
import { mockJobDescription } from "@/fixtures/mocks/job.mocks";
import {
  JOBS_ATTACH_COMMAND,
  JOBS_LIST_COMMAND,
  JOBS_POLL_INTERVAL,
  JOBS_SLOW_LISTING,
  mockWatchedJobsService,
} from "@/fixtures/utils/jobs-service";

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
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()] });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);

    expect(service.jobs).toHaveLength(1);
    expect(service.jobs[0].id).toBe("b8f0");
    expect(service.jobs[0].isAdopted).toBe(true);
  });

  it("watches an adopted job over a channel of its own", async () => {
    // The whole point of attaching: the run is still reporting to the channel of the page that reloaded, which is a
    // callback the webview can no longer find. A channel of this window's own replaces it and the bar moves again.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()], [JOBS_ATTACH_COMMAND]: true });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);

    emitMockChannelMessage({
      levels: [{ id: "write", completed: 4, total: 10, unit: "items" }],
      duration: 1200,
      detail: null,
    });

    expect(service.getJob("b8f0")?.progress?.levels[0].completed).toBe(4);
  });

  it("keeps the listing from overwriting what the channel is reporting", async () => {
    // A listing lags the channel by up to one emission interval, so applying both would let a running bar go backwards
    // — which reads as work being redone.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()], [JOBS_ATTACH_COMMAND]: true });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);

    emitMockChannelMessage({
      levels: [{ id: "write", completed: 9, total: 10, unit: "items" }],
      duration: 3000,
      detail: null,
    });

    mockIpcCommands({
      [JOBS_LIST_COMMAND]: [
        mockJobDescription({
          progress: {
            levels: [{ id: "write", completed: 2, total: 10, unit: "items" }],
            duration: 1000,
            detail: null,
          },
        }),
      ],
      [JOBS_ATTACH_COMMAND]: true,
    });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(service.getJob("b8f0")?.progress?.levels[0].completed).toBe(9);
  });

  it("still follows a job by asking when it cannot be attached to", async () => {
    // The job finished between the listing and the attach, or the command failed. Polling is slower, not broken.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()], [JOBS_ATTACH_COMMAND]: false });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);

    mockIpcCommands({
      [JOBS_LIST_COMMAND]: [
        mockJobDescription({
          progress: {
            levels: [{ id: "write", completed: 7, total: 10, unit: "items" }],
            duration: 2000,
            detail: null,
          },
        }),
      ],
      [JOBS_ATTACH_COMMAND]: false,
    });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(service.getJob("b8f0")?.progress?.levels[0].completed).toBe(7);
  });

  it("drops an adoption the window that asked for it no longer wants", async () => {
    // Deprovisioning ends the provision that sent the listing. Letting the answer land would take jobs over on behalf
    // of a window that has gone, and start a timer nothing is left to stop.
    mockIpcCommands({
      [JOBS_LIST_COMMAND]: () =>
        new Promise((resolve) => setTimeout(() => resolve([mockJobDescription()]), JOBS_SLOW_LISTING)),
      [JOBS_ATTACH_COMMAND]: true,
    });

    const { service } = mockWatchedJobsService();

    void service.onProvision(1);

    service.onDeprovision(1);

    await jest.advanceTimersByTimeAsync(JOBS_SLOW_LISTING);

    expect(service.jobs).toHaveLength(0);
    expect(jest.getTimerCount()).toBe(0);
  });

  it("lets the newest provision own the adoption when an older one is still asking", async () => {
    // Two listings in the air at once is what a reload during start-up produces, and what strict mode produces on
    // every mount. The older answer must not adopt behind the newer one.
    mockIpcCommands({
      [JOBS_LIST_COMMAND]: () =>
        new Promise((resolve) => setTimeout(() => resolve([mockJobDescription()]), JOBS_SLOW_LISTING)),
      [JOBS_ATTACH_COMMAND]: true,
    });

    const { service } = mockWatchedJobsService();

    void service.onProvision(1);
    void service.onProvision(2);

    await jest.advanceTimersByTimeAsync(JOBS_SLOW_LISTING);

    expect(service.jobs).toHaveLength(1);
    expect(service.getJob("b8f0")?.isAdopted).toBe(true);
  });

  it("adopts one job once when the same service is provisioned twice", async () => {
    // What React strict mode does on every mount in development: the provider deprovisions and provisions again, and
    // the container it retains hands back the same service. Two entries for one run would draw two bars over one pack.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()] });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);
    service.onDeprovision(1);
    await service.onProvision(2);

    expect(service.jobs).toHaveLength(1);
  });

  it("adopts nothing when the backend has already finished everything", async () => {
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription({ conclusion: "completed" })] });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);

    expect(service.jobs).toHaveLength(0);
  });

  it("carries what an adopted job was asked to do", async () => {
    // The arguments live nowhere else once the page that sent them is gone, and a bar with no subject cannot say which
    // of two packs the user is looking at.
    mockIpcCommands({
      [JOBS_LIST_COMMAND]: [mockJobDescription({ request: { source: "c:\\gamedata", destination: "c:\\out" } })],
      [JOBS_ATTACH_COMMAND]: true,
    });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);

    expect(service.getJob("b8f0")?.request).toEqual({ source: "c:\\gamedata", destination: "c:\\out" });
  });

  it("lets a tool re-attach to an adopted job by kind", async () => {
    // What stops a tool showing an idle form over a pack that is still writing volumes, and stops the user starting a
    // second one that the lease would refuse by naming a job they cannot see.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()] });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);

    expect(service.getJobOfKind(ARCHIVES_JOB_PACK)?.id).toBe("b8f0");
  });

  it("attributes an adopted job to the tool that owns its kind", async () => {
    // The notification panel resolves a record's source through the application catalog. Attributing it to the kind
    // string instead left the panel with nothing to look up, and the user with a row labelled `archives.pack`.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()], [JOBS_ATTACH_COMMAND]: true });

    const { service, raised } = mockWatchedJobsService();

    await service.onProvision(1);

    mockIpcCommands({
      [JOBS_LIST_COMMAND]: [mockJobDescription({ conclusion: "completed" })],
      [JOBS_ATTACH_COMMAND]: true,
    });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(raised[0].source).toBe(EApplicationId.ARCHIVES_PACKER);
    expect(raised[0].details).toContain("Archive packing");
  });
});
