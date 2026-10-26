import { inject, Injectable } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { transformError } from "@xrf/errors";
import { call, cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { formatDuration } from "@xrf/format";
import { configsCommands } from "@xrf/ipc/commands/configs";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { LtxAnchoredFinding } from "@xrf/ipc/types/xrf-ltx-inspect";
import { Logger, Timer } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { toOrderedFindings } from "@/core/ltx/lib/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";

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
    const sessionId: Nullable<string> = this.projectService.project.sessionId;

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
      const { value: found }: SessionSnapshot<Array<LtxAnchoredFinding>> = yield* call(
        configsCommands.listFindings({ entry, sessionId })
      );

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
