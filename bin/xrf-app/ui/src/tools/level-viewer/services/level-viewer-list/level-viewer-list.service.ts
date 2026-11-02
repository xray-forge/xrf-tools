import { Injectable } from "@wirestate/core";
import { BoundAction, Computed } from "@wirestate/mobx";
import { LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { LevelEntry } from "@xrf/ipc/types/xrf-app-levels";
import { XrayRoots } from "@xrf/ipc/types/xrf-vfs";
import { Logger } from "@xrf/logging";

import { describeRoots } from "@/core/assets/lib";
import { CommandOperation } from "@/core/operations/lib/command-operation";

/**
 * What levels the mounted roots hold, for a picker to choose from.
 */
@Injectable()
export class LevelViewerListService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Every level the roots last listed hold. */
  public readonly levels: CommandOperation<Array<LevelEntry>> = new CommandOperation(this.log);

  /**
   * @returns Levels that ship render geometry, which are the ones there is anything to draw for.
   */
  @Computed()
  public get drawable(): Array<LevelEntry> {
    return (this.levels.value ?? []).filter((entry: LevelEntry) => entry.hasGeometry);
  }

  /** Forgets a listing, which is what roots that are no longer the ones it was made for makes it. */
  @BoundAction()
  public reset(): void {
    this.levels.reset();
  }

  /**
   * Lists the levels of mounted roots, starting from nothing so a failed listing never offers another root's levels.
   *
   * @param roots - Roots to search, loose or archived alike.
   */
  @LatestFlow()
  public *list(roots: XrayRoots): TFlow {
    this.levels.reset();

    yield* this.levels.run({
      name: "List levels",
      subject: describeRoots(roots),
      command: () => levelsCommands.listLevels(roots),
      failure: "Could not list the levels",
    });
  }
}
