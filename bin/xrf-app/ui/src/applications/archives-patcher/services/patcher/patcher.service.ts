import { Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";
import { call, ExclusiveFlow, TFlow } from "@xrf/flow";
import { archivesCommands } from "@xrf/ipc/commands/archives";
import { ARCHIVES_JOB_COMPARE, ARCHIVES_JOB_PATCH, ArchivesPatchRequest } from "@xrf/ipc/types/xrf-app-archives";
import { JobKind } from "@xrf/ipc/types/xrf-app-core";
import { ArchivePatchConfig, ArchivePatchResult } from "@xrf/ipc/types/xrf-pack";
import { Logger } from "@xrf/logging";

import { describePatchOutcome } from "@/applications/archives-patcher/lib/describe-patch-outcome";
import { ArchiveConfigEditorService } from "@/core/archive/services/config-editor";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";

import { toSavedState } from "./patcher.service.utils";

/** Sections of the patching configuration, in the order they are edited. */
export enum EPatcherSection {
  COMPARISON = "comparison",
  OUTPUT = "output",
  SELECTION = "selection",
  HEADER = "header",
  OPTIONS = "options",
}

/**
 * The patching configuration being edited, and what was done with it.
 */
@Injectable()
export class PatcherService extends ArchiveConfigEditorService<ArchivePatchConfig, ArchivePatchResult> {
  /** Which section of the configuration is open. */
  @Observable()
  public section: EPatcherSection = EPatcherSection.COMPARISON;

  public constructor() {
    super(new Logger(__MODULE_NAME__), [ARCHIVES_JOB_COMPARE, ARCHIVES_JOB_PATCH]);
  }

  /**
   * Reads the format's own defaults once, so the form never becomes a second definition of them.
   */
  @OnProvision()
  @ExclusiveFlow("isBusy")
  public *load(): TFlow {
    if (this.config) {
      return;
    }

    this.config = yield* call(archivesCommands.defaultPatchConfig());
  }

  /**
   * Opens one section of the configuration.
   *
   * @param section - Section to show.
   */
  @BoundAction()
  public openSection(section: EPatcherSection): void {
    this.section = section;
  }

  /**
   * Compares two worlds and reports the difference, writing nothing.
   *
   * @param request - What to compare and where the difference would go.
   */
  @ExclusiveFlow("operation")
  public *compare(request: ArchivesPatchRequest): TFlow {
    yield* this.run(ARCHIVES_JOB_COMPARE, request);
  }

  /**
   * Compares two worlds and publishes the difference.
   *
   * @param request - What to compare and where to publish it.
   */
  @ExclusiveFlow("operation")
  public *patch(request: ArchivesPatchRequest): TFlow {
    yield* this.run(ARCHIVES_JOB_PATCH, request);
  }

  private *run(kind: JobKind, request: ArchivesPatchRequest): TFlow {
    const config: ArchivePatchConfig = request.config;

    this.log.info(
      kind === ARCHIVES_JOB_PATCH ? "Patching:" : "Comparing:",
      config.input,
      "against",
      config.target ?? "its own loose gamedata"
    );
    this.configError = null;

    yield* this.operation.run({
      kind,
      invoke: (id: string, progress) =>
        kind === ARCHIVES_JOB_PATCH
          ? archivesCommands.patchArchives(request, id, progress)
          : archivesCommands.compareArchives(request, id, progress),
      describe: (outcome: IJobOutcome<ArchivePatchResult>): IJobNotice => describePatchOutcome(config, outcome),
    });
  }

  protected override toSavedState(config: ArchivePatchConfig): string {
    return toSavedState(config);
  }

  protected override readConfig(path: string, current: ArchivePatchConfig): Promise<ArchivePatchConfig> {
    return archivesCommands.importPatchConfig(path, current);
  }

  protected override writeConfig(path: string, config: ArchivePatchConfig): Promise<unknown> {
    return archivesCommands.exportPatchConfig(path, config);
  }

  protected override listVolumes(config: ArchivePatchConfig): Promise<Array<string>> {
    return archivesCommands.listPatchVolumes(config);
  }
}
