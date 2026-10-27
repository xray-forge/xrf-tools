import { EventBus, inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, flowResult, Observable } from "@wirestate/mobx";
import { call, ExclusiveFlow, TFlow } from "@xrf/flow";
import { archivesCommands } from "@xrf/ipc/commands/archives";
import { ARCHIVES_JOB_PACK } from "@xrf/ipc/types/xrf-app-archives";
import { ArchivePackConfig, ArchivePackResult } from "@xrf/ipc/types/xrf-pack";
import { Logger } from "@xrf/logging";

import { describePackOutcome } from "@/applications/archives-packer/lib/describe-pack-outcome";
import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { ArchiveConfigEditorService } from "@/core/archive/services/config-editor";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";

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
  /** Which section of the configuration is open. */
  @Observable()
  public section: EPackerSection = EPackerSection.OUTPUT;

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {
    super(new Logger(__MODULE_NAME__), [ARCHIVES_JOB_PACK]);
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
    this.log.info("Packing:", config.source);

    this.isBusy = true;
    this.configError = null;

    try {
      yield* this.operation.run({
        kind: ARCHIVES_JOB_PACK,
        invoke: (id: string, progress) => archivesCommands.packDirectory({ config, isForced }, id, progress),
        describe: (outcome: IJobOutcome<ArchivePackResult>): IJobNotice => describePackOutcome(config, outcome),
      });
    } finally {
      // Releasing this view unlocks the form; the job continues through JobsService.
      this.isBusy = false;
    }
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
