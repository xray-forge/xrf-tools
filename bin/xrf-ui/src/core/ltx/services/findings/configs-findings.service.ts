import { inject, Injectable } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { configsCommands } from "@/core/ipc/commands/configs";
import { LtxAnchoredFinding } from "@/core/ipc/types/xrf-ltx-inspect";
import { toOrderedFindings } from "@/core/ltx/lib/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";

/**
 * Everything wrong with one resolved root.
 */
@Injectable()
export class ConfigsFindingsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The entry point whose findings are held, or null when none are. */
  @Observable()
  public entry: Nullable<string> = null;

  /** What verifying that entry point found, in the order a reader walks them. */
  @Observable()
  public findings: AsyncState<Array<LtxAnchoredFinding>> = AsyncState.idle([]);

  public constructor(private readonly projectService: ConfigsProjectService = inject(ConfigsProjectService)) {}

  /**
   * Verify one entry point, replacing whatever was held.
   *
   * @param entry - Engine identity of the entry point to verify.
   */
  @LatestFlow("findings")
  public *open(entry: string): TFlow {
    const sessionId: Nullable<string> = this.projectService.sessionId;

    if (!sessionId) {
      this.log.error("Cannot verify an entry point with no project open:", entry);

      return;
    }

    if (this.entry === entry && this.findings.isReady) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Verifying config entry point:", entry);
    this.entry = entry;
    this.findings = this.findings.asLoading();

    try {
      const found: Array<LtxAnchoredFinding> = yield* call(configsCommands.listFindings({ entry, sessionId }));

      this.log.info("Verified", entry, "with", found.length, "finding(s), in", formatDuration(timer.elapsed()));

      this.findings = this.findings.asReady(toOrderedFindings(found));
    } catch (error) {
      this.log.error("Failed to verify the entry point:", entry, "after", formatDuration(timer.elapsed()), error);

      this.findings = this.findings.asFailed(transformError(error));
    }
  }

  /**
   * Cancel pending verification and forget the findings when their project closes.
   */
  public clear(): void {
    cancelFlow(this, "findings");

    runInAction(() => {
      this.entry = null;
      this.findings = this.findings.asIdle([]);
    });
  }
}
