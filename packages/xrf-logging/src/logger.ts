import { AnyCallable, Nullable } from "@xrf/types";

import { PREFIX_TIMESTAMP_TAG } from "#/logger-prefix";
import { ELogLevel, ILogger, ILogSink } from "#/logger.types";

/**
 * The console's methods as they are when this module loads. A devtools extension replaces them a moment later, and a
 * method taken after that puts the extension's frame between a logger and the console, so devtools links every line
 * to the extension.
 */
const CONSOLE = {
  log: console.log,
  info: console.info,
  warn: console.warn,
  error: console.error,
  debug: console.debug,
} as const;

/**
 * Writes lines to the console under a styled prefix, and hands each to the sink the application installed.
 *
 * Its methods stand between a caller and `console`, so a bundler has to list this module in its source maps' ignore
 * list for devtools to link a line to its caller rather than to this file. A line logged while no devtools is attached
 * keeps this module's frame alone, and links here until the page reloads.
 */
export class Logger implements ILogger {
  public static IS_GLOBAL_LOGGING_ENABLED: boolean = true;

  public static readonly PREFIX_STYLE: string = "color: #bada53; font-weight: 700;";

  /** Where every logger's lines also go; none until the application installs one. */
  public static sink: Nullable<ILogSink> = null;

  protected static global: Logger = new Logger("application");

  /** Logs a general message. */
  public static readonly log: AnyCallable = this.global.log;

  /** Logs an informational message. */
  public static readonly info: AnyCallable = this.global.info;

  /** Logs a warning message. */
  public static readonly warn: AnyCallable = this.global.warn;

  /** Logs an error message. */
  public static readonly error: AnyCallable = this.global.error;

  /** Logs a debug message. */
  public static readonly debug: AnyCallable = this.global.debug;

  public static getLogTagsForPrefix(prefix: string): Array<unknown> {
    return [`%c %s [${prefix}]`, Logger.PREFIX_STYLE, PREFIX_TIMESTAMP_TAG];
  }

  /**
   * @param write - The console's method the line goes to.
   * @param level - The severity the sink is told.
   * @param prefix - The logger's prefix, which is the line's scope.
   * @returns A method writing a line to the console under the prefix, then to the sink.
   */
  private static createMethod(write: AnyCallable, level: ELogLevel, prefix: string): AnyCallable {
    const print: AnyCallable = write.bind(console, ...Logger.getLogTagsForPrefix(prefix));

    return (...args: Array<unknown>): void => {
      print(...args);
      Logger.sink?.write(level, prefix, args);
    };
  }

  public readonly prefix: string;
  public readonly isEnabled: boolean;

  /**
   * Creates a new Logger instance.
   *
   * @param prefix - Text displayed as the console tag.
   * @param isEnabled - Whether this logger accepts messages.
   */
  public constructor(prefix: string, isEnabled: boolean = true) {
    this.prefix = prefix;
    this.isEnabled = isEnabled;

    const isActive: boolean = isEnabled && Logger.IS_GLOBAL_LOGGING_ENABLED;

    if (isActive) {
      this.log = Logger.createMethod(CONSOLE.log, ELogLevel.INFO, prefix);
      this.info = Logger.createMethod(CONSOLE.info, ELogLevel.INFO, prefix);
      this.warn = Logger.createMethod(CONSOLE.warn, ELogLevel.WARN, prefix);
      this.error = Logger.createMethod(CONSOLE.error, ELogLevel.ERROR, prefix);
      this.debug = Logger.createMethod(CONSOLE.debug, ELogLevel.DEBUG, prefix);
    } else {
      this.log = () => {};
      this.info = () => {};
      this.warn = () => {};
      this.error = () => {};
      this.debug = () => {};
    }
  }

  /** Logs a general message, kept as an informational one. */
  public readonly log: AnyCallable;

  /** Logs an informational message. */
  public readonly info: AnyCallable;

  /** Logs a warning message. */
  public readonly warn: AnyCallable;

  /** Logs an error message. */
  public readonly error: AnyCallable;

  /** Logs a debug message. */
  public readonly debug: AnyCallable;
}
