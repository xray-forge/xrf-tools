import { EventBus, inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, flowResult, Observable } from "@wirestate/mobx";

import { describePackOutcome } from "@/applications/archives-packer/lib/describe-pack-outcome";
import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { ArchivePackConfig, ArchivePackResult } from "@/core/bindings/types/xrf-pack";
import { transformError } from "@/core/error/lib";
import { IJobOutcome, IJobRun, IJobState } from "@/core/jobs/lib";
import { JobsService } from "@/core/jobs/services/jobs";
import { emitNotification, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { bytesToWholeMegabytes, megabytesToBytes } from "@/lib/memory/size";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { Nullable } from "@/lib/types/general";

/**
 * What a pack calls itself in the jobs registry.
 */
export const ARCHIVES_PACK_JOB_KIND: string = "archives.pack";

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

  /** Which section of the configuration is open. */
  @Observable()
  public section: EPackerSection = EPackerSection.OUTPUT;

  @Observable()
  public config: Nullable<ArchivePackConfig> = null;

  @Observable()
  public isBusy: boolean = false;

  @Observable()
  public error: Nullable<string> = null;

  @Observable()
  public result: Nullable<ArchivePackResult> = null;

  /** Configuration file this was read from or last written to. */
  @Observable()
  public configPath: Nullable<string> = null;

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
    return this.configPath ? (this.configPath.split(/[\\/]/).pop() ?? this.configPath) : null;
  }

  /**
   * @returns The packer's volume ceiling in megabytes, or zero before defaults arrive.
   */
  @Computed()
  public get maxVolumeSizeMegabytes(): number {
    return this.config ? bytesToWholeMegabytes(this.config.maxVolumeSize) : 0;
  }

  /**
   * @returns What is wrong with the typed volume size, or null when it is usable or empty.
   */
  @Computed()
  public get volumeSizeError(): Nullable<string> {
    const value: number = Number(this.volumeSize);

    if (!this.volumeSize.trim()) {
      return null;
    }

    return !Number.isInteger(value) || value < 1 || value > this.maxVolumeSizeMegabytes
      ? `Enter a whole number between 1 and ${this.maxVolumeSizeMegabytes}`
      : null;
  }

  /**
   * @returns The typed ceiling in bytes when it is usable, and the packer's own otherwise.
   */
  @Computed()
  public get volumeSizeBytes(): number {
    if (!this.config) {
      return 0;
    }

    return this.volumeSize.trim() && !this.volumeSizeError
      ? megabytesToBytes(Number(this.volumeSize))
      : this.config.maxVolumeSize;
  }

  /** The pack this service started, while it runs. Null once it settles, and after a reload until re-attach. */
  @Observable()
  public jobId: Nullable<string> = null;

  /**
   * @returns The pack currently running, whether this service started it or found it again.
   */
  @Computed()
  public get job(): Nullable<IJobState> {
    return this.jobId ? this.jobsService.getJob(this.jobId) : this.jobsService.getJobOfKind(ARCHIVES_PACK_JOB_KIND);
  }

  public constructor(
    private readonly eventBus: EventBus = inject(EventBus),
    private readonly jobsService: JobsService = inject(JobsService)
  ) {}

  /**
   * Stops the running pack, if there is one.
   */
  @BoundAction()
  public cancel(): void {
    if (this.job) {
      this.jobsService.cancel(this.job.id);
    }
  }

  /**
   * Opens the editor on the packer's own defaults.
   *
   * A failure falls back to a local copy of them rather than leaving the editor shut: the values are the
   * packer's to own, but not at the cost of an editor that never appears.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    await flowResult(this.restore());
  }

  /**
   * Reads the packing defaults the backend reports.
   *
   * Exclusive rather than latest: an operation the user started owns the packer, and a defaults read that has not
   * finished must join it rather than land on top. Every user operation takes the lane the other way round, which
   * is the same mutual exclusion `isBusy` reports to the form.
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
    this.result = null;
    this.error = null;
  }

  /**
   * Reads a configuration file over the open configuration.
   *
   * @param path - Configuration file to read.
   */
  @LatestFlow("isBusy")
  public *importConfig(path: string): TFlow {
    if (!this.config) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Importing config:", path);

    this.isBusy = true;
    this.error = null;

    try {
      const imported: ArchivePackConfig = yield* call(archivesCommands.importPackConfig(path, this.config));

      this.log.info("Config imported in:", formatDuration(timer.elapsed()));

      this.config = imported;
      this.configPath = path;
      this.savedState = toSavedState(imported);
      this.result = null;
    } catch (error: unknown) {
      this.log.error("Import error after:", formatDuration(timer.elapsed()), error);

      this.error = transformError(error).message;
    } finally {
      // Reached on cancellation too, so a superseded import does not leave the form disabled.
      this.isBusy = false;
    }
  }

  /**
   * Writes the open configuration to a file.
   *
   * @param path - Configuration file to write.
   */
  @LatestFlow("isBusy")
  public *exportConfig(path: string): TFlow {
    const config: Nullable<ArchivePackConfig> = this.config;

    if (!config) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Exporting config:", path);

    this.isBusy = true;
    this.error = null;

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

      this.error = transformError(error).message;
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
   */
  @LatestFlow("isBusy")
  public *pack(config: ArchivePackConfig): TFlow {
    const timer: Timer = new Timer();

    this.log.info("Packing:", config.source);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    // Started through the jobs service rather than invoked here, so the run has an identity the cancel control can
    // reach and survives this view being torn down. What it answers is still this service's to render.
    const run: IJobRun<ArchivePackResult> = this.jobsService.run<ArchivePackResult>({
      kind: ARCHIVES_PACK_JOB_KIND,
      source: EApplicationId.ARCHIVES_PACKER,
      invoke: (id: string, progress) => archivesCommands.packDirectory(config, id, progress),
      describe: (outcome: IJobOutcome<ArchivePackResult>): INotificationPayload => describePackOutcome(config, outcome),
    });

    this.jobId = run.id;

    try {
      const packed: ArchivePackResult = yield* call(run.promise);

      this.log.info("Packed in:", formatDuration(timer.elapsed()), `(backend ${formatDuration(packed.duration)})`);

      this.result = packed;
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Pack error after:", formatDuration(timer.elapsed()), transformed);

      this.error = transformed.message;
    } finally {
      // Reached on cancellation of this generator too, which is a superseded view rather than a stopped job: the run
      // itself keeps going and reports through the jobs service.
      this.isBusy = false;
      this.jobId = null;
    }
  }
}
