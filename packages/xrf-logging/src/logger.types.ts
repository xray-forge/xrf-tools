export interface ILogger {
  log(...args: Array<unknown>): void;
  info(...args: Array<unknown>): void;
  debug(...args: Array<unknown>): void;
  error(...args: Array<unknown>): void;
  warn(...args: Array<unknown>): void;
}
