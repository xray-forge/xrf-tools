import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";

import { JobProgress } from "@/core/bindings/types/xrf-job";
import { EJobKind, IJobNotice, IJobOutcome, IJobState } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs/jobs.service";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import {
  emitMockChannelMessage,
  getMockChannels,
  resetMockChannels,
  resetMockInvoke,
  setMockInvokeResponses,
} from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";
import { Nullable } from "@/lib/types/general";

const CANCEL_COMMAND: string = "plugin:jobs|cancel";

function progress(completed: number, total: Nullable<number> = 10): JobProgress {
  return {
    levels: [{ id: "write", label: null, completed, total, unit: "items" }],
    duration: 1000,
    detail: null,
  };
}

/** A descriptor whose command never settles, so the job stays running for the assertion. */
function pending(): { descriptor: Parameters<JobsService["run"]>[0]; settle: (value: unknown) => void } {
  let settle: (value: unknown) => void = noop;

  const promise: Promise<unknown> = new Promise((resolve) => {
    settle = resolve;
  });

  return {
    descriptor: {
      kind: EJobKind.ARCHIVES_PACK,
      invoke: () => promise,
      describe: (): IJobNotice => ({
        severity: ENotificationSeverity.SUCCESS,
        title: "Packed",
      }),
    } as Parameters<JobsService["run"]>[0],
    settle,
  };
}

