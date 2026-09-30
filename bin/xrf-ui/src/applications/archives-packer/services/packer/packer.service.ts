import { EventBus, inject, Injectable, OnEvent, OnProvision, WireEvent } from "@wirestate/core";
import { BoundAction, flowResult, Observable } from "@wirestate/mobx";

import { describePackOutcome } from "@/applications/archives-packer/lib/describe-pack-outcome";
import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { ArchiveConfigEditorService } from "@/core/archive/services/config-editor";
import { archivesCommands } from "@/core/ipc/commands/archives";
import { EJobKind } from "@/core/ipc/types/xrf-app";
import { ArchivePackConfig, ArchivePackResult } from "@/core/ipc/types/xrf-pack";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, TFlow } from "@/lib/mobx";

import { toSavedState } from "./packer.service.utils";

/** Sections of the packing configuration, in the order they are edited. */
export enum EPackerSection {
  OUTPUT = "output",
  SELECTION = "selection",
  HEADER = "header",
  OPTIONS = "options",
}

/**
 * The packing configuration being edited, and what was done with it.
 *
 * A service rather than editor state because the shell draws the section navigation outside this
 * application's tree, and because import, export and the dirty rule are worth testing without a
 * rendered editor around them.
 */
@Injectable()
export class PackerService extends ArchiveConfigEditorService<ArchivePackConfig, ArchivePackResult> {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Which section of the configuration is open. */
  @Observable()
  public section: EPackerSection = EPackerSection.OUTPUT;

  public constructor(
    private readonly eventBus: EventBus = inject(EventBus),
    public readonly jobsService: JobsService = inject(JobsService)
  ) {
    super(jobsService, [EJobKind.ARCHIVES_PACK]);
  }

  /**
   * Opens the editor on the packer's own defaults.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  /**
   * Reads the packing defaults the backend reports.
   *
   * Defaults, configuration I/O and packing share an exclusive lane so they cannot overwrite each other's state.
   */
  @ExclusiveFlow("isBusy")
  private *restore(): TFlow {
    try {
      this.config = yield* call(archivesCommands.defaultPackConfig());
    } catch (error: unknown) {
      this.log.error("Could not read packing defaults:", error);

      this.config = FALLBACK_PACK_CONFIG;
    }
  }

  @BoundAction()
  public setSection(section: EPackerSection): void {
    this.section = section;
  }

  /**
   * Packs a resolved configuration.
   *
   * Takes the configuration to pack rather than reading its own, because the paths come from the
   * editor's pickers and never belong to the configuration itself.
   *
   * @param config - Configuration with the source and destination filled in.
   * @param isForced - Whether the user agreed to replace volumes the destination already holds.
   */
  @ExclusiveFlow("isBusy")
  public *pack(config: ArchivePackConfig, isForced: boolean): TFlow {
    if (this.operation.isRunning) {
      return;
    }

    this.log.info("Packing:", config.source);

    this.isBusy = true;
    this.configError = null;

    try {
      yield* this.operation.run({
        kind: EJobKind.ARCHIVES_PACK,
        invoke: (id: string, progress) => archivesCommands.packDirectory({ config, isForced }, id, progress),
        describe: (outcome: IJobOutcome<ArchivePackResult>): IJobNotice => describePackOutcome(config, outcome),
      });
    } finally {
      // Releasing this view unlocks the form; the job continues through JobsService.
      this.isBusy = false;
    }
  }

  /**
   * Shows what a pack this window watched rather than started has answered.
   *
   * The reload case: the command replied to a page that is gone, so nothing here ever awaited this run. What the
   * backend retained arrives instead, and the editor renders it exactly as if this service had packed it.
   *
   * @param event - Settled job announced by the jobs service.
   */
  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.adoptSettled(event.payload);
  }

  protected override onExported(path: string): void {
    emitNotification(this.eventBus, {
      details: path,
      severity: ENotificationSeverity.SUCCESS,
      source: EApplicationId.ARCHIVES_PACKER,
      title: "Exported packing configuration",
    });
  }

  protected override toSavedState(config: ArchivePackConfig): string {
    return toSavedState(config);
  }

  protected override readConfig(path: string, current: ArchivePackConfig): Promise<ArchivePackConfig> {
    return archivesCommands.importPackConfig(path, current);
  }

  protected override writeConfig(path: string, config: ArchivePackConfig): Promise<unknown> {
    return archivesCommands.exportPackConfig(path, config);
  }

  protected override listVolumes(config: ArchivePackConfig): Promise<Array<string>> {
    return archivesCommands.listPackVolumes(config);
  }
}
