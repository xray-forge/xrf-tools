import { inject, Injectable } from "@wirestate/core";
import { Observable, runInAction } from "@wirestate/mobx";

import { configsCommands } from "@/core/bindings/commands/configs";
import { LtxSectionSchemeReport } from "@/core/bindings/types/xrf-ltx-inspect";
import { transformError } from "@/core/error/lib";
import { ConfigsProjectService } from "@/core/ltx/services/project";
import { Loadable } from "@/lib/loadable";
import { Logger } from "@/lib/logging";
import { call, LatestFlow, TFlow } from "@/lib/mobx";
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
   *
   * A ready `null` is an answer rather than an absence: the root no longer holds the section, which a selection made
   * before a reopen reaches.
   */
  @Observable()
  public report: Loadable<Nullable<LtxSectionSchemeReport>> = Loadable.idle(null);

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

    this.entry = entry;
    this.section = section;
    this.report = this.report.asLoading();

    try {
      const report: Nullable<LtxSectionSchemeReport> = yield* call(
        configsCommands.readSectionScheme({ entry, section, sessionId })
      );

      this.report = this.report.asReady(report);
    } catch (error) {
      this.log.error("Failed to explain the section:", section, error);

      this.report = this.report.asFailed(transformError(error));
    }
  }

  /**
   * Forget the held report, when the selection or the project behind it goes.
   */
  public clear(): void {
    runInAction(() => {
      this.entry = null;
      this.section = null;
      this.report = this.report.asIdle(null);
    });
  }
}
