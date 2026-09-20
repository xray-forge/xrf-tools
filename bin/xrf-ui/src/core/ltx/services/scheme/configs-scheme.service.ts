import { inject, Injectable } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";

import { transformError } from "@/core/error/lib";
import { configsCommands } from "@/core/ipc/commands/configs";
import { LtxSectionSchemeReport } from "@/core/ipc/types/xrf-ltx-inspect";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { AsyncState } from "@/lib/async-state";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, cancelFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * What judges the selected section, and how the section measures against it.
 */
@Injectable()
export class ConfigsSchemeService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The entry point the held report was read from, or null when none is. */
  @Observable()
  public entry: Nullable<string> = null;

  /** The section the held report is about, or null when none is. */
  @Observable()
  public section: Nullable<string> = null;

  /**
   * What the selected section is judged by.
   */
  @Observable()
  public report: AsyncState<LtxSectionSchemeReport> = AsyncState.idle();

  public constructor(private readonly projectService: ConfigsProjectService = inject(ConfigsProjectService)) {}

  /**
   * Read what one section of one root is judged by.
   *
   * @param entry - Engine identity of the entry point the section belongs to.
   * @param section - The section to explain, as the index named it.
   */
  @LatestFlow("report")
  public *read(entry: string, section: string): TFlow {
    const sessionId: Nullable<string> = this.projectService.sessionId;

    if (!sessionId) {
      this.log.error("Cannot explain a section with no project open:", section);

      return;
    }

    // The same section of the same root answers the same rows: a panel closed and opened again should cost nothing.
    if (this.entry === entry && this.section === section && this.report.isReady) {
      return;
    }

    const timer: Timer = new Timer();

    this.entry = entry;
    this.section = section;
    this.report = this.report.asLoading();

    try {
      const report: Nullable<LtxSectionSchemeReport> = yield* call(
        configsCommands.readSectionScheme({ entry, section, sessionId })
      );

      this.report = this.report.asReady(report);

      this.log.info(
        "Section scheme read:",
        entry,
        section,
        report ? "found" : "absent",
        "in",
        formatDuration(timer.elapsed())
      );
    } catch (error) {
      this.log.error("Failed to explain the section:", entry, section, "after", formatDuration(timer.elapsed()), error);

      this.report = this.report.asFailed(transformError(error));
    }
  }

  /**
   * Cancel the pending read and forget the report when its selection or project goes.
   */
  public clear(): void {
    cancelFlow(this, "report");

    runInAction(() => {
      this.entry = null;
      this.section = null;
      this.report = this.report.asIdle(null);
    });
  }
}
