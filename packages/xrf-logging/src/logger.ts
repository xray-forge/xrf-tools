import { AnyCallable, Nullable } from "@xrf/types";

import { PREFIX_TIMESTAMP_TAG } from "#/logger-prefix";
import { ELogLevel, ILogger, ILogSink } from "#/logger.types";

/**
 * Writes lines to the console under a styled prefix, and hands each to the sink the application installed.
 *
 * Each method wraps the console's, so devtools links a line to its caller only where the page's source maps list this
 * package as ignored.
 */
export class Logger implements ILogger {
  /**
   * The console's methods as they are when this module loads. A devtools hook, such as React DevTools', replaces them a
   * moment later, and a method taken after that puts the hook's frame between a logger and the console, so devtools
   * links every line to the hook.
   */
  private static readonly CONSOLE = {
    log: console.log,
    info: console.info,
    warn: console.warn,
    error: console.error,
    debug: console.debug,
  } as const;

  /** Every method of a logger that writes nothing. */
  private static readonly NOOP: AnyCallable = (): void => {};

  /** Whether loggers made from now on write anything; a logger keeps what it read when it was made. */
  public static IS_GLOBAL_LOGGING_ENABLED: boolean = true;

  public static readonly PREFIX_STYLE: string = "color: #bada53; font-weight: 700;";

  /**
   * Where every logger's lines also go; none until the application installs one. Read at each line, so a logger made
   * before the sink was installed hands it lines too.
   */
  public static sink: Nullable<ILogSink> = null;

  /**
   * The logger behind the static methods. It is made while the class initialises, so the statics it reads as it is
   * made (the console's methods, the no-op, the switch and the prefix style) must be declared above it.
   */
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

  /** Whether this logger writes anything: enabled itself, and made while logging was on. */
  private readonly isActive: boolean;

  /**
   * Creates a logger that writes nothing where it is disabled or global logging was off when it was made.
   *
   * @param prefix - Text displayed as the console tag, and the scope the sink is told.
   * @param isEnabled - Whether this logger accepts messages.
   */
  public constructor(prefix: string, isEnabled: boolean = true) {
    this.prefix = prefix;
    this.isEnabled = isEnabled;
    this.isActive = isEnabled && Logger.IS_GLOBAL_LOGGING_ENABLED;
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
    if (this.isActive) {
      Logger.sink?.write(level, this.prefix, args);
    }
  }

  /**
   * Makes one of this logger's methods: it prints a line through the console's `method` under the prefix, then hands
   * it to the sink at `level`. A logger that writes nothing gets the shared no-op.
   *
   * @param method - The console's method the line goes to.
   * @param level - The severity the sink is told.
   */
  private createMethod(method: keyof typeof Logger.CONSOLE, level: ELogLevel): AnyCallable {
    if (!this.isActive) {
      return Logger.NOOP;
    }

    const prefix: string = this.prefix;
    const print: AnyCallable = Logger.CONSOLE[method].bind(console, ...Logger.getLogTagsForPrefix(prefix));

    return (...args: Array<unknown>): void => {
      print(...args);
      Logger.sink?.write(level, prefix, args);
    };
  }
}
