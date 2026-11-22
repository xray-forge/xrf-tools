import { afterEach, beforeAll, describe, expect, it, jest } from "@jest/globals";

import type { Logger as LoggerClass } from "#/logger";
import { ELogLevel, ILogSink } from "#/logger.types";

describe("Logger", () => {
  const sink: jest.Mocked<ILogSink> = { write: jest.fn() };

  let Logger: typeof LoggerClass;

  // The logger takes the console's methods as its module loads, so they are silenced before it is imported.
  beforeAll(async () => {
    for (const method of ["log", "info", "warn", "error", "debug"] as const) {
      jest.spyOn(console, method).mockImplementation(() => {});
    }

    ({ Logger } = await import("#/logger"));
  });

  afterEach(() => {
    Logger.sink = null;
    Logger.IS_GLOBAL_LOGGING_ENABLED = true;
    sink.write.mockClear();
  });

  it("hands each line to the sink with its level, scope and values", () => {
    Logger.sink = sink;

    const failure: Error = new Error("failed");
    const logger: LoggerClass = new Logger("scope");

    logger.error("could not read", failure);
    logger.warn("slow");
    logger.info("opened", 3);
    logger.debug("detail");

    expect(sink.write.mock.calls).toEqual([
      [ELogLevel.ERROR, "scope", ["could not read", failure]],
      [ELogLevel.WARN, "scope", ["slow"]],
      [ELogLevel.INFO, "scope", ["opened", 3]],
      [ELogLevel.DEBUG, "scope", ["detail"]],
    ]);
  });

  it("keeps a general message as an informational one", () => {
    Logger.sink = sink;

    new Logger("scope").log("general");

    expect(sink.write).toHaveBeenCalledWith(ELogLevel.INFO, "scope", ["general"]);
  });

  it("reaches a sink installed after the logger was made", () => {
    const logger: LoggerClass = new Logger("early");

    Logger.sink = sink;
    logger.info("late line");

    expect(sink.write).toHaveBeenCalledWith(ELogLevel.INFO, "early", ["late line"]);
  });

  it("records a line to the sink alone", () => {
    Logger.sink = sink;

    const failure: Error = new Error("uncaught");

    jest.mocked(console.error).mockClear();
    new Logger("scope").record(ELogLevel.ERROR, "Unhandled rejection:", failure);

    expect(sink.write).toHaveBeenCalledWith(ELogLevel.ERROR, "scope", ["Unhandled rejection:", failure]);
    expect(console.error).not.toHaveBeenCalled();
  });

  it("writes nothing from a disabled logger", () => {
    Logger.sink = sink;

    const logger: LoggerClass = new Logger("scope", false);

    logger.error("unseen");
    logger.record(ELogLevel.ERROR, "unseen");

    expect(sink.write).not.toHaveBeenCalled();
  });

  it("writes nothing from a logger made while logging is off", () => {
    Logger.sink = sink;
    Logger.IS_GLOBAL_LOGGING_ENABLED = false;

    new Logger("scope").error("unseen");

    expect(sink.write).not.toHaveBeenCalled();
  });
});
