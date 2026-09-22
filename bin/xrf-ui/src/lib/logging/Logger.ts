import { AnyCallable } from "@xrf/types";

import { noop } from "@/lib/callbacks/noop";
import { ILogger } from "@/lib/logging/ILogger";
import { PREFIX_TIMESTAMP_TAG } from "@/lib/logging/prefix";

/**
 * A lightweight wrapper around `console` methods that adds a styled prefix.
 * Logging is automatically disabled in the production environment.
 */
export class Logger implements ILogger {
  public static IS_GLOBAL_LOGGING_ENABLED: boolean = true;

  public static readonly PREFIX_STYLE: string = "color: #bada53; font-weight: 700;";

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
      const tags: Array<unknown> = Logger.getLogTagsForPrefix(prefix);

      this.log = console.log.bind(console, ...tags);
      this.info = console.info.bind(console, ...tags);
      this.warn = console.warn.bind(console, ...tags);
      this.error = console.error.bind(console, ...tags);
      this.debug = console.debug.bind(console, ...tags);
    } else {
      this.log = noop;
      this.info = noop;
      this.warn = noop;
      this.error = noop;
      this.debug = noop;
    }
  }

  /** Logs a general message. */
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
