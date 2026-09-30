import { inject, Injectable, OnEvent, OnProvision, WireEvent } from "@wirestate/core";
import { BoundAction, Observable } from "@wirestate/mobx";

import { describePatchOutcome } from "@/applications/archives-patcher/lib/describe-patch-outcome";
import { ArchiveConfigEditorService } from "@/core/archive/services/config-editor";
import { archivesCommands } from "@/core/ipc/commands/archives";
import { ArchivesPatchRequest, EJobKind } from "@/core/ipc/types/xrf-app";
import { ArchivePatchConfig, ArchivePatchResult } from "@/core/ipc/types/xrf-pack";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, TFlow } from "@/lib/mobx";

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
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Which section of the configuration is open. */
  @Observable()
  public section: EPatcherSection = EPatcherSection.COMPARISON;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    super(jobsService, [EJobKind.ARCHIVES_COMPARE, EJobKind.ARCHIVES_PATCH]);
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
    yield* this.run(EJobKind.ARCHIVES_COMPARE, request);
  }

  /**
   * Compares two worlds and publishes the difference.
   *
   * @param request - What to compare and where to publish it.
   */
  @ExclusiveFlow("operation")
  public *patch(request: ArchivesPatchRequest): TFlow {
    yield* this.run(EJobKind.ARCHIVES_PATCH, request);
  }

  private *run(kind: EJobKind.ARCHIVES_COMPARE | EJobKind.ARCHIVES_PATCH, request: ArchivesPatchRequest): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    const config: ArchivePatchConfig = request.config;

    this.log.info(
      kind === EJobKind.ARCHIVES_PATCH ? "Patching:" : "Comparing:",
      config.input,
      "against",
      config.target ?? "its own loose gamedata"
    );
    this.configError = null;

    yield* this.operation.run({
      kind,
      invoke: (id: string, progress) =>
        kind === EJobKind.ARCHIVES_PATCH
          ? archivesCommands.patchArchives(request, id, progress)
          : archivesCommands.compareArchives(request, id, progress),
      describe: (outcome: IJobOutcome<ArchivePatchResult>): IJobNotice => describePatchOutcome(config, outcome),
    });
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.adoptSettled(event.payload);
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
