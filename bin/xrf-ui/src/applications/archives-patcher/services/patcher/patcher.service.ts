import { inject, Injectable, OnEvent, OnProvision, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, Observable } from "@wirestate/mobx";

import { describePatchOutcome } from "@/applications/archives-patcher/lib/describe-patch-outcome";
import { toSavedState } from "@/applications/archives-patcher/services/patcher/patcher.service.utils";
import { IResolvedArchiveVolumeSize, resolveArchiveVolumeSize } from "@/core/archive/lib/volume-size";
import { transformError } from "@/core/error/lib";
import { archivesCommands } from "@/core/ipc/commands/archives";
import { ArchivesPatchRequest, EJobKind } from "@/core/ipc/types/xrf-app";
import { ArchivePatchConfig, ArchivePatchResult } from "@/core/ipc/types/xrf-pack";
import { IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { getPathName } from "@/lib/path/separator";
import { Nullable } from "@/lib/types/general";

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
export class PatcherService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  public readonly operation: JobOperation<ArchivePatchResult>;

  /** Which section of the configuration is open. */
  @Observable()
  public section: EPatcherSection = EPatcherSection.COMPARISON;

  @Observable()
  public config: Nullable<ArchivePatchConfig> = null;

  /** Set while a configuration file is being read or written, which is not a job. */
  @Observable()
  public isBusy: boolean = false;

  @Observable()
  private configError: Nullable<string> = null;

  @Computed()
  public get error(): Nullable<string> {
    return this.configError ?? this.operation.error;
  }

  /** Configuration file this was read from or last written to. */
  @Observable()
  public configPath: Nullable<string> = null;

  /** Volumes of the configured set the output already held when it was last looked at. */
  @Observable()
  public publishedVolumes: Array<string> = [];

  /** Volume ceiling as typed, in megabytes, empty while the packer's own maximum applies. */
  @Observable()
  public volumeSize: string = "";

  /** What the configuration looked like when it was last read from or written to a file. */
  @Observable()
  private savedState: Nullable<string> = null;

  public constructor(jobsService: JobsService = inject(JobsService)) {
    this.operation = new JobOperation(jobsService, [EJobKind.ARCHIVES_COMPARE, EJobKind.ARCHIVES_PATCH]);
  }

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
   * @returns The patcher's volume ceiling in megabytes, or zero before defaults arrive.
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

  @Computed()
  public get volumeSizeBytes(): number {
    return this.resolvedVolumeSize.bytes;
  }

  @Computed()
  private get resolvedVolumeSize(): IResolvedArchiveVolumeSize {
    return resolveArchiveVolumeSize(this.volumeSize, this.config?.maxVolumeSize ?? 0);
  }

  /**
   * Records a per-run volume ceiling without changing the configuration's default or validation limit.
   *
   * @param volumeSize - Ceiling in megabytes as typed, which may be empty or not yet a number.
   */
  @BoundAction()
  public setVolumeSize(volumeSize: string): void {
    this.volumeSize = volumeSize;
    this.resetResult();
  }

  @BoundAction()
  public resetResult(): void {
    this.operation.reset();
    this.configError = null;
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
   * Writes fields over the open configuration.
   *
   * @param patch - Fields to change.
   */
  @BoundAction()
  public patchConfig(patch: Partial<ArchivePatchConfig>): void {
    if (this.config) {
      this.config = { ...this.config, ...patch };

      // A result describes the configuration that produced it and stops being true the moment one is changed.
      this.resetResult();
    }
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
   * Asks what the output already holds, so the confirmation can say so.
   *
   * @param config - Configuration whose output and volume name to look at.
   */
  @LatestFlow()
  public *checkDestination(config: ArchivePatchConfig): TFlow {
    try {
      this.publishedVolumes = yield* call(archivesCommands.listPatchVolumes(config));
    } catch (error: unknown) {
      this.log.error("Could not list the output:", error);

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
      const imported: ArchivePatchConfig = yield* call(archivesCommands.importPatchConfig(path, this.config));

      this.log.info("Config imported in:", formatDuration(timer.elapsed()));

      this.config = imported;
      this.configPath = path;
      this.savedState = toSavedState(imported);

      this.operation.reset();
    } catch (error: unknown) {
      this.log.error("Import error after:", formatDuration(timer.elapsed()), error);

      this.configError = transformError(error).message;
    } finally {
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
    const config: Nullable<ArchivePatchConfig> = this.config;

    if (!config || this.operation.isRunning) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Exporting config:", path);

    this.isBusy = true;
    this.configError = null;
    this.operation.clearError();

    try {
      yield* call(archivesCommands.exportPatchConfig(path, config));

      this.log.info("Config exported in:", formatDuration(timer.elapsed()));

      this.configPath = path;
      this.savedState = toSavedState(config);
    } catch (error: unknown) {
      this.log.error("Export error after:", formatDuration(timer.elapsed()), error);

      this.configError = transformError(error).message;
    } finally {
      this.isBusy = false;
    }
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
    if (this.operation.adopt(event.payload)) {
      this.configError = null;
    }
  }
}
