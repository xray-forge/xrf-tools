import { Injectable } from "@wirestate/core";
import { Computed, Observable } from "@wirestate/mobx";

import { describeRoots } from "@/core/assets/lib";
import { transformError } from "@/core/error/lib";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { LevelEntry } from "@/core/ipc/types/xrf-app";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";

/**
 * What levels the mounted roots hold, for a picker to choose from.
 */
@Injectable()
export class LevelListService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  @Observable()
  public levels: AsyncState<Array<LevelEntry>> = AsyncState.idle();

  /**
   * @returns Levels that ship render geometry, which are the ones there is anything to draw for.
   */
  @Computed()
  public get drawable(): Array<LevelEntry> {
    return (this.levels.value ?? []).filter((entry: LevelEntry) => entry.hasGeometry);
  }

  /**
   * Lists the levels of mounted roots.
   *
   * @param roots - Roots to search, loose or archived alike.
   */
  @LatestFlow("levels")
  public *list(roots: XrayRoots): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Listing levels:", describeRoots(roots));

    try {
      this.levels = this.levels.asLoading();

      const entries: Array<LevelEntry> = yield* call(levelsCommands.listLevels(roots));

      this.log.info(
        "Listed levels:",
        describeRoots(roots),
        entries.length,
        "levels, in",
        formatDuration(timer.elapsed())
      );

      this.levels = this.levels.asReady(entries);
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error(
        "Failed to list levels:",
        describeRoots(roots),
        "after",
        formatDuration(timer.elapsed()),
        transformed
      );

      this.levels = this.levels.asFailed(transformed);
    }
  }
}
