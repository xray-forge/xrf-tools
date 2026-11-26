import { AnyCallable } from "@xrf/types";

import { LOGGER_CONFIG } from "#/logger-config";
import { PREFIX_TIMESTAMP_TAG } from "#/logger-prefix";
import { ELogLevel, ILogger } from "#/logger.types";

/**
 * The console's methods as they are when this module loads. A devtools hook, such as React DevTools', replaces them a
 * moment later, and a method taken after that puts the hook's frame between a logger and the console, so devtools links
 * every line to the hook.
 */
const CONSOLE = {
  log: console.log,
  info: console.info,
  warn: console.warn,
  error: console.error,
  debug: console.debug,
} as const;

/** Every method of a disabled logger. */
function noop(): void {}

/**
 * Writes lines to the console under a styled prefix, and hands each to the sink the application installed. Whether
 * anything is written, and where else it goes, is read from {@link LOGGER_CONFIG} at each line.
 *
 * Each method wraps the console's, so devtools links a line to its caller only where the page's source maps list this
 * package as ignored.
 */
export class Logger implements ILogger {
  public static readonly PREFIX_STYLE: string = "color: #bada53; font-weight: 700;";

  /** The logger behind the static methods, made while the class initialises: the prefix style must be declared above. */
  protected static global: Logger = new Logger("application");

  /** Logs a general message, which the sink takes as an informational one. */
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

  public readonly prefix: string;
  public readonly isEnabled: boolean;

  /**
   * Creates a logger; a disabled one writes nothing, and any writes only while {@link LOGGER_CONFIG} has logging on.
   *
   * @param prefix - Text displayed as the console tag, and the scope the sink is told.
   * @param isEnabled - Whether this logger accepts messages.
   */
  public constructor(prefix: string, isEnabled: boolean = true) {
    this.prefix = prefix;
    this.isEnabled = isEnabled;
    this.log = this.createMethod("log", ELogLevel.INFO);
    this.info = this.createMethod("info", ELogLevel.INFO);
    this.warn = this.createMethod("warn", ELogLevel.WARN);
    this.error = this.createMethod("error", ELogLevel.ERROR);
    this.debug = this.createMethod("debug", ELogLevel.DEBUG);
  }

  /** Logs a general message, which the sink takes as an informational one. */
  public readonly log: AnyCallable;

  /** Logs an informational message. */
  public readonly info: AnyCallable;

  /** Logs a warning message. */
  public readonly warn: AnyCallable;

  /** Logs an error message. */
  public readonly error: AnyCallable;

  /** Logs a debug message. */
  public readonly debug: AnyCallable;

  /**
   * Hands a line to the sink alone, for a line the console already shows, such as an uncaught failure the browser
   * printed itself.
   *
   * @param level - The line's severity.
   * @param args - The values of the line.
   */
  public record(level: ELogLevel, ...args: Array<unknown>): void {
    if (this.isEnabled && LOGGER_CONFIG.isEnabled) {
      LOGGER_CONFIG.sink?.write(level, this.prefix, args);
    }
  }

  /**
   * Makes one of this logger's methods: while logging is on, it prints a line through the console's `method` under the
   * prefix, then hands it to the sink at `level`. A disabled logger gets the shared no-op.
   *
   * @param method - The console's method the line goes to.
   * @param level - The severity the sink is told.
   */
  private createMethod(method: keyof typeof CONSOLE, level: ELogLevel): AnyCallable {
    if (!this.isEnabled) {
      return noop;
    }

    const prefix: string = this.prefix;
    const print: AnyCallable = CONSOLE[method].bind(console, ...Logger.getLogTagsForPrefix(prefix));

    return (...args: Array<unknown>): void => {
      if (!LOGGER_CONFIG.isEnabled) {
        return;
      }

      print(...args);
      LOGGER_CONFIG.sink?.write(level, prefix, args);
    };
  }
}
