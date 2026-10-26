import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";

import { mockJobDescription } from "@/core/jobs/fixtures/job.mocks";
import {
  JOBS_ATTACH_COMMAND,
  JOBS_LIST_COMMAND,
  JOBS_POLL_INTERVAL,
  mockWatchedJobsService,
} from "@/core/jobs/fixtures/jobs-service";

describe("JobsService adoption ending", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it("hands the answer of a settled adopted job to whoever owns that kind", async () => {
    // Nothing here awaited this run: the command answered a page that no longer exists. The copy the backend retained
    // is the only way the tool can render what the pack produced.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()], [JOBS_ATTACH_COMMAND]: true });

    const { service, settled } = mockWatchedJobsService();

    await service.onProvision(1);

    mockIpcCommands({
      [JOBS_LIST_COMMAND]: [mockJobDescription({ conclusion: "completed", result: { volumes: ["textures.db0"] } })],
      [JOBS_ATTACH_COMMAND]: true,
    });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(settled).toHaveLength(1);
    expect(settled[0].kind).toBe("archives.pack");
    expect(settled[0].conclusion).toBe("completed");
    expect(settled[0].result).toEqual({ volumes: ["textures.db0"] });
  });

  it("announces a settled adopted job that answered nothing", async () => {
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()], [JOBS_ATTACH_COMMAND]: true });

    const { service, settled } = mockWatchedJobsService();

    await service.onProvision(1);

    mockIpcCommands({
      [JOBS_LIST_COMMAND]: [mockJobDescription({ conclusion: "failed", error: "volume cap refuses particles.xr" })],
      [JOBS_ATTACH_COMMAND]: true,
    });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(settled[0].result).toBeNull();
    expect(settled[0].error).toBe("volume cap refuses particles.xr");
  });

  it("follows an adopted job by asking, because its channel died with the page", async () => {
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()] });

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
    });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(service.getJob("b8f0")?.progress?.levels[0].completed).toBe(7);
  });

  it("announces an adopted job that ended, and forgets it", async () => {
    // Nobody is awaiting its answer, so the polled conclusion is the only way the user learns their pack finished.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()] });

    const { service, raised } = mockWatchedJobsService();

    await service.onProvision(1);

    expect(raised).toHaveLength(0);

    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription({ conclusion: "completed" })] });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(service.jobs).toHaveLength(0);
    expect(raised).toHaveLength(1);
    expect(raised[0].title).toContain("reloading");
  });

  it("stops asking once nothing is adopted", async () => {
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()] });

    const { service } = mockWatchedJobsService();

    await service.onProvision(1);

    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription({ conclusion: "completed" })] });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(jest.getTimerCount()).toBe(0);
  });

  it("treats a job that left the listing as ended with an unrecorded outcome", async () => {
    // Twenty finished jobs push it out of the retained ring. It ended; how is no longer knowable.
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()] });

    const { service, raised } = mockWatchedJobsService();

    await service.onProvision(1);

    mockIpcCommands({ [JOBS_LIST_COMMAND]: [] });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(service.jobs).toHaveLength(0);
    expect(raised[0].details).toContain("no longer recorded");
  });

  it("keeps working when the listing cannot be read", async () => {
    // A start-up is a bad moment to fail loudly, and the lease still prevents a duplicate from doing any damage.
    mockIpcCommands({
      [JOBS_LIST_COMMAND]: () => {
        throw new Error("state unavailable");
      },
    });

    const { service } = mockWatchedJobsService();

    await expect(service.onProvision(1)).resolves.toBeUndefined();
    expect(service.jobs).toHaveLength(0);
  });

  it("keeps watching through a failed poll and settles only after recovery", async () => {
    mockIpcCommands({ [JOBS_LIST_COMMAND]: [mockJobDescription()], [JOBS_ATTACH_COMMAND]: true });

    const { service, raised, settled } = mockWatchedJobsService();

    await service.onProvision(1);

    mockIpcCommands({
      [JOBS_LIST_COMMAND]: () => {
        throw new Error("temporarily unavailable");
      },
    });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL * 2);

    expect(service.getJob("b8f0")?.isAdopted).toBe(true);
    expect(raised).toHaveLength(0);
    expect(settled).toHaveLength(0);

    mockIpcCommands({
      [JOBS_LIST_COMMAND]: [mockJobDescription({ conclusion: "completed", result: { volumes: ["textures.db0"] } })],
    });

    await jest.advanceTimersByTimeAsync(JOBS_POLL_INTERVAL);

    expect(service.jobs).toHaveLength(0);
    expect(settled).toHaveLength(1);
    expect(settled[0].result).toEqual({ volumes: ["textures.db0"] });
    expect(raised).toHaveLength(1);
    expect(jest.getTimerCount()).toBe(0);
  });
});
