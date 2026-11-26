import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container, EventBus, WireEvent } from "@wirestate/core";
import { ELogLevel, ILogSink, LOGGER_CONFIG } from "@xrf/logging";

import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { ErrorCaptureService } from "@/core/notifications/services/error-capture/error-capture.service";
import { mockContainer } from "@/fixtures/utils/container";

interface IWatchedCapture {
  container: Container;
  service: ErrorCaptureService;
  raised: Array<INotificationPayload>;
}

/**
 * Create an error capture service and collect the notifications it emits.
 *
 * @returns Provisionable container, service instance, and emitted payload collection.
 */
function watchCapture(): IWatchedCapture {
  const container: Container = mockContainer([ErrorCaptureService]);

  const raised: Array<INotificationPayload> = [];

  container
    .get(EventBus)
    .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
      raised.push(event.payload as INotificationPayload)
    );

  return { container, raised, service: container.get(ErrorCaptureService) };
}

describe("ErrorCaptureService", () => {
  const sink: jest.Mocked<ILogSink> = { write: jest.fn() };

  beforeEach(() => {
    jest.restoreAllMocks();
    sink.write.mockReset();
    LOGGER_CONFIG.install(sink);
  });

  afterEach(() => {
    LOGGER_CONFIG.release(sink);
  });

  it("registers and releases the same handler references", () => {
    const added: Array<[string, unknown]> = [];
    const removed: Array<[string, unknown]> = [];

    jest.spyOn(window, "addEventListener").mockImplementation((type: string, handler: unknown) => {
      added.push([type, handler]);
    });

    jest.spyOn(window, "removeEventListener").mockImplementation((type: string, handler: unknown) => {
      removed.push([type, handler]);
    });

    const { container }: IWatchedCapture = watchCapture();

    container.provision();
    container.unbindAll();

    // The whole reason the handlers are bound actions rather than plain methods: a fresh function per
    // registration would leave `removeEventListener` a no-op and the listeners attached for good.
    expect(added.map(([type]: [string, unknown]) => type)).toEqual(["error", "unhandledrejection"]);
    expect(removed).toEqual(added);
  });

  it("records a dropped rejection, which nothing else reports", () => {
    const { raised, service }: IWatchedCapture = watchCapture();

    service.onUnhandledRejection({ reason: new Error("nobody caught this") } as PromiseRejectionEvent);

    expect(raised).toHaveLength(1);
    expect(raised[0].severity).toBe(ENotificationSeverity.DEV);
    expect(raised[0].source).toBe("application");
    expect(raised[0].title).toBe("nobody caught this");
  });

  it("records where an uncaught exception came from", () => {
    const { raised, service }: IWatchedCapture = watchCapture();

    service.onWindowError({
      colno: 7,
      error: new Error("exploded"),
      filename: "http://localhost/src/thing.ts",
      lineno: 42,
      message: "exploded",
    } as ErrorEvent);

    expect(raised).toHaveLength(1);
    expect(raised[0].title).toBe("exploded");
    expect(raised[0].details).toBe("http://localhost/src/thing.ts:42:7");
  });

  it("ignores the resource load failures that arrive on the same event", () => {
    const { raised, service }: IWatchedCapture = watchCapture();

    service.onWindowError({ error: null, message: "" } as unknown as ErrorEvent);

    expect(raised).toHaveLength(0);
    expect(sink.write).not.toHaveBeenCalled();
  });

  describe("error records", () => {
    // The suite turns logging off, and nothing is recorded while it is.
    beforeEach(() => {
      LOGGER_CONFIG.setEnabled(true);
    });

    afterEach(() => {
      LOGGER_CONFIG.setEnabled(false);
    });

    it("writes an uncaught exception with its location and stack", () => {
      const { service }: IWatchedCapture = watchCapture();
      const error: Error = new Error("exploded");

      service.onWindowError({
        colno: 7,
        error,
        filename: "http://localhost/src/thing.ts",
        lineno: 42,
        message: "exploded",
      } as ErrorEvent);

      expect(sink.write).toHaveBeenCalledTimes(1);
      expect(sink.write).toHaveBeenCalledWith(ELogLevel.ERROR, service.log.prefix, [
        "Uncaught error at http://localhost/src/thing.ts:42:7:",
        error,
      ]);
    });

    it("writes the message of an uncaught failure that carries no error", () => {
      const { service }: IWatchedCapture = watchCapture();

      service.onWindowError({ error: null, message: "Script error." } as unknown as ErrorEvent);

      expect(sink.write).toHaveBeenCalledWith(ELogLevel.ERROR, service.log.prefix, [
        "Uncaught error at unknown location:",
        "Script error.",
      ]);
    });

    it("writes a dropped rejection with its stack, or its message for a reason that is no error", () => {
      const { service }: IWatchedCapture = watchCapture();
      const error: Error = new Error("nobody caught this");

      service.onUnhandledRejection({ reason: error } as PromiseRejectionEvent);
      service.onUnhandledRejection({ reason: "plain" } as PromiseRejectionEvent);

      expect(sink.write.mock.calls).toEqual([
        [ELogLevel.ERROR, service.log.prefix, ["Unhandled rejection:", error]],
        [ELogLevel.ERROR, service.log.prefix, ["Unhandled rejection:", "plain"]],
      ]);
    });

    it("does not print the failure to the console again", () => {
      const { service }: IWatchedCapture = watchCapture();
      const printed: jest.SpiedFunction<typeof console.error> = jest.spyOn(console, "error");

      service.onUnhandledRejection({ reason: new Error("quiet") } as PromiseRejectionEvent);

      expect(printed).not.toHaveBeenCalled();
    });

    it("records nothing while a failure is being recorded", () => {
      const { raised, service }: IWatchedCapture = watchCapture();

      sink.write.mockImplementationOnce(() => {
        service.onUnhandledRejection({ reason: new Error("again") } as PromiseRejectionEvent);
      });

      service.onUnhandledRejection({ reason: new Error("first") } as PromiseRejectionEvent);

      expect(sink.write).toHaveBeenCalledTimes(1);
      expect(raised.map((it: INotificationPayload) => it.title)).toEqual(["first"]);
    });
  });
});
