import { describe, expect, it } from "@jest/globals";
import { EventBus, WireEvent } from "@wirestate/core";
import { flow } from "@wirestate/mobx";
import { isCancellation } from "@xrf/flow";
import { Logger } from "@xrf/logging";

import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockContainer } from "@/fixtures/utils/container";
import { deferred } from "@/fixtures/utils/promise";

import { CommandOperation } from "./command-operation";
import { CommandReporter, ICommandReport } from "./command-reporter";

function setup() {
  const bus: EventBus = mockContainer().get(EventBus);
  const raised: Array<INotificationPayload> = [];
  const operation = new CommandOperation<string>(
    new Logger("test"),
    new CommandReporter(new Logger("test"), bus, "exports-explorer")
  );

  bus.subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
    raised.push(event.payload as INotificationPayload)
  );

  return { operation, raised };
}

function report(command: () => Promise<string>, patch: Partial<ICommandReport<string>> = {}): ICommandReport<string> {
  return { name: "Write manifest", subject: "C:\\out.json", command, failure: "Could not write", ...patch };
}

function run(operation: CommandOperation<string>, command: ICommandReport<string>) {
  return flow(function* () {
    return yield* operation.run(command);
  })();
}

describe("CommandOperation", () => {
  it("publishes loading, then the answer and the success notice the run describes", async () => {
    const { operation, raised } = setup();
    const answer = deferred<string>();
    const running = run(
      operation,
      report(() => answer.promise, { success: (path) => ({ title: "Wrote", details: path }) })
    );

    expect(operation.isLoading).toBe(true);

    answer.resolve("C:\\out.json");

    await expect(running).resolves.toEqual({ result: "C:\\out.json", error: null });
    expect(operation.state.isReady).toBe(true);
    expect(operation.value).toBe("C:\\out.json");
    expect(raised).toEqual([
      { details: "C:\\out.json", severity: ENotificationSeverity.SUCCESS, source: "exports-explorer", title: "Wrote" },
    ]);
  });

  it("keeps the previous answer where a run fails, and raises the failure with its subject", async () => {
    const { operation, raised } = setup();

    await run(
      operation,
      report(() => Promise.resolve("first"))
    );
    await expect(
      run(
        operation,
        report(() => Promise.reject(new Error("read only")))
      )
    ).resolves.toEqual({
      result: null,
      error: new Error("read only"),
    });

    expect(operation.value).toBe("first");
    expect(operation.error).toEqual(new Error("read only"));
    expect(raised).toEqual([
      {
        details: "C:\\out.json\nread only",
        severity: ENotificationSeverity.ERROR,
        source: "exports-explorer",
        title: "Could not write",
      },
    ]);
  });

  it("raises a failure naming no subject with the message alone, at the severity asked for", async () => {
    const { operation, raised } = setup();

    await run(
      operation,
      report(() => Promise.reject(new Error("gone")), {
        subject: null,
        failureSeverity: ENotificationSeverity.WARNING,
      })
    );

    expect(raised).toEqual([
      {
        details: "gone",
        severity: ENotificationSeverity.WARNING,
        source: "exports-explorer",
        title: "Could not write",
      },
    ]);
  });

  it("publishes nothing for a run cancelled while the command runs", async () => {
    const { operation, raised } = setup();
    const answer = deferred<string>();

    await run(
      operation,
      report(() => Promise.resolve("first"))
    );

    const running = run(
      operation,
      report(() => answer.promise, { success: () => ({ title: "Wrote" }) })
    );

    running.cancel();
    answer.resolve("second");

    await expect(running.catch((error: unknown) => isCancellation(error))).resolves.toBe(true);
    expect(operation.isLoading).toBe(false);
    expect(operation.value).toBe("first");
    expect(raised).toEqual([]);
  });

  it("forgets the last outcome on reset", async () => {
    const { operation } = setup();

    await run(
      operation,
      report(() => Promise.reject(new Error("read only")))
    );

    operation.reset();

    expect(operation.state.isIdle).toBe(true);
    expect(operation.error).toBeNull();
  });
});
