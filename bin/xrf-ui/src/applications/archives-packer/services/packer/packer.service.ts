import { EventBus, inject, Injectable, OnProvision } from "@wirestate/core";
import { BoundAction, Computed, Observable, runInAction } from "@wirestate/mobx";

import { FALLBACK_PACK_CONFIG } from "@/applications/archives-packer/lib/pack-config";
import { archivesCommands } from "@/core/bindings/commands/archives";
import { ArchivePackConfig, ArchivePackResult } from "@/core/bindings/types/xrf-pack";
import { transformError } from "@/core/error/lib";
import { emitNotification, ENotificationSeverity } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";
import { formatDuration } from "@/lib/format/duration";
import { Logger, Timer } from "@/lib/logging";
import { bytesToWholeMegabytes, megabytesToBytes } from "@/lib/memory/size";
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

  public constructor(private readonly eventBus: EventBus = inject(EventBus)) {}

  /**
   * Opens the editor on the packer's own defaults.
   *
   * A failure falls back to a local copy of them rather than leaving the editor shut: the values are the
   * packer's to own, but not at the cost of an editor that never appears.
   */
  @OnProvision()
  public async onProvision(): Promise<void> {
    try {
      const defaults: ArchivePackConfig = await archivesCommands.defaultPackConfig();

      runInAction(() => (this.config = defaults));
    } catch (error: unknown) {
      this.log.error("Could not read packing defaults:", error);

      runInAction(() => (this.config = FALLBACK_PACK_CONFIG));
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
  @BoundAction()
  public async importConfig(path: string): Promise<void> {
    if (!this.config) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Importing config:", path);

    this.isBusy = true;
    this.error = null;

    try {
      const imported: ArchivePackConfig = await archivesCommands.importPackConfig(path, this.config);

      this.log.info("Config imported in:", formatDuration(timer.elapsed()));

      runInAction(() => {
        this.config = imported;
        this.configPath = path;
        this.savedState = toSavedState(imported);
        this.result = null;
      });
    } catch (error: unknown) {
      this.log.error("Import error after:", formatDuration(timer.elapsed()), error);

      runInAction(() => (this.error = transformError(error).message));
    } finally {
      runInAction(() => (this.isBusy = false));
    }
  }

  /**
   * Writes the open configuration to a file.
   *
   * @param path - Configuration file to write.
   */
  @BoundAction()
  public async exportConfig(path: string): Promise<void> {
    const config: Nullable<ArchivePackConfig> = this.config;

    if (!config) {
      return;
    }

    const timer: Timer = new Timer();

    this.log.info("Exporting config:", path);

    this.isBusy = true;
    this.error = null;

    try {
      await archivesCommands.exportPackConfig(path, config);

      this.log.info("Config exported in:", formatDuration(timer.elapsed()));

      runInAction(() => {
        this.configPath = path;
        this.savedState = toSavedState(config);
      });

      emitNotification(this.eventBus, {
        details: path,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_PACKER,
        title: "Exported packing configuration",
      });
    } catch (error: unknown) {
      this.log.error("Export error after:", formatDuration(timer.elapsed()), error);

      runInAction(() => (this.error = transformError(error).message));
    } finally {
      runInAction(() => (this.isBusy = false));
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
  @BoundAction()
  public async pack(config: ArchivePackConfig): Promise<void> {
    const timer: Timer = new Timer();

    this.log.info("Packing:", config.source);

    this.isBusy = true;
    this.result = null;
    this.error = null;

    try {
      const packed: ArchivePackResult = await archivesCommands.packDirectory(config);

      this.log.info("Packed in:", formatDuration(timer.elapsed()), `(backend ${formatDuration(packed.duration)})`);

      runInAction(() => (this.result = packed));

      emitNotification(this.eventBus, {
        details: `${config.source}\n${config.destination}`,
        severity: ENotificationSeverity.SUCCESS,
        source: EApplicationId.ARCHIVES_PACKER,
        title: "Packed archives",
      });
    } catch (error: unknown) {
      const transformed: Error = transformError(error);

      this.log.error("Pack error after:", formatDuration(timer.elapsed()), transformed);

      runInAction(() => (this.error = transformed.message));

      emitNotification(this.eventBus, {
        details: `${config.source}\n${transformed.message}`,
        severity: ENotificationSeverity.ERROR,
        source: EApplicationId.ARCHIVES_PACKER,
        title: "Could not pack archives",
      });
    } finally {
      runInAction(() => (this.isBusy = false));
    }
  }
}
