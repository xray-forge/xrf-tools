import { isTauri } from "@tauri-apps/api/core";
import { systemCommands } from "@xrf/ipc/commands/system";
import { EUiLogLevel, UiLogLevel, UiLogRecord } from "@xrf/ipc/types/xrf-app-system";
import { ELogLevel, ILogRecord, LogForwarder, Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

/**
 * The page's side of this run's UI log: while installed, every logger's lines are forwarded in batches to the backend,
 * which filters them again and writes them to the UI log file.
 */
export class UiLog {
  /** How long a closing window waits for its log to be written before it closes anyway, in milliseconds. */
  private static readonly CLOSE_TIMEOUT: number = 500;

  /** Each logger level as the backend names it. */
  private static readonly UI_LEVEL: Readonly<Record<ELogLevel, EUiLogLevel>> = {
    [ELogLevel.ERROR]: EUiLogLevel.ERROR,
    [ELogLevel.WARN]: EUiLogLevel.WARN,
    [ELogLevel.INFO]: EUiLogLevel.INFO,
    [ELogLevel.DEBUG]: EUiLogLevel.DEBUG,
  };

  /** Each level the backend names as the logger's own. */
  private static readonly LOGGER_LEVEL: Readonly<Record<UiLogLevel, ELogLevel>> = {
    [EUiLogLevel.ERROR]: ELogLevel.ERROR,
    [EUiLogLevel.WARN]: ELogLevel.WARN,
    [EUiLogLevel.INFO]: ELogLevel.INFO,
    [EUiLogLevel.DEBUG]: ELogLevel.DEBUG,
  };

  /**
   * @param record - A record as the forwarder keeps it.
   * @returns The record as the backend takes it.
   */
  private static toUiLogRecord(record: ILogRecord): UiLogRecord {
    return { level: UiLog.UI_LEVEL[record.level], scope: record.scope, time: record.time, message: record.message };
  }

  /** The forwarder every logger writes into, while this log is installed. */
  private forwarder: Nullable<LogForwarder> = null;

  /**
   * Forwards every logger's lines to the UI log file, asking the backend which levels it keeps.
   *
   * Call it once as the page starts, before anything else logs: lines logged before the backend answers are kept, up
   * to the forwarder's queue bound, and sent once it does, or dropped where it writes no UI log. What is still queued
   * is sent as the page is hidden. Outside a Tauri webview, or while installed already, it does nothing.
   */
  public install(): void {
    if (this.forwarder !== null || !isTauri()) {
      return;
    }

    // The default batch size is the most records `write_ui_log` takes in one call; a larger batch is refused whole.
    const forwarder: LogForwarder = new LogForwarder((records: Array<ILogRecord>) =>
      systemCommands.writeUiLog(records.map(UiLog.toUiLogRecord))
    );

    this.forwarder = forwarder;
    Logger.sink = forwarder;
    window.addEventListener("pagehide", this.onPageHide);

    systemCommands.getUiLogLevel().then(
      (level: Nullable<UiLogLevel>) => forwarder.setLevel(level === null ? null : UiLog.LOGGER_LEVEL[level]),
      (error: unknown) => {
        // The forwarder drops this line itself, having been told there is no level first.
        forwarder.setLevel(null);
        Logger.warn("Cannot read the UI log level, the UI log stays empty:", error);
      }
    );
  }

  /**
   * Stops forwarding, leaving what is still queued unsent.
   */
  public uninstall(): void {
    window.removeEventListener("pagehide", this.onPageHide);

    if (Logger.sink === this.forwarder) {
      Logger.sink = null;
    }

    this.forwarder = null;
  }

  /**
   * Sends what the UI log still holds, as a window closes.
   *
   * @param timeout - Milliseconds to wait at most, so a slow or failing write never holds the close.
   * @returns Settles once everything logged before the call is written or dropped, or the timeout passes; never
   *   rejects.
   */
  public flush(timeout: number = UiLog.CLOSE_TIMEOUT): Promise<void> {
    const forwarder: Nullable<LogForwarder> = this.forwarder;

    if (forwarder === null) {
      return Promise.resolve();
    }

    return new Promise((resolve: () => void) => {
      const timer: ReturnType<typeof setTimeout> = setTimeout(resolve, timeout);

      void forwarder.flush().then(() => {
        clearTimeout(timer);
        resolve();
      });
    });
  }

  /**
   * Sends what is queued as the page is hidden, the last moment a closing or reloading page is sure to run; nothing
   * waits for the send, so it may not finish before the page goes.
   */
  private readonly onPageHide = (): void => {
    void this.forwarder?.flush();
  };
}

/**
 * The page's UI log. One instance rather than a bound service, because it is installed before any container exists:
 * the lines logged while the application starts reach the file too.
 */
export const UI_LOG: UiLog = new UiLog();
