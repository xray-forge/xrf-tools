import { inject, Injectable } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";

import { configsCommands } from "@/core/bindings/commands/configs";
import { LtxAnchoredFinding } from "@/core/bindings/types/xrf-ltx-inspect";
import { transformError } from "@/core/error/lib";
import { toOrderedFindings } from "@/core/ltx/lib/findings";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

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
  public findings: Loadable<Array<LtxAnchoredFinding>> = Loadable.idle([]);

  public constructor(private readonly projectService: ConfigsProjectService = inject(ConfigsProjectService)) {}

  /**
   * Verify one entry point, replacing whatever was held.
   *
   * Superseding: a different entry point is a different root, and the findings of the one being replaced describe a
   * document nobody is looking at.
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

    this.entry = entry;
    this.findings = this.findings.asLoading();

    try {
      const found: Array<LtxAnchoredFinding> = yield* call(configsCommands.listFindings({ entry, sessionId }));

      this.log.info("Verified", entry, "with", found.length, "finding(s)");

      this.findings = this.findings.asReady(toOrderedFindings(found));
    } catch (error) {
      this.log.error("Failed to verify the entry point:", entry, error);

      this.findings = this.findings.asFailed(transformError(error));
    }
  }

  /**
   * Forget what was found, when the project behind it closes.
   */
  public clear(): void {
    runInAction(() => {
      this.entry = null;
      this.findings = this.findings.asIdle([]);
    });
  }
}