describe("JobsService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
  });

  it("lists a job before its command can answer", async () => {
    // The entry has to exist by the time `run` returns: a caller rendering from `jobs` would otherwise see a gap
    // between asking for work and anything appearing, which reads as the control having done nothing.
    const { service } = mockInjectedService(JobsService);
    const { descriptor, settle } = pending();

    const run = service.run(descriptor);

    expect(service.jobs).toHaveLength(1);
    expect(service.jobs[0].id).toBe(run.id);
    expect(service.jobs[0].kind).toBe("archives.pack");
    expect(service.jobs[0].progress).toBeNull();

    settle(null);
    await run.promise;
  });

  it("mints a fresh identity for every run", async () => {
    // A reused identity would let a cancel aimed at one run land on its successor, which is the stale-run hazard the
    // whole per-run identity rule exists to avoid.
    const { service } = mockInjectedService(JobsService);
    const first = pending();
    const second = pending();

    const one = service.run(first.descriptor);
    const two = service.run(second.descriptor);

    expect(one.id).not.toBe(two.id);

    first.settle(null);
    second.settle(null);
    await Promise.all([one.promise, two.promise]);
  });

  it("routes a snapshot to the job whose channel carried it", async () => {
    const { service } = mockInjectedService(JobsService);
    const { descriptor, settle } = pending();

    const run = service.run(descriptor);

    emitMockChannelMessage(progress(4));

    expect(service.getJob(run.id)?.progress?.levels[0].completed).toBe(4);

    settle(null);
    await run.promise;
  });

  it("keeps each run's snapshots to its own job", async () => {
    // Two channels, two jobs. The transport is what separates them, so a snapshot cannot reach the wrong reader even
    // though both runs are the same kind.
    const { service } = mockInjectedService(JobsService);
    const first = pending();
    const second = pending();

    const one = service.run(first.descriptor);
    const two = service.run(second.descriptor);

    getMockChannels()[0].onmessage(progress(1));
    getMockChannels()[1].onmessage(progress(7));

    expect(service.getJob(one.id)?.progress?.levels[0].completed).toBe(1);
    expect(service.getJob(two.id)?.progress?.levels[0].completed).toBe(7);

    first.settle(null);
    second.settle(null);
    await Promise.all([one.promise, two.promise]);
  });

  it("marks a job as asked to stop and tells the backend", async () => {
    // The request is recorded rather than the job removed: cancellation lands at a boundary the run chooses, and
    // hiding the gap would make the control look broken.
    setMockInvokeResponses({ [CANCEL_COMMAND]: true });

    const { service } = mockInjectedService(JobsService);
    const { descriptor, settle } = pending();

    const run = service.run(descriptor);

    service.cancel(run.id);

    expect(service.getJob(run.id)?.isCancelRequested).toBe(true);

    settle(null);
    await run.promise;
  });

  it("forgets a job once it settles", async () => {
    const { service } = mockInjectedService(JobsService);
    const { descriptor, settle } = pending();

    const run = service.run(descriptor);

    settle(null);
    await run.promise;

    expect(service.jobs).toHaveLength(0);
    expect(service.getJob(run.id)).toBeNull();
  });

  it("forgets a job that failed and hands the failure on", async () => {
    const { service } = mockInjectedService(JobsService);

    const run = service.run({
      kind: EJobKind.ARCHIVES_PACK,
      invoke: () => Promise.reject(new Error("volume cap refuses particles.xr")),
      describe: (): IJobNotice => ({
        severity: ENotificationSeverity.ERROR,
        title: "Failed",
      }),
    });

    await expect(run.promise).rejects.toThrow("volume cap refuses particles.xr");
    expect(service.jobs).toHaveLength(0);
  });

  it("finds a running job by kind, which is how a rebuilt view re-attaches", async () => {
    // A tool's own service dies with its container while the job keeps running. The kind is the only handle that
    // survives that, and without it the user would come back to an idle form over work still in flight.
    const { service } = mockInjectedService(JobsService);
    const { descriptor, settle } = pending();

    const run = service.run(descriptor);

    expect(service.getJobOfKind(EJobKind.ARCHIVES_PACK)?.id).toBe(run.id);
    expect(service.getJobOfKind(EJobKind.ARCHIVES_UNPACK)).toBeNull();

    settle(null);
    await run.promise;

    expect(service.getJobOfKind(EJobKind.ARCHIVES_PACK)).toBeNull();
  });

  it("notifies exactly once, when the job ends and not while it runs", async () => {
    // The rule the notification surface depends on: a phase transition is not an outcome, and a run that reported one
    // per phase would bury the one line that matters.
    const { service, container } = mockInjectedService(JobsService);
    const raised: Array<INotificationPayload> = [];

    container
      .get(EventBus)
      .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
        raised.push(event.payload as INotificationPayload)
      );

    const { descriptor, settle } = pending();
    const run = service.run(descriptor);

    emitMockChannelMessage(progress(1));
    emitMockChannelMessage(progress(2));

    expect(raised).toHaveLength(0);

    settle(null);
    await run.promise;

    expect(raised).toHaveLength(1);
    expect(raised[0].title).toBe("Packed");
  });

  it("tells the tool whether stopping was asked for when it describes the outcome", async () => {
    // The service knows the request was made; only the tool can read its own payload to see what the run then did.
    setMockInvokeResponses({ [CANCEL_COMMAND]: true });

    const { service } = mockInjectedService(JobsService);
    const describe = jest.fn((outcome: IJobOutcome<unknown>): INotificationPayload => {
      expect(outcome.isCancelRequested).toBe(true);

      return { severity: ENotificationSeverity.INFO, source: "archives-packer", title: "Stopped" };
    });

    let settle: (value: unknown) => void = noop;
    const pendingPromise: Promise<unknown> = new Promise((resolve) => {
      settle = resolve;
    });

    const run = service.run({
      kind: EJobKind.ARCHIVES_PACK,
      invoke: () => pendingPromise,
      describe,
    });

    service.cancel(run.id);
    settle(null);

    await run.promise;

    expect(describe).toHaveBeenCalledTimes(1);
  });

  it("drops a snapshot that arrives after its job settled", async () => {
    // A late snapshot must not resurrect a finished job. The channel is dropped with the run, but the transport is
    // asynchronous and this is the one ordering the service cannot control.
    const { service } = mockInjectedService(JobsService);
    const { descriptor, settle } = pending();

    const run = service.run(descriptor);
    const channel = getMockChannels()[0];

    settle(null);
    await run.promise;

    channel.onmessage(progress(9));

    expect(service.jobs).toHaveLength(0);
  });

  it("keeps working when a cancel is refused because the job already finished", async () => {
    // Ordinary: the control was pressed a moment too late. The run's own answer reports the outcome either way.
    setMockInvokeResponses({
      [CANCEL_COMMAND]: () => {
        throw new Error("no such job");
      },
    });

    const { service } = mockInjectedService(JobsService);
    const { descriptor, settle } = pending();

    const run = service.run(descriptor);

    expect(() => service.cancel(run.id)).not.toThrow();

    const job: Nullable<IJobState> = service.getJob(run.id);

    expect(job?.isCancelRequested).toBe(true);

    settle(null);
    await run.promise;
  });
});
