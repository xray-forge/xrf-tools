import { Nullable } from "@xrf/types";

import { ILogSink } from "#/logger.types";

/**
 * What every logger of the page reads at each line: whether loggers write at all, and the sink their lines also go to.
 * One instance, {@link LOGGER_CONFIG}, owns both, so the application installs its sink and a test setup silences
 * logging through it rather than by assigning to `Logger`.
 */
export class LoggerConfig {
  private currentSink: Nullable<ILogSink> = null;
  private isOn: boolean = true;

  /** Where every logger's lines also go; none until the application installs one. */
  public get sink(): Nullable<ILogSink> {
    return this.currentSink;
  }

  /** Whether loggers write anything, to the console or the sink. */
  public get isEnabled(): boolean {
    return this.isOn;
  }

  /**
   * Sends every logger's lines to `sink` too, in place of any sink installed before.
   *
   * @param sink - Where lines also go.
   */
  public install(sink: ILogSink): void {
    this.currentSink = sink;
  }

  /**
   * Stops sending lines to `sink`, leaving a sink installed since in place.
   *
   * @param sink - The sink its owner installed.
   */
  public release(sink: ILogSink): void {
    if (this.currentSink === sink) {
      this.currentSink = null;
    }
  }

  /**
   * Switches every logger, those already made included, on or off from the next line.
   *
   * @param isEnabled - Whether loggers write anything.
   */
  public setEnabled(isEnabled: boolean): void {
    this.isOn = isEnabled;
  }

  /** Returns to how the page starts: logging on, no sink. */
  public reset(): void {
    this.currentSink = null;
    this.isOn = true;
  }
}

/** The page's logger configuration. */
export const LOGGER_CONFIG: LoggerConfig = new LoggerConfig();
