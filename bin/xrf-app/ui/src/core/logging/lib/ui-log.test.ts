import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { ESystemCommand } from "@xrf/ipc/commands/system";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands, mockIsTauri } from "@xrf/ipc/testing";
import { EUiLogLevel, UiLogRecord } from "@xrf/ipc/types/xrf-app-system";
import { ELogLevel, Logger } from "@xrf/logging";

import { UiLog } from "@/core/logging/lib/ui-log";
import { flush } from "@/fixtures/utils/promise";

describe("UiLog", () => {
  let log: UiLog = new UiLog();

  beforeEach(() => {
    mockInvoke.mockClear();
    log = new UiLog();
  });

  afterEach(() => {
    log.uninstall();
    jest.useRealTimers();
  });

  it("installs nothing outside a Tauri webview", () => {
    mockIsTauri.mockReturnValue(false);

    log.install();

    expect(Logger.sink).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("writes each record through the UI log command at the level the backend names", async () => {
    mockIpcCommands({ [ESystemCommand.GET_UI_LOG_LEVEL]: EUiLogLevel.INFO });

    log.install();
    await flush();

    Logger.sink?.write(ELogLevel.DEBUG, "scope", ["detail"]);
    Logger.sink?.write(ELogLevel.WARN, "scope", ["slow", 3]);
    await flush();

    expect(listMockInvokeCalls(ESystemCommand.WRITE_UI_LOG)).toEqual([
      { records: [{ level: EUiLogLevel.WARN, scope: "scope", time: expect.any(Number), message: "slow 3" }] },
    ]);
  });

  // The two level tables are inverse by hand: each level the backend names keeps that level and those above it, and
  // writes each kept record at the level it was logged at.
  it.each(Object.values(EUiLogLevel))("keeps and writes back each level up to %s", async (level: EUiLogLevel) => {
    const spellings: Array<string> = Object.values(ELogLevel);

    mockIpcCommands({ [ESystemCommand.GET_UI_LOG_LEVEL]: level });

    log.install();
    await flush();

    for (const it of Object.values(ELogLevel)) {
      Logger.sink?.write(it, "scope", [it]);
    }

    await log.flush();

    const written: Array<string> = listMockInvokeCalls(ESystemCommand.WRITE_UI_LOG).flatMap(
      (call: { records: Array<UiLogRecord> }) => call.records.map((record: UiLogRecord) => record.level)
    );

    expect(written).toEqual(spellings.slice(0, spellings.indexOf(level) + 1));
  });

  it("writes nothing when the level cannot be read", async () => {
    mockIpcCommands({
      [ESystemCommand.GET_UI_LOG_LEVEL]: () => {
        throw new Error("unavailable");
      },
    });

    log.install();
    Logger.sink?.write(ELogLevel.ERROR, "scope", ["failed"]);
    await flush();
    await log.flush();

    expect(listMockInvokeCalls(ESystemCommand.WRITE_UI_LOG)).toEqual([]);
  });

  it("sends what is queued as the page is hidden, without waiting for the interval", async () => {
    mockIpcCommands({ [ESystemCommand.GET_UI_LOG_LEVEL]: EUiLogLevel.DEBUG });

    log.install();
    await flush();

    Logger.sink?.write(ELogLevel.INFO, "scope", ["leaving"]);
    window.dispatchEvent(new Event("pagehide"));
    await flush();

    expect(listMockInvokeCalls(ESystemCommand.WRITE_UI_LOG)).toHaveLength(1);
  });

  it("hands the loggers back once uninstalled", async () => {
    mockIpcCommands({ [ESystemCommand.GET_UI_LOG_LEVEL]: EUiLogLevel.DEBUG });

    log.install();
    await flush();
    log.uninstall();

    expect(Logger.sink).toBeNull();
  });

  it("stops waiting for a write that does not settle once the timeout passes", async () => {
    jest.useFakeTimers();
    mockIpcCommands({
      [ESystemCommand.GET_UI_LOG_LEVEL]: EUiLogLevel.DEBUG,
      [ESystemCommand.WRITE_UI_LOG]: () => new Promise<null>(() => {}),
    });

    log.install();
    await jest.advanceTimersByTimeAsync(0);

    Logger.sink?.write(ELogLevel.INFO, "scope", ["closing"]);

    const done: jest.Mock<() => void> = jest.fn();

    void log.flush(500).then(done);
    await jest.advanceTimersByTimeAsync(499);
    expect(done).not.toHaveBeenCalled();

    await jest.advanceTimersByTimeAsync(1);
    expect(done).toHaveBeenCalledTimes(1);
  });

  it("settles a flush at once when nothing is installed", async () => {
    await expect(log.flush()).resolves.toBeUndefined();
  });
});
