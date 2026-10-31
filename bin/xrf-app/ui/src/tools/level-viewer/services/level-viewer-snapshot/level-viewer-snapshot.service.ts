import { join } from "@tauri-apps/api/path";
import { open, save } from "@tauri-apps/plugin-dialog";
import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, flowResult, RefObservable } from "@wirestate/mobx";
import { ExclusiveFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { systemCommands } from "@xrf/ipc/commands/system";
import { BuildInfo } from "@xrf/ipc/types/xrf-build-info";
import { EXrayExtension } from "@xrf/ipc/types/xrf-extension";
import { Logger } from "@xrf/logging";
import { getPathName } from "@xrf/path";
import { isRecord, Maybe, Nullable } from "@xrf/types";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { isLevelGoTo, toLevelGoTo, toRoundedLevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import {
  ILevelFeatureOptions,
  LEVEL_FEATURE_TOGGLES_ON,
  toLevelOutput,
  toLevelRendererFeatures,
} from "@/core/level/lib/features/level-feature-options";
import { ELevelLookSource, ILevelLookChoice, toLevelLookChoice } from "@/core/level/lib/look/level-look-choice";
import { ILevelHelperOptions, toLevelHelperOptions } from "@/core/level/lib/objects/level-helper-options";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { CommandReporter } from "@/core/operations/lib/command-reporter";
import { mergeRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";
import { flattenDottedRecord, unflattenDottedRecord } from "@/lib/object/dotted-record";
import { toLevelViewerFileStem, toLevelViewerLevelName } from "@/tools/level-viewer/lib/files/level-viewer-file-stem";
import {
  compareLevelViewerSnapshot,
  ELevelViewerSnapshotStatus,
  ILevelViewerImport,
  ILevelViewerSnapshot,
  ILevelViewerSnapshotApplied,
  ILevelViewerSnapshotEntry,
  LEVEL_VIEWER_SNAPSHOT_CAMERA_DECIMALS,
  LEVEL_VIEWER_SNAPSHOT_FORMAT,
  TLevelViewerSnapshotPart,
} from "@/tools/level-viewer/lib/snapshot/level-viewer-snapshot";
import {
  LEVEL_VIEWER_SNAPSHOT_FILTERS,
  readLevelViewerSnapshot,
  writeLevelViewerSnapshot,
} from "@/tools/level-viewer/lib/snapshot/level-viewer-snapshot-files";

/**
 * A level view as a file holds it: what the viewer draws, from where and under what weather, taken whole and given
 * back in part, a value at a time, into the view's own settings alone; File › Export view and Import view, the view
 * written to a file the user picks, in the tool's own folder of views by default, and read back for import.
 */
@Injectable()
export class LevelViewerSnapshotService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The file read for import while its dialog is open. */
  @RefObservable()
  public importing: Nullable<ILevelViewerImport> = null;

  /** What the import applied of the file read, once it has; idle until applied, and again once the dialog closes. */
  public readonly applied: CommandOperation<ILevelViewerSnapshotApplied> = new CommandOperation(this.log);

  /** Reports the exports and the reads for import. */
  private readonly reporter: CommandReporter = new CommandReporter(this.log);

  public constructor(
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly renderService: LevelRenderService = inject(LevelRenderService),
    private readonly weatherService: LevelWeatherService = inject(LevelWeatherService),
    private readonly lookService: LevelLookService = inject(LevelLookService),
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {}

  /** The open level as a snapshot names it, engine paths with forward slashes; null with none open. */
  @Computed()
  public get level(): Nullable<string> {
    return toLevelViewerLevelName(this.loadService.level.value?.selected.value.source);
  }

  /**
   * @returns The view as it stands, with the build that draws it.
   */
  public async exportSnapshot(): Promise<ILevelViewerSnapshot> {
    return this.describe(await this.readBuild());
  }

  /**
   * @returns The build that draws the view, or null where it cannot be read.
   */
  public readBuild(): Promise<Nullable<BuildInfo>> {
    return systemCommands.getBuildInfo().catch((error: unknown) => {
      this.log.warn("Could not read the build for a level view's snapshot:", error);

      return null;
    });
  }

  /** Writes the view as it stands where a save dialog answers, in the folder of views by default. */
  @ExclusiveFlow()
  public *exportView(): TFlow {
    yield* this.reporter.run({
      name: "Export view",
      subject: this.level,
      command: () => this.writeView(),
      failure: "Could not export the view",
    });
  }

  /** Reads the view an open dialog answers, from the folder of views by default, and compares it for import. */
  @ExclusiveFlow()
  public *beginImport(): TFlow {
    const { result } = yield* this.reporter.run({
      name: "Read view",
      command: () => this.readView(),
      failure: "Could not read the view",
    });

    if (result) {
      this.applied.reset();
      this.importing = result;
    }
  }

  /**
   * Applies the entries chosen of the file read for import, once: a call while they are being applied joins it. One
   * that fails closes the dialog, its notice saying why.
   *
   * @param entries - The values chosen.
   */
  @ExclusiveFlow()
  public *applyImport(entries: ReadonlyArray<ILevelViewerSnapshotEntry>): TFlow {
    const importing: Nullable<ILevelViewerImport> = this.importing;

    if (!importing) {
      return;
    }

    const { error } = yield* this.applied.run({
      name: "Import view",
      subject: importing.fileName,
      command: () => this.apply(entries),
      failure: "Could not import the view",
    });

    if (error) {
      this.closeImport();
    }
  }

  @BoundAction()
  public closeImport(): void {
    this.importing = null;
    this.applied.reset();
  }

  /**
   * @param build - The build that draws the view, or null where it is not known.
   * @returns The view as it stands: the camera's place where the viewport has reported it, the weather's cycle where
   *   the weather lights the level, the look's own values where they are edited by hand, the helpers, and every setting
   *   as the view draws it with every toolbar toggle on, the toggles beside them.
   */
  public describe(build: Nullable<BuildInfo>): ILevelViewerSnapshot {
    const camera: Nullable<ILevelCamera> = this.viewportService.camera;
    const weather: LevelWeatherService = this.weatherService;
    const look: ILevelLookChoice = this.lookService.choice;
    const features: ILevelFeatureOptions = this.viewService.features;
    const cycle: Maybe<string> = weather.source === ELevelWeatherSource.WEATHER ? weather.cycle.value?.name : null;

    return {
      format: LEVEL_VIEWER_SNAPSHOT_FORMAT,
      ...(build ? { build } : {}),
      exportedAt: new Date().toISOString(),
      level: this.level,
      camera: {
        ...(camera ? toRoundedLevelGoTo(toLevelGoTo(camera), LEVEL_VIEWER_SNAPSHOT_CAMERA_DECIMALS) : {}),
        fieldOfView: this.viewService.cameraOptions.fieldOfView,
      },
      weather: {
        ...(cycle ? { cycle } : {}),
        time: Math.round(weather.report?.time ?? weather.time),
        isPlaying: !weather.control.isPaused,
      },
      look: { source: look.source, ...(look.source === ELevelLookSource.CUSTOM ? { custom: look.custom } : {}) },
      output: toLevelOutput(
        features.output,
        this.renderSettingsService.renderResolution,
        this.renderSettingsService.renderOutputScale
      ),
      options: { ...this.viewService.options },
      helpers: { ...this.viewService.helpers },
      settings: {
        ...toLevelRendererFeatures(this.renderSettingsService.rendererFeatures, features, LEVEL_FEATURE_TOGGLES_ON),
      },
    };
  }

  /**
   * @param file - A snapshot to import.
   * @returns Every value of it, compared with the view.
   */
  public compare(file: ILevelViewerSnapshot): Array<ILevelViewerSnapshotEntry> {
    return compareLevelViewerSnapshot(file, this.describe(null), {
      exposure: this.renderSettingsService.rendererFeatures.exposure,
      level: this.level,
    });
  }

  /**
   * Applies every change a snapshot makes to the view: what the driver imports.
   *
   * @param file - A snapshot to import.
   * @returns What was applied and what was not, with every value as compared.
   */
  public async importSnapshot(
    file: ILevelViewerSnapshot
  ): Promise<ILevelViewerSnapshotApplied & { entries: Array<ILevelViewerSnapshotEntry> }> {
    const entries: Array<ILevelViewerSnapshotEntry> = this.compare(file);

    return { ...(await this.apply(entries)), entries };
  }

  /**
   * Applies the changes chosen of a snapshot to the view: its settings and output, toolbar, helpers and look, then its
   * weather and camera.
   *
   * @param entries - The values chosen; those not changing the view are passed over.
   * @returns What was applied, and what could not be with why.
   */
  public async apply(entries: ReadonlyArray<ILevelViewerSnapshotEntry>): Promise<ILevelViewerSnapshotApplied> {
    const chosen: Array<ILevelViewerSnapshotEntry> = entries.filter(
      (it: ILevelViewerSnapshotEntry) => it.status === ELevelViewerSnapshotStatus.CHANGE
    );
    const {
      settings,
      output,
      options,
      helpers,
      look,
      weather,
      camera,
    }: Record<string, Maybe<TLevelViewerSnapshotPart>> = unflattenDottedRecord(
      Object.fromEntries(chosen.map((it: ILevelViewerSnapshotEntry) => [it.key, it.applied]))
    ) as Record<string, Maybe<TLevelViewerSnapshotPart>>;
    const features: ILevelFeatureOptions = this.viewService.features;

    if (settings || output) {
      this.viewService.setFeatures({
        ...(mergeRenderFeatureOverrides(
          features as IRenderFeatureOverrides,
          (settings ?? {}) as IRenderFeatureOverrides
        ) as ILevelFeatureOptions),
        output: { ...features.output, ...output },
      });
    }

    if (options) {
      this.viewService.setOptions({ ...this.viewService.options, ...options });
    }

    if (helpers) {
      // The values and kinds chosen over the helpers drawn now.
      const current: ILevelHelperOptions = this.viewService.helpers;

      this.viewService.setHelpers(
        toLevelHelperOptions({
          ...current,
          ...helpers,
          kinds: { ...current.kinds, ...(isRecord(helpers.kinds) ? helpers.kinds : {}) },
        })
      );
    }

    if (typeof look?.source === "string") {
      this.lookService.setSource(look.source as ELevelLookSource);
    }

    if (look?.custom) {
      // The values chosen over the look drawn now, which edits it by hand.
      const custom: ILevelLookChoice["custom"] = toLevelLookChoice(
        {
          custom: unflattenDottedRecord({
            ...flattenDottedRecord(this.lookService.choice.custom ?? this.lookService.look),
            ...flattenDottedRecord(look.custom),
          }),
          source: ELevelLookSource.CUSTOM,
        },
        this.renderSettingsService.rendererFeatures.exposure
      ).custom;

      if (custom) {
        this.lookService.edit(custom);
      }
    }

    const failed: ILevelViewerSnapshotApplied["failed"] = [
      ...(await this.applyWeather(weather ?? {})),
      ...this.applyCamera(camera ?? {}),
    ];
    const result: ILevelViewerSnapshotApplied = {
      applied: chosen
        .map((it: ILevelViewerSnapshotEntry) => it.key)
        .filter((key: string) => !failed.some((it: { key: string }) => it.key === key)),
      failed,
    };

    this.log.info("Imported a level view:", result);

    return result;
  }

  /**
   * Writes the view as it stands where a save dialog answers, in the folder of views by default.
   *
   * @returns Where it was written, or null where the dialog was dismissed.
   */
  private async writeView(): Promise<Nullable<string>> {
    const snapshot: ILevelViewerSnapshot = await this.exportSnapshot();
    const folder: string = await levelsCommands.getViewFolder();
    const defaultPath: string = await join(
      folder,
      `${toLevelViewerFileStem(this.level, new Date())}.${EXrayExtension.JSON}`
    );
    const path: Nullable<string> = await save({
      defaultPath,
      filters: LEVEL_VIEWER_SNAPSHOT_FILTERS,
      title: "Export view",
    });

    return path ? writeLevelViewerSnapshot(path, snapshot) : null;
  }

  /**
   * Reads the view an open dialog answers, from the folder of views by default, compared with the view.
   *
   * @returns The file for import, or null where the dialog was dismissed.
   */
  private async readView(): Promise<Nullable<ILevelViewerImport>> {
    const defaultPath: string = await levelsCommands.getViewFolder();
    const path: Nullable<string> = (await open({
      defaultPath,
      directory: false,
      filters: LEVEL_VIEWER_SNAPSHOT_FILTERS,
      multiple: false,
      title: "Import view",
    })) as Nullable<string>;

    if (!path) {
      return null;
    }

    const snapshot: ILevelViewerSnapshot = await readLevelViewerSnapshot(path);

    return { entries: this.compare(snapshot), fileName: getPathName(path), snapshot };
  }

  private async applyWeather(weather: TLevelViewerSnapshotPart): Promise<ILevelViewerSnapshotApplied["failed"]> {
    const failed: ILevelViewerSnapshotApplied["failed"] = [];

    if (typeof weather.cycle === "string") {
      await flowResult(this.weatherService.selectCycle(weather.cycle));

      if (this.weatherService.cycle.value?.name !== weather.cycle) {
        failed.push({
          key: "weather.cycle",
          reason: this.weatherService.failure ?? `the level does not play the cycle ${weather.cycle}`,
        });
      }
    }

    if (typeof weather.time === "number") {
      this.renderService.seekWeather(weather.time);
    }

    if (typeof weather.isPlaying === "boolean") {
      this.weatherService.setPlaying(weather.isPlaying);
    }

    return failed;
  }

  /** The lens, then the place over where the camera stands now, which a file may give only in part. */
  private applyCamera({ fieldOfView, ...place }: TLevelViewerSnapshotPart): ILevelViewerSnapshotApplied["failed"] {
    if (typeof fieldOfView === "number") {
      this.viewService.setCameraOptions({ ...this.viewService.cameraOptions, fieldOfView });
    }

    if (!Object.keys(place).length) {
      return [];
    }

    const current: Nullable<ILevelCamera> = this.viewportService.camera;
    const goTo: TLevelViewerSnapshotPart = { ...(current ? toLevelGoTo(current) : {}), ...place };

    if (isLevelGoTo(goTo)) {
      this.renderService.goTo(goTo);

      return [];
    }

    return Object.keys(place).map((key: string) => ({
      key: `camera.${key}`,
      reason: "the file places the camera only in part, and the viewport has not said where it stands",
    }));
  }
}
