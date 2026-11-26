import { afterEach, beforeAll, describe, expect, it, jest } from "@jest/globals";

import type { Logger as LoggerClass } from "#/logger";
import type { LoggerConfig } from "#/logger-config";
import { ELogLevel, ILogSink } from "#/logger.types";

describe("Logger", () => {
  const sink: jest.Mocked<ILogSink> = { write: jest.fn() };

  let Logger: typeof LoggerClass;
  let config: LoggerConfig;

  // The logger takes the console's methods as its module loads, so they are silenced before it is imported.
  beforeAll(async () => {
    for (const method of ["log", "info", "warn", "error", "debug"] as const) {
      jest.spyOn(console, method).mockImplementation(() => {});
    }

    ({ Logger } = await import("#/logger"));
    ({ LOGGER_CONFIG: config } = await import("#/logger-config"));
  });

  afterEach(() => {
    config.reset();
    sink.write.mockClear();
  });

  it("hands each line to the sink with its level, scope and values", () => {
    config.install(sink);

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
    config.install(sink);

    new Logger("scope").log("general");

    expect(sink.write).toHaveBeenCalledWith(ELogLevel.INFO, "scope", ["general"]);
  });

  it("reaches a sink installed after the logger was made", () => {
    const logger: LoggerClass = new Logger("early");

    config.install(sink);
    logger.info("late line");

    expect(sink.write).toHaveBeenCalledWith(ELogLevel.INFO, "early", ["late line"]);
  });

  it("leaves a sink installed since in place when an earlier one is released", () => {
    const earlier: ILogSink = { write: jest.fn() };

    config.install(earlier);
    config.install(sink);
    config.release(earlier);

    expect(config.sink).toBe(sink);

    config.release(sink);

    expect(config.sink).toBeNull();
  });

  it("records a line to the sink alone", () => {
    config.install(sink);

    const failure: Error = new Error("uncaught");

    jest.mocked(console.error).mockClear();
    new Logger("scope").record(ELogLevel.ERROR, "Unhandled rejection:", failure);

    expect(sink.write).toHaveBeenCalledWith(ELogLevel.ERROR, "scope", ["Unhandled rejection:", failure]);
    expect(console.error).not.toHaveBeenCalled();
  });

  it("writes nothing from a disabled logger", () => {
    config.install(sink);

    const logger: LoggerClass = new Logger("scope", false);

    logger.error("unseen");
    logger.record(ELogLevel.ERROR, "unseen");

    expect(sink.write).not.toHaveBeenCalled();
  });

  it("writes nothing while logging is off, from the static methods too, and writes again once it is back on", () => {
    config.install(sink);

    const logger: LoggerClass = new Logger("scope");

    jest.mocked(console.error).mockClear();
    config.setEnabled(false);
    logger.error("unseen");
    logger.record(ELogLevel.ERROR, "unseen");
    Logger.error("unseen");

    expect(sink.write).not.toHaveBeenCalled();
    expect(console.error).not.toHaveBeenCalled();

    config.setEnabled(true);
    logger.error("seen");

    expect(sink.write).toHaveBeenCalledWith(ELogLevel.ERROR, "scope", ["seen"]);
  });

  it("writes from a logger made while logging was off once it is on", () => {
    config.install(sink);
    config.setEnabled(false);

    const logger: LoggerClass = new Logger("scope");

    config.setEnabled(true);
    logger.warn("seen");

    expect(sink.write).toHaveBeenCalledWith(ELogLevel.WARN, "scope", ["seen"]);
  });
});
