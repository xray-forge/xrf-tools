import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { autorun, flow } from "@wirestate/mobx";

import { EJobKind, IJobDescriptor, IJobSettledPayload } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { mockInvoke, resetMockInvoke } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { noop } from "@/lib/callbacks/noop";
import { Logger } from "@/lib/logging";

interface IResult {
  count: number;
}

function deferred<T>() {
  let resolve: (value: T) => void = noop;

  const promise = new Promise<T>((settle) => {
    resolve = settle;
  });

  return { promise, resolve };
}

function setup(kinds: ReadonlyArray<EJobKind> = [EJobKind.CONFIGS_VERIFY]) {
  const { service: jobs } = mockInjectedService(JobsService);
  const operation = new JobOperation<IResult>(jobs, kinds, new Logger("job-operation.test"));

  return { operation, jobs };
}

function descriptor(answer: Promise<IResult>): IJobDescriptor<IResult> {
  return {
    kind: EJobKind.CONFIGS_VERIFY,
    invoke: () => answer,
    describe: () => ({ title: "Verified", severity: ENotificationSeverity.SUCCESS }),
  };
}

function settled(patch: Partial<IJobSettledPayload> = {}): IJobSettledPayload {
  return {
    id: "adopted",
    kind: EJobKind.CONFIGS_VERIFY,
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

    expect(operation.job?.kind).toBe(EJobKind.CONFIGS_VERIFY);
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
    const { operation } = setup();

    operation.adopt(settled());

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

    expect(mockInvoke).toHaveBeenCalledWith("plugin:jobs|cancel", { id });
    expect(operation.job?.isCancelRequested).toBe(true);
    expect(operation.isRunning).toBe(true);

    answer.resolve({ count: 2 });
    await running;

    expect(operation.result).toEqual({ count: 2 });
  });

  it("keeps an abandoned run from publishing over its replacement", async () => {
    const { operation, jobs } = setup();
    const oldAnswer = deferred<IResult>();
    const oldRun = flow(function* () {
      return yield* operation.run(descriptor(oldAnswer.promise));
    })();
    const abandoned = oldRun.catch(() => undefined);

    oldRun.cancel();

    expect(jobs.jobs).toHaveLength(1);
    expect(operation.isRunning).toBe(true);

    await flow(function* () {
      yield* operation.run(descriptor(Promise.resolve({ count: 9 })));
    })();

    oldAnswer.resolve({ count: 1 });
    await abandoned;
    await oldAnswer.promise;

    expect(operation.result).toEqual({ count: 9 });
    expect(jobs.jobs).toHaveLength(0);
  });

  it("rediscovers either formatter mode and cancels the adopted job", () => {
    const { operation, jobs } = setup([EJobKind.CONFIGS_CHECK_FORMAT, EJobKind.CONFIGS_FORMAT]);

    jobs.jobs = [
      {
        id: "format",
        kind: EJobKind.CONFIGS_FORMAT,
        request: null,
        progress: null,
        isAdopted: true,
        isCancelRequested: false,
      },
    ];

    expect(operation.isRunning).toBe(true);

    operation.cancel();

    expect(mockInvoke).toHaveBeenCalledWith("plugin:jobs|cancel", { id: "format" });
  });

  it("adopts matching success and failure outcomes but ignores other commands", () => {
    const { operation } = setup();

    operation.adopt(settled());
    operation.adopt(settled({ kind: EJobKind.TRANSLATIONS_BUILD, result: { count: 99 } }));

    expect(operation.result).toEqual({ count: 4 });

    operation.adopt(settled({ conclusion: "failed", error: "cannot read", result: null }));

    expect(operation.result).toBeNull();
    expect(operation.error).toBe("cannot read");

    operation.adopt(undefined);

    expect(operation.error).toBe("cannot read");
  });

  it("does not let an adopted outcome replace a run it is awaiting", async () => {
    const { operation } = setup();
    const answer = deferred<IResult>();
    const running = flow(function* () {
      return yield* operation.run(descriptor(answer.promise));
    })();

    operation.adopt(settled());
    operation.reset();

    expect(operation.result).toBeNull();
    expect(operation.isRunning).toBe(true);

    answer.resolve({ count: 3 });
    await running;

    expect(operation.result).toEqual({ count: 3 });
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
});
