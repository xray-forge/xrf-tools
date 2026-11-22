export interface ILogger {
  log(...args: Array<unknown>): void;
  info(...args: Array<unknown>): void;
  debug(...args: Array<unknown>): void;
  error(...args: Array<unknown>): void;
  warn(...args: Array<unknown>): void;
}

/**
 * How severe a logged line is, most severe first.
 */
export enum ELogLevel {
  ERROR = "error",
  WARN = "warn",
  INFO = "info",
  DEBUG = "debug",
}

/**
 * Where logged lines go besides the console.
 */
export interface ILogSink {
  /**
   * Takes one logged line. Called inside the logging call, so it must be cheap and must not throw or log.
   *
   * @param level - The line's severity.
   * @param scope - The prefix of the logger that wrote it.
   * @param args - The values it was called with, not yet serialized.
   */
  write(level: ELogLevel, scope: string, args: ReadonlyArray<unknown>): void;
}
