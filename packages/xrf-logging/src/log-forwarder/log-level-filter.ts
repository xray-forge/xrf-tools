import { Nullable, Optional } from "@xrf/types";

import { LOG_LEVEL_RANK } from "#/log-level-rank";
import { ELogLevel } from "#/logger.types";

/**
 * Which levels are kept: every level until the filter is told one, then that level and the more severe ones, or none.
 */
export class LogLevelFilter {
  /** The most verbose level kept: undefined until told, null once told nothing is kept. */
  private level: Optional<Nullable<ELogLevel>> = undefined;

  /** Whether the filter has been told which levels are kept. */
  public get isKnown(): boolean {
    return this.level !== undefined;
  }

  /**
   * Keeps a level and the more severe ones from now on.
   *
   * @param level - The most verbose level kept, or null to keep nothing.
   */
  public set(level: Nullable<ELogLevel>): void {
    this.level = level;
  }

  /**
   * @param level - A line's level.
   * @returns Whether a line at that level is kept.
   */
  public passes(level: ELogLevel): boolean {
    if (this.level === undefined) {
      return true;
    }

    return this.level !== null && LOG_LEVEL_RANK[level] <= LOG_LEVEL_RANK[this.level];
  }
}
