import { inject, Injectable } from "@wirestate/core";
import { Maybe, Nullable } from "@xrf/types";

import {
  compareLevelViewerSnapshot,
  ELevelViewerSnapshotStatus,
  ILevelViewerSnapshot,
  ILevelViewerSnapshotEntry,
  LEVEL_VIEWER_SNAPSHOT_CAMERA_DECIMALS,
  LEVEL_VIEWER_SNAPSHOT_FORMAT,
  TLevelViewerSnapshotPart,
} from "@/applications/level-viewer/lib/level-viewer-snapshot";
import { systemCommands } from "@/core/ipc/commands/system";
import { LevelSource } from "@/core/ipc/types/xrf-app";
import { BuildInfo } from "@/core/ipc/types/xrf-build-info";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { isLevelGoTo, toLevelGoTo, toRoundedLevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import {
  ILevelFeatureOptions,
  LEVEL_FEATURE_TOGGLES_ON,
  toLevelRendererFeatures,
} from "@/core/level/lib/features/level-feature-options";
import { ELevelLookSource, ILevelLookChoice, toLevelLookChoice } from "@/core/level/lib/look/level-look-choice";
import { describeLevelSource } from "@/core/level/lib/source/level-source";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { mergeRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { Logger } from "@/lib/logging";
import { flattenDottedRecord, unflattenDottedRecord } from "@/lib/object/dotted-record";

/** What an import applied of the values chosen, by their keys, and those it could not, with why. */
export interface ILevelViewerSnapshotApplied {
  applied: Array<string>;
  failed: Array<{ key: string; reason: string }>;
}

/**
 * A level view as a file holds it: what the viewer draws, from where and under what weather, taken whole and given
 * back in part, a value at a time, into the view's own settings alone.
 */
@Injectable()
export class LevelViewerSnapshotService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

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
  public get level(): Nullable<string> {
    const source: Maybe<LevelSource> = this.loadService.level.value?.selected.value.source;

    return source ? describeLevelSource(source).replaceAll("\\", "/") : null;
  }

  /**
   * @returns The view as it stands, with the build that draws it.
   */
  public async exportSnapshot(): Promise<ILevelViewerSnapshot> {
    const build: Nullable<BuildInfo> = await systemCommands.getBuildInfo().catch((error: unknown) => {
      this.log.warn("Could not read the build for a level view's snapshot:", error);

      return null;
    });

    return this.describe(build);
  }

  /**
   * @param build - The build that draws the view, or null where it is not known.
   * @returns The view as it stands: the camera's place where the viewport has reported it, the weather's cycle where
   *   the weather lights the level, the look's own values where they are edited by hand, and every setting as the view
   *   draws it with every toolbar toggle on, the toggles beside them.
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
      output: {
        resolution: features.output.resolution ?? this.renderSettingsService.renderResolution,
        scale: features.output.scale ?? this.renderSettingsService.renderOutputScale,
      },
      options: { ...this.viewService.options },
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
   * Applies the changes chosen of a snapshot to the view: its settings and output, toolbar and look, then its weather
   * and camera.
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

  private async applyWeather(weather: TLevelViewerSnapshotPart): Promise<ILevelViewerSnapshotApplied["failed"]> {
    const failed: ILevelViewerSnapshotApplied["failed"] = [];

    if (typeof weather.cycle === "string") {
      await this.weatherService.selectCycle(weather.cycle);

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
