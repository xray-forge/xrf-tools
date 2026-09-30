import { BoundAction, Computed, Observable } from "@wirestate/mobx";
import { Nullable, Optional } from "@xrf/types";

import { IResolvedArchiveVolumeSize, resolveArchiveVolumeSize } from "@/core/archive/lib/volume-size";
import { transformError } from "@/core/error/lib";
import { EJobKind } from "@/core/ipc/types/xrf-app";
import { IJobSettledPayload } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";
import { getPathName } from "@/lib/path/separator";

/** What every archive configuration carries that this editor reads: the packer's own volume ceiling. */
export interface IArchiveEditedConfig {
  maxVolumeSize: number;
}

/**
 * An archive configuration being edited - packing or patching - and the file it was read from or written to.
 *
 * A base class rather than a composed part, because configuration I/O and the run share one exclusive `isBusy` lane
 * on the service itself: a composed editor would hold a lane of its own and let an import land over a run.
 */
export abstract class ArchiveConfigEditorService<TConfig extends IArchiveEditedConfig, TResult> {
  public abstract readonly log: Logger;

  public readonly operation: JobOperation<TResult>;

  @Observable()
  public config: Nullable<TConfig> = null;

  /** Set while a configuration file is being read or written, or while this window starts a run. */
  @Observable()
  public isBusy: boolean = false;

  @Observable()
  protected configError: Nullable<string> = null;

  /** Configuration file this was read from or last written to. */
  @Observable()
  public configPath: Nullable<string> = null;

  /**
   * Volumes of the configured set the destination already held when it was last looked at.
   *
   * What the confirmation shows before a run that would replace an archive the user still has. A view of the
   * filesystem at one moment rather than a guarantee: the run refuses such a destination on its own.
   */
  @Observable()
  public publishedVolumes: Array<string> = [];

  /** Volume ceiling as typed, in megabytes, empty while the packer's own maximum applies. */
  @Observable()
  public volumeSize: string = "";

  /** What the configuration looked like when it was last read from or written to a file. */
  @Observable()
  private savedState: Nullable<string> = null;

  protected constructor(jobsService: JobsService, kinds: ReadonlyArray<EJobKind>) {
    this.operation = new JobOperation(jobsService, kinds);
  }

  /**
   * @returns The latest configuration I/O failure, or the run's failure when configuration I/O has not failed.
   */
  @Computed()
  public get error(): Nullable<string> {
    return this.configError ?? this.operation.error;
  }

  /**
   * @returns Whether there are edits no configuration file holds.
   */
  @Computed()
  public get isDirty(): boolean {
    return Boolean(this.savedState && this.config && this.savedState !== this.toSavedState(this.config));
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

  /**
   * Clears the displayed outcome and configuration error when an input changes.
   */
  @BoundAction()
  public resetResult(): void {
    this.operation.reset();
    this.configError = null;
  }

  /**
   * Writes fields over the open configuration.
   *
   * @param patch - Fields to change.
   */
  @BoundAction()
  public patchConfig(patch: Partial<TConfig>): void {
    if (this.config) {
      this.config = { ...this.config, ...patch };

      // A result describes the configuration that produced it and stops being true the moment one is changed.
      this.resetResult();
    }
  }

  /**
   * Looks at what the destination already holds under the configured set name.
   *
   * Its own lane rather than the form's `isBusy`: this runs while the confirmation opens, and disabling the editor
   * for a directory listing would read as the run having already started. A failure is logged and read as nothing
   * found, because the run refuses such a destination itself and a listing that did not answer is no reason to keep
   * the user from confirming.
   *
   * @param config - Configuration with the destination filled in.
   */
  @LatestFlow()
  public *checkDestination(config: TConfig): TFlow {
    try {
      this.publishedVolumes = yield* call(this.listVolumes(config));
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
      const imported: TConfig = yield* call(this.readConfig(path, this.config));

      this.log.info("Config imported in:", formatDuration(timer.elapsed()));

      this.config = imported;
      this.configPath = path;
      this.savedState = this.toSavedState(imported);
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
    const config: Nullable<TConfig> = this.config;

    if (!config || this.operation.isRunning) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Exporting config:", path);

    this.isBusy = true;
    this.configError = null;
    this.operation.clearError();

    try {
      yield* call(this.writeConfig(path, config));

      this.log.info("Config exported in:", formatDuration(timer.elapsed()));

      this.configPath = path;
      this.savedState = this.toSavedState(config);
      this.onExported(path);
    } catch (error: unknown) {
      this.log.error("Export error after:", formatDuration(timer.elapsed()), error);

      this.configError = transformError(error).message;
    } finally {
      this.isBusy = false;
    }
  }

  /**
   * Shows what a run this window watched rather than started has answered: the reload case, where the command replied
   * to a page that is gone and what the backend retained arrives instead.
   *
   * @param settled - Settled job announced by the jobs service.
   */
  protected adoptSettled(settled: Optional<IJobSettledPayload>): void {
    if (this.operation.adopt(settled)) {
      this.configError = null;
    }
  }

  /** Told once a configuration file was written, for a subclass that says so. */
  protected onExported(_path: string): void {}

  /** What the configuration holds that a file does, as one comparable value. */
  protected abstract toSavedState(config: TConfig): string;

  /** Reads a configuration file over what is open. */
  protected abstract readConfig(path: string, current: TConfig): Promise<TConfig>;

  /** Writes a configuration file. */
  protected abstract writeConfig(path: string, config: TConfig): Promise<unknown>;

  /** The volumes the destination already holds under the configured set name. */
  protected abstract listVolumes(config: TConfig): Promise<Array<string>>;
}
