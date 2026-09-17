import { EventBus, inject, Injectable, OnEvent, OnProvision, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { describePackOutcome } from "@/applications/archives-packer/lib/describe-pack-outcome";
import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { IResolvedArchiveVolumeSize, resolveArchiveVolumeSize } from "@/core/archive/lib/volume-size";
import { transformError } from "@/core/error/lib";
import { archivesCommands } from "@/core/ipc/commands/archives";
import { EJobKind } from "@/core/ipc/types/xrf-app";
import { ArchivePackConfig, ArchivePackResult } from "@/core/ipc/types/xrf-pack";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { getPathName } from "@/lib/path/separator";
import { Nullable } from "@/lib/types/general";

/** Sections of the packing configuration, in the order they are edited. */
export enum EPackerSection {
  OUTPUT = "output",
  SELECTION = "selection",
  HEADER = "header",
  OPTIONS = "options",
}

/**
 * What a configuration file actually carries, as one comparable value.
 *
 * Paths and volume name are deliberately left out: they are chosen per run and never written to a
 * configuration, so counting them as edits would mark a freshly imported file as unsaved.
 */
function toSavedState(config: ArchivePackConfig): string {
  return JSON.stringify([
    config.includeDirectories,
    config.includeFiles,
    config.excludeDirectories,
    config.excludeExtensions,
    config.header,
  ]);
}

/**
 * The packing configuration being edited, and what was done with it.
 *
 * A service rather than editor state because the shell draws the section navigation outside this
 * application's tree, and because import, export and the dirty rule are worth testing without a
 * rendered editor around them.
 */
@Injectable()
export class PackerService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<ArchivePackResult>;

  /** Which section of the configuration is open. */
  @Observable()
  public section: EPackerSection = EPackerSection.OUTPUT;

  @Observable()
  public config: Nullable<ArchivePackConfig> = null;

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  private configError: Nullable<string> = null;

  /**
   * @returns The latest configuration I/O failure, or the packing failure when configuration I/O has not failed.
   */
  @Computed()
  public get error(): Nullable<string> {
    return this.configError ?? this.operation.error;
  }

  /** Configuration file this was read from or last written to. */
  @Observable()
  public configPath: Nullable<string> = null;

  /**
   * Volumes of the configured set the destination already held when it was last looked at.
   *
   * What the confirmation shows before a pack that would replace an archive the user still has. A view of the
   * filesystem at one moment rather than a guarantee: packing refuses such a destination on its own.
   */
  @Observable()
  public publishedVolumes: Array<string> = [];

  /** Volume ceiling as typed, in megabytes, empty while the packer's own maximum applies. */
  @Observable()
  public volumeSize: string = "";

  /** What the configuration looked like when it was last read from or written to a file. */
  @Observable()
  private savedState: Nullable<string> = null;

  /**
   * @returns Whether there are edits no configuration file holds.
   */
  @Computed()
  public get isDirty(): boolean {
    return Boolean(this.savedState && this.config && this.savedState !== toSavedState(this.config));
  }

  /**
   * @returns The file name of the open configuration, or null when nothing was imported or exported.
   */
  @Computed()
  public get configName(): Nullable<string> {
    return this.configPath ? getPathName(this.configPath) : null;
  }

  /**
   * @returns The packer's volume ceiling in megabytes, or zero before defaults arrive.
   */
  @Computed()
  public get maxVolumeSizeMegabytes(): number {
    return this.resolvedVolumeSize.maxMegabytes;
  }

  /**
   * @returns What is wrong with the typed volume size, or null when it is usable or empty.
   */
  @Computed()
  public get volumeSizeError(): Nullable<string> {
    return this.resolvedVolumeSize.error;
  }

  /**
   * @returns The typed ceiling in bytes when it is usable, and the packer's own otherwise.
   */
  @Computed()
  public get volumeSizeBytes(): number {
    return this.resolvedVolumeSize.bytes;
  }

  @Computed()
  private get resolvedVolumeSize(): IResolvedArchiveVolumeSize {
    return resolveArchiveVolumeSize(this.volumeSize, this.config?.maxVolumeSize ?? 0);
  }

  public constructor(
    private readonly eventBus: EventBus = inject(EventBus),
    public readonly jobsService: JobsService = inject(JobsService)
  ) {
    this.operation = new JobOperation(jobsService, [EJobKind.ARCHIVES_PACK]);
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

  @BoundAction()
  public setVolumeSize(volumeSize: string): void {
    this.volumeSize = volumeSize;
    this.resetResult();
  }

  /**
   * Clears the displayed outcome and configuration error when a packing input changes.
   */
  @BoundAction()
  public resetResult(): void {
    this.operation.reset();
    this.configError = null;
  }

  /**
   * Applies an edit to the configuration.
   *
   * Clears the last outcome along with it, because a result describes the configuration that produced it
   * and stops being true the moment one is changed.
   *
   * @param patch - Fields to change on the open configuration.
   */
  @BoundAction()
  public patchConfig(patch: Partial<ArchivePackConfig>): void {
    if (!this.config) {
      return;
    }

    this.config = { ...this.config, ...patch };
    this.resetResult();
  }

  /**
   * Looks at what the destination already holds under the configured set name.
   *
   * Its own lane rather than the form's `isBusy`: this runs while the confirmation opens, and disabling the editor
   * for a directory listing would read as the pack having already started. A failure is logged and read as nothing
   * found, because packing refuses such a destination itself and a listing that did not answer is no reason to keep
   * the user from confirming.
   *
   * @param config - Configuration with the source and destination filled in.
   */
  @LatestFlow()
  public *checkDestination(config: ArchivePackConfig): TFlow {
    try {
      this.publishedVolumes = yield* call(archivesCommands.listPackVolumes(config));
    } catch (error: unknown) {
      this.log.error("Could not list the destination:", error);

      this.publishedVolumes = [];
    }
  }

  /**
   * Reads a configuration file over the open configuration.
   *
   * @param path - Configuration file to read.
   */
  @ExclusiveFlow("isBusy")
  public *importConfig(path: string): TFlow {
    if (!this.config || this.operation.isRunning) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Importing config:", path);

    this.isBusy = true;
    this.configError = null;
    this.operation.clearError();

    try {
      const imported: ArchivePackConfig = yield* call(archivesCommands.importPackConfig(path, this.config));

      this.log.info("Config imported in:", formatDuration(timer.elapsed()));

      this.config = imported;
      this.configPath = path;
      this.savedState = toSavedState(imported);
      this.operation.reset();
    } catch (error: unknown) {
      this.log.error("Import error after:", formatDuration(timer.elapsed()), error);

      this.configError = transformError(error).message;
    } finally {
      // Deactivation also releases the form's local busy state.
      this.isBusy = false;
    }
  }

  /**
   * Writes the open configuration to a file.
   *
   * @param path - Configuration file to write.
   */
  @ExclusiveFlow("isBusy")
  public *exportConfig(path: string): TFlow {
    const config: Nullable<ArchivePackConfig> = this.config;

    if (!config || this.operation.isRunning) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Exporting config:", path);

    this.isBusy = true;
    this.configError = null;
    this.operation.clearError();

    try {
      yield* call(archivesCommands.exportPackConfig(path, config));

      this.log.info("Config exported in:", formatDuration(timer.elapsed()));

      this.configPath = path;
      this.savedState = toSavedState(config);

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_PACKER,
        title: "Exported packing configuration",
      });
    } catch (error: unknown) {
      this.log.error("Export error after:", formatDuration(timer.elapsed()), error);

      this.configError = transformError(error).message;
    } finally {
      this.isBusy = false;
    }
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
    if (this.operation.adopt(event.payload)) {
      this.configError = null;
    }
  }
}
