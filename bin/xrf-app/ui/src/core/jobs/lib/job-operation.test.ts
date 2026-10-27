import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container, EventBus } from "@wirestate/core";
import { autorun, flow } from "@wirestate/mobx";
import { EJobsCommand } from "@xrf/ipc/commands/jobs";
import { SessionScopeService } from "@xrf/ipc/session";
import { mockInvoke, resetMockInvoke } from "@xrf/ipc/testing";
import { CONFIGS_JOB_CHECK_FORMAT, CONFIGS_JOB_FORMAT, CONFIGS_JOB_VERIFY } from "@xrf/ipc/types/xrf-app-configs";
import { TRANSLATIONS_JOB_BUILD } from "@xrf/ipc/types/xrf-app-translations";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { IJobDescriptor, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { mockContainer } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

import { IJobOperationOptions, JobOperation } from "./job-operation";

interface IResult {
  count: number;
}

function setup(options: IJobOperationOptions<IResult> = { kinds: [CONFIGS_JOB_VERIFY] }) {
  const container: Container = mockContainer();
  const jobs: JobsService = container.get(JobsService);
  const scope: SessionScopeService = container.get(SessionScopeService);
  const bus: EventBus = container.get(EventBus);
  const operation: JobOperation<IResult> = new JobOperation(new Logger("job-operation"), options, jobs, scope, bus);

  function settle(patch: Partial<IJobSettledPayload> = {}): void {
    bus.emit<IJobSettledPayload>(JOB_SETTLED_EVENT, settled(patch));
  }

  return { operation, jobs, scope, settle };
}

function descriptor(answer: Promise<IResult>): IJobDescriptor<IResult> {
  return {
    kind: CONFIGS_JOB_VERIFY,
    invoke: jest.fn(() => answer),
    describe: () => ({ title: "Verified", severity: ENotificationSeverity.SUCCESS }),
  };
}

function settled(patch: Partial<IJobSettledPayload> = {}): IJobSettledPayload {
  return {
    id: "adopted",
    kind: CONFIGS_JOB_VERIFY,
    conclusion: "completed",
    error: null,
    result: { count: 4 },
    ...patch,
  };
}

describe("JobOperation", () => {
  beforeEach(resetMockInvoke);

  it("publishes loading and completion through observable state", async () => {
    const { operation } = setup();
    const answer = deferred<IResult>();
    const seen: Array<string> = [];
    const stop = autorun(() => seen.push(`${operation.isRunning}:${operation.result?.count ?? "empty"}`));
    const running = flow(function* () {
      return yield* operation.run(descriptor(answer.promise));
    })();

    expect(operation.job?.kind).toBe(CONFIGS_JOB_VERIFY);
    expect(operation.isRunning).toBe(true);

    answer.resolve({ count: 7 });

    await expect(running).resolves.toEqual({ result: { count: 7 }, error: null });

    expect(operation.isRunning).toBe(false);
    expect(operation.job).toBeNull();
    expect(seen).toContain("true:empty");
    expect(seen).toContain("false:7");

    stop();
  });

  it("normalizes rejection and clears the previous result before retrying", async () => {
    const { operation, settle } = setup();

    settle();

    const running = flow(function* () {
      return yield* operation.run(descriptor(Promise.reject("backend refused")));
    })();

    expect(operation.result).toBeNull();

    await expect(running).resolves.toEqual({ result: null, error: new Error("backend refused") });

    expect(operation.error).toBe("backend refused");
    expect(operation.isRunning).toBe(false);

    operation.reset();

    expect(operation.error).toBeNull();
    expect(operation.result).toBeNull();
  });

  it("requests cancellation without discarding the eventual partial result", async () => {
    const { operation } = setup();
    const answer = deferred<IResult>();
    const running = flow(function* () {
      return yield* operation.run(descriptor(answer.promise));
    })();
    const id = operation.job?.id;

    operation.cancel();

    expect(mockInvoke).toHaveBeenCalledWith(EJobsCommand.CANCEL, { id });
    expect(operation.job?.isCancelRequested).toBe(true);
    expect(operation.isRunning).toBe(true);

    answer.resolve({ count: 2 });
    await running;

    expect(operation.result).toEqual({ count: 2 });
  });

  it("ignores a run while an abandoned one still runs, and shows the abandoned one's answer as it settles", async () => {
    const { operation, jobs } = setup();
    const oldAnswer = deferred<IResult>();
    const oldRun = flow(function* () {
      return yield* operation.run(descriptor(oldAnswer.promise));
    })();
    const abandoned = oldRun.catch(() => undefined);

    oldRun.cancel();

    expect(jobs.jobs).toHaveLength(1);
    expect(operation.isRunning).toBe(true);

    const next: IJobDescriptor<IResult> = descriptor(Promise.resolve({ count: 9 }));
    const ignored = flow(function* () {
      return yield* operation.run(next);
    })();

    await expect(ignored).resolves.toBeNull();
    expect(next.invoke).not.toHaveBeenCalled();

    oldAnswer.resolve({ count: 1 });
    await abandoned;
    await oldAnswer.promise;

    expect(operation.result).toEqual({ count: 1 });
    expect(jobs.jobs).toHaveLength(0);
  });

  it("ignores a run while a job of its kinds found after a reload is in flight", async () => {
    const { operation, jobs } = setup();

    jobs.jobs = [
      {
        id: "found",
        kind: CONFIGS_JOB_VERIFY,
        request: null,
        progress: null,
        isAdopted: true,
        isCancelRequested: false,
      },
    ];

    const next: IJobDescriptor<IResult> = descriptor(Promise.resolve({ count: 9 }));
    const ignored = flow(function* () {
      return yield* operation.run(next);
    })();

    await expect(ignored).resolves.toBeNull();
    expect(next.invoke).not.toHaveBeenCalled();
    expect(operation.isRunning).toBe(true);
  });

  it("rediscovers either formatter mode and cancels the adopted job", () => {
    const { operation, jobs } = setup({ kinds: [CONFIGS_JOB_CHECK_FORMAT, CONFIGS_JOB_FORMAT] });

    jobs.jobs = [
      {
        id: "format",
        kind: CONFIGS_JOB_FORMAT,
        request: null,
        progress: null,
        isAdopted: true,
        isCancelRequested: false,
      },
    ];

    expect(operation.isRunning).toBe(true);

    operation.cancel();

    expect(mockInvoke).toHaveBeenCalledWith(EJobsCommand.CANCEL, { id: "format" });
  });

  it("shows settled success and failure of its kinds, telling the owner, and ignores other commands", () => {
    const adopted: Array<Nullable<IResult>> = [];
    const { operation, settle } = setup({
      kinds: [CONFIGS_JOB_VERIFY],
      adopted: (result: Nullable<IResult>) => adopted.push(result),
    });

    settle();
    settle({ kind: TRANSLATIONS_JOB_BUILD, result: { count: 99 } });

    expect(operation.result).toEqual({ count: 4 });

    settle({ conclusion: "failed", error: "cannot read", result: null });

    expect(operation.result).toBeNull();
    expect(operation.error).toBe("cannot read");
    expect(adopted).toEqual([{ count: 4 }, null]);
  });

  it("does not let a settled outcome replace a run it is awaiting", async () => {
    const { operation, settle } = setup();
    const answer = deferred<IResult>();
    const running = flow(function* () {
      return yield* operation.run(descriptor(answer.promise));
    })();

    settle();
    operation.reset();

    expect(operation.result).toBeNull();
    expect(operation.isRunning).toBe(true);

    answer.resolve({ count: 3 });
    await running;

    expect(operation.result).toEqual({ count: 3 });
  });

  it("stops following settled jobs once its tool's scope is released", () => {
    const { operation, scope, settle } = setup();

    scope.release();
    settle();

    expect(operation.result).toBeNull();
  });

  it("reports a start failure without leaving the operation loading", async () => {
    const { operation, jobs } = setup();

    jest.spyOn(jobs, "run").mockImplementation(() => {
      throw new Error("cannot start");
    });

    await flow(function* () {
      yield* operation.run(descriptor(Promise.resolve({ count: 1 })));
    })();

    expect(operation.error).toBe("cannot start");
    expect(operation.isRunning).toBe(false);
  });

  it("clears a failure without discarding its partial result", () => {
    const { operation, settle } = setup();

    settle({ conclusion: "failed", error: "could not finish", result: { count: 2 } });
    operation.clearError();

    expect(operation.error).toBeNull();
    expect(operation.result).toEqual({ count: 2 });
    expect(operation.isRunning).toBe(false);
  });

  it("leaves a pending run active when its error is cleared", async () => {
    const { operation } = setup();
    const answer = deferred<IResult>();
    const running = flow(function* () {
      return yield* operation.run(descriptor(answer.promise));
    })();
    const job = operation.job;

    operation.clearError();

    expect(operation.isRunning).toBe(true);
    expect(operation.job).toBe(job);

    answer.resolve({ count: 3 });
    await running;

    expect(operation.result).toEqual({ count: 3 });
  });
});
