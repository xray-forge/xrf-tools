import { Channel } from "@tauri-apps/api/core";
import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { call, ExclusiveFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { JobKind } from "@xrf/ipc/types/xrf-app-core";
import {
  LevelMapDescription,
  LEVELS_JOB_CAPTURE_CUBE,
  LEVELS_JOB_CAPTURE_MAP,
  SelectedLevelDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { CaptureOutcome } from "@xrf/ipc/types/xrf-app-render";
import { JobProgress } from "@xrf/ipc/types/xrf-job";
import { Logger } from "@xrf/logging";
import { assertExhaustive, Nullable } from "@xrf/types";

import {
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
} from "@/applications/level-viewer/commands";
import { describeLevelCaptureOutcome } from "@/applications/level-viewer/lib/capture/describe-level-capture-outcome";
import { ELevelCaptureMode, LEVEL_CAPTURE_NAMES } from "@/applications/level-viewer/lib/capture/level-capture";
import { TLevelCapture } from "@/applications/level-viewer/lib/capture/level-captured";
import {
  ILevelCubeParameters,
  isLevelCubeCapturable,
  toDefaultLevelCubeParameters,
  toLevelCubeSupersampling,
  toRenderCubeCapture,
} from "@/applications/level-viewer/lib/cube/level-cube-capture";
import {
  toLevelViewerFileStem,
  toLevelViewerLevelName,
} from "@/applications/level-viewer/lib/files/level-viewer-file-stem";
import { ILevelViewerImage, readLevelViewerImage } from "@/applications/level-viewer/lib/files/level-viewer-image";
import {
  ELevelMapArea,
  ELevelMapLook,
  ILevelMapParameters,
  ILevelMapRect,
  ILevelMapVolume,
  isLevelMapCapturable,
  toDefaultLevelMapParameters,
  toLevelMapAreaRect,
  toLevelMapRect,
  toLevelMapSize,
  toLevelMapSupersampling,
  toLevelMapViewRect,
  toLevelMapVolume,
  toRenderMapCapture,
} from "@/applications/level-viewer/lib/map/level-map-capture";
import { LevelViewerCaptureResultService } from "@/applications/level-viewer/services/capture-result";
import { KeybindCommand } from "@/core/commands";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LEVEL_WEATHER_NOON } from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherHold } from "@/core/level/lib/weather/level-weather-hold";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { TCommandCompletion } from "@/core/operations/lib/command-reporter";

/** The weather cycle `CDemoRecord` captures a level map under. */
const MAP_WEATHER_CYCLE: string = "map";

/** What each mode's file in the screenshots folder is named first. */
const CAPTURE_PREFIXES: Record<ELevelCaptureMode, string> = {
  [ELevelCaptureMode.MAP]: "map",
  [ELevelCaptureMode.CUBEMAP]: "cube",
};

/** The job each mode's capture runs as. */
const CAPTURE_JOB_KINDS: Record<ELevelCaptureMode, JobKind> = {
  [ELevelCaptureMode.MAP]: LEVELS_JOB_CAPTURE_MAP,
  [ELevelCaptureMode.CUBEMAP]: LEVELS_JOB_CAPTURE_CUBE,
};

/** What one capture takes: a map's parameters or a cubemap's. */
type TLevelCaptureRequest =
  | { mode: ELevelCaptureMode.MAP; parameters: ILevelMapParameters }
  | { mode: ELevelCaptureMode.CUBEMAP; parameters: ILevelCubeParameters };

/**
 * @param request - What was captured.
 * @param image - The picture it wrote.
 * @returns The capture as the dialog shows it.
 */
function toLevelCapture(request: TLevelCaptureRequest, image: ILevelViewerImage): TLevelCapture {
  switch (request.mode) {
    case ELevelCaptureMode.MAP:
      return { mode: ELevelCaptureMode.MAP, image, area: request.parameters.rect };
    case ELevelCaptureMode.CUBEMAP:
      return { mode: ELevelCaptureMode.CUBEMAP, image };
    default:
      return assertExhaustive(request);
  }
}

/**
 * The level viewer's captures of a level map or a cubemap: the dialog's parameters, and the capture run as a job under
 * the look asked for with the view put back after. What a capture wrote is `LevelViewerCaptureResultService`'s.
 */
@Injectable()
export class LevelViewerCaptureService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The capture's job: its progress, and stopping it. */
  public readonly operation: JobOperation<CaptureOutcome> = new JobOperation(this.log, {
    kinds: [LEVELS_JOB_CAPTURE_MAP, LEVELS_JOB_CAPTURE_CUBE],
  });

  @RefObservable()
  public isOpen: boolean = false;

  /** What the dialog captures. */
  @RefObservable()
  public mode: ELevelCaptureMode = ELevelCaptureMode.MAP;

  /** What the next map and cubemap take, set when the dialog opens. */
  @RefObservable()
  public mapParameters: Nullable<ILevelMapParameters> = null;

  @RefObservable()
  public cubeParameters: ILevelCubeParameters = toDefaultLevelCubeParameters();

  /** Why the last capture failed, until another is asked for or the dialog opens again. */
  @RefObservable()
  public failure: Nullable<string> = null;

  /** Whether a capture has the view, from its look applied until the view is put back as it was. */
  @RefObservable()
  private isViewHeld: boolean = false;

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly renderService: LevelRenderService = inject(LevelRenderService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly weatherService: LevelWeatherService = inject(LevelWeatherService),
    private readonly resultService: LevelViewerCaptureResultService = inject(LevelViewerCaptureResultService)
  ) {}

  /** The level open, or null for none. */
  @Computed()
  public get level(): Nullable<SelectedLevelDescription> {
    return this.loadService.level.value?.selected.value ?? null;
  }

  /** Where the open level's `level.ltx` places its map, or null where it places none or it is not read yet. */
  @Computed()
  public get levelMap(): Nullable<LevelMapDescription> {
    return this.loadService.levelMap.value;
  }

  /** Where the open level's `level.ltx` lays its map, or null where it lays none. */
  @Computed()
  public get levelMapRect(): Nullable<ILevelMapRect> {
    return toLevelMapRect(this.levelMap);
  }

  /** The box the open level's sectors cover, engine space, or null for none. */
  @Computed()
  public get volume(): Nullable<ILevelMapVolume> {
    return toLevelMapVolume(this.level?.bounds ?? null);
  }

  /**
   * Whether a capture runs, from its look applied until the view is put back, or a capture job found running: the
   * viewport draws its shots rather than its own frames meanwhile.
   */
  @Computed()
  public get isCapturing(): boolean {
    return this.isViewHeld || this.operation.isRunning;
  }

  /** Whether the dialog can open on a cubemap: a level is open and no capture runs. */
  @Computed()
  public get canOpen(): boolean {
    return this.level !== null && !this.isCapturing;
  }

  /** Whether the dialog can open on a level map: it can open, and the level's sectors cover a box to map. */
  @Computed()
  public get canOpenMap(): boolean {
    return this.canOpen && this.volume !== null;
  }

  /** Whether the mode's parameters ask for something that can be captured, and no capture runs. */
  @Computed()
  public get canCapture(): boolean {
    return !this.isCapturing && this.request !== null;
  }

  /** What the mode's parameters ask to capture, or null where they ask for nothing that can be. */
  @Computed()
  private get request(): Nullable<TLevelCaptureRequest> {
    switch (this.mode) {
      case ELevelCaptureMode.MAP: {
        const parameters: Nullable<ILevelMapParameters> = this.mapParameters;

        return parameters && isLevelMapCapturable(parameters) ? { mode: ELevelCaptureMode.MAP, parameters } : null;
      }
      case ELevelCaptureMode.CUBEMAP: {
        const parameters: ILevelCubeParameters = this.cubeParameters;

        return isLevelCubeCapturable(parameters) ? { mode: ELevelCaptureMode.CUBEMAP, parameters } : null;
      }
      default:
        return assertExhaustive(this.mode);
    }
  }

  /** The map's pixels across and down the parameters ask for, or null with none. */
  @Computed()
  public get mapSize(): Nullable<[number, number]> {
    return this.mapParameters ? toLevelMapSize(this.mapParameters.rect, this.mapParameters.longSide) : null;
  }

  /** Opens the dialog on a level map and the defaults: the level's map where it lays one, its whole box otherwise. */
  @KeybindCommand(CAPTURE_LEVEL_MAP_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewerCaptureService) => service.canOpenMap,
  })
  public open(): void {
    this.openOn(ELevelCaptureMode.MAP);
  }

  /** Opens the dialog on a cubemap and the defaults. */
  @KeybindCommand(CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND, {
    isEnabled: (service: LevelViewerCaptureService) => service.canOpen,
  })
  public openCubemap(): void {
    this.openOn(ELevelCaptureMode.CUBEMAP);
  }

  /** Closes the dialog, letting what it showed go; nothing while a capture or an export runs. */
  @BoundAction()
  public close(): void {
    if (!this.isCapturing && !this.resultService.isExporting) {
      this.isOpen = false;
      this.resultService.show(null);
    }
  }

  /**
   * @param mode - What the dialog captures from now on.
   */
  @BoundAction()
  public setMode(mode: ELevelCaptureMode): void {
    if (!this.isCapturing) {
      this.mode = mode;
    }
  }

  /**
   * @param patch - What of the map's parameters changes; a new size takes its default supersampling unless it says
   *   one.
   */
  @BoundAction()
  public setMapParameters(patch: Partial<ILevelMapParameters>): void {
    if (!this.mapParameters) {
      return;
    }

    const supersampling: number =
      patch.supersampling ??
      (patch.longSide === undefined ? this.mapParameters.supersampling : toLevelMapSupersampling(patch.longSide));

    this.mapParameters = { ...this.mapParameters, ...patch, supersampling };
  }

  /**
   * @param patch - What of the cubemap's parameters changes; a new size takes its default supersampling unless it
   *   says one.
   */
  @BoundAction()
  public setCubeParameters(patch: Partial<ILevelCubeParameters>): void {
    const supersampling: number =
      patch.supersampling ??
      (patch.size === undefined ? this.cubeParameters.supersampling : toLevelCubeSupersampling(patch.size));

    this.cubeParameters = { ...this.cubeParameters, ...patch, supersampling };
  }

  /**
   * @param area - Where the map's area comes from from now on.
   */
  @BoundAction()
  public setArea(area: ELevelMapArea): void {
    const volume: Nullable<ILevelMapVolume> = this.volume;

    if (this.mapParameters && volume) {
      this.mapParameters = {
        ...this.mapParameters,
        area,
        rect: toLevelMapAreaRect(area, this.levelMapRect, volume, this.mapParameters.rect),
      };
    }
  }

  /** Takes a custom area from the ground about the camera. */
  @BoundAction()
  public takeAreaFromView(): void {
    const camera: Nullable<ILevelCamera> = this.viewportService.camera;

    if (this.mapParameters && camera) {
      this.mapParameters = {
        ...this.mapParameters,
        area: ELevelMapArea.CUSTOM,
        rect: toLevelMapViewRect(camera.position, this.mapParameters.bottom),
      };
    }
  }

  /** Cuts the level away above the camera's height. */
  @BoundAction()
  public takeTopFromCamera(): void {
    const camera: Nullable<ILevelCamera> = this.viewportService.camera;

    if (this.mapParameters && camera) {
      this.mapParameters = {
        ...this.mapParameters,
        top: Math.max(camera.position.y, this.mapParameters.bottom + 1),
      };
    }
  }

  /** Puts the map's heights back to the level's whole box. */
  @BoundAction()
  public resetHeights(): void {
    const volume: Nullable<ILevelMapVolume> = this.volume;

    if (this.mapParameters && volume) {
      this.mapParameters = { ...this.mapParameters, bottom: volume.bottom, top: volume.top };
    }
  }

  /**
   * Captures what the mode's parameters ask for into the screenshots folder, then reads it back for the dialog: the
   * weather held meanwhile, a map under the look it asks for, and the view put back after, however it ends.
   */
  @ExclusiveFlow()
  public *capture(): TFlow {
    const request: Nullable<TLevelCaptureRequest> = this.isCapturing ? null : this.request;

    if (!request) {
      return;
    }

    const { mode } = request;
    const name: string = LEVEL_CAPTURE_NAMES[mode];
    const options: ILevelViewOptions = this.viewService.options;
    const weather: ILevelWeatherHold = this.weatherService.hold();
    const stem: string = toLevelViewerFileStem(toLevelViewerLevelName(this.level?.source), new Date());

    this.isViewHeld = true;
    this.failure = null;

    try {
      const path: string = yield* call(levelsCommands.findScreenshotPath(`${CAPTURE_PREFIXES[mode]}_${stem}`));

      if (request.mode === ELevelCaptureMode.MAP) {
        yield* call(this.applyMapLook(request.parameters, options));
      }

      this.log.info(`Capturing a ${name}:`, path);

      const completion: Nullable<TCommandCompletion<CaptureOutcome>> = yield* this.operation.run({
        kind: CAPTURE_JOB_KINDS[mode],
        invoke: async (id, progress): Promise<CaptureOutcome> => {
          const outcome: Nullable<CaptureOutcome> = await this.captureShots(request, path, [id, progress]);

          if (!outcome) {
            throw new Error(`The level viewer has no viewport attached to capture a ${name} in`);
          }

          return outcome;
        },
        describe: (outcome: IJobOutcome<CaptureOutcome>): IJobNotice =>
          describeLevelCaptureOutcome(mode, path, outcome),
      });

      if (completion?.error) {
        this.failure = `Failed to capture the ${name}: ${completion.error.message}`;
      } else if (completion?.result?.outcome === "completed") {
        this.resultService.show(toLevelCapture(request, yield* call(readLevelViewerImage(path))));
        this.log.info(`Captured a ${name}:`, path);
        void this.resultService.drawAnnotations();
      }
    } catch (error: unknown) {
      this.fail(`capture the ${name}`, error);
    } finally {
      yield* call(this.restoreLook(options, weather));
      this.isViewHeld = false;
    }
  }

  /** Stops the capture running, which writes nothing. */
  public stop(): void {
    this.operation.cancel();
  }

  /**
   * Opens the dialog on `mode` and every default.
   *
   * @param mode - What it captures first.
   */
  private openOn(mode: ELevelCaptureMode): void {
    const volume: Nullable<ILevelMapVolume> = this.volume;

    if (mode === ELevelCaptureMode.MAP ? !this.canOpenMap : !this.canOpen) {
      return;
    }

    runInAction(() => {
      this.resultService.reset();
      this.failure = null;
      this.mode = mode;
      this.mapParameters = volume ? toDefaultLevelMapParameters(this.levelMapRect, volume) : null;
      this.cubeParameters = toDefaultLevelCubeParameters();
      this.isOpen = true;
    });
  }

  /**
   * @param request - The capture.
   * @param path - The PNG it is written to.
   * @param job - The job it runs as, by its id, and where its progress is told.
   * @returns How it ended, or null with no viewport attached.
   */
  private captureShots(
    request: TLevelCaptureRequest,
    path: string,
    job: [string, Channel<JobProgress>]
  ): Promise<Nullable<CaptureOutcome>> {
    switch (request.mode) {
      case ELevelCaptureMode.MAP:
        return this.renderService.captureMap(toRenderMapCapture(request.parameters), path, job);
      case ELevelCaptureMode.CUBEMAP:
        return this.renderService.captureCube(toRenderCubeCapture(request.parameters), path, job);
      default:
        return assertExhaustive(request);
    }
  }

  /**
   * Shows and lights the view as a map's look asks: the map's weather, or the view's own as the hold keeps it; a
   * cubemap keeps the view's whole look.
   *
   * @param parameters - The map's parameters.
   * @param options - The view's options as they stand, which the switches change.
   */
  private async applyMapLook(parameters: ILevelMapParameters, options: ILevelViewOptions): Promise<void> {
    const { isSpawned, isWatered, isShadowed, look } = parameters;

    this.viewService.setOptions({
      ...options,
      isSpawnedProps: isSpawned && options.isSpawnedProps,
      isSpawnedItems: isSpawned && options.isSpawnedItems,
      isSpawnedWeapons: isSpawned && options.isSpawnedWeapons,
      isSpawnedLamps: isSpawned && options.isSpawnedLamps,
      isWaterVisible: isWatered,
      isShadowed,
    });

    if (look === ELevelMapLook.CURRENT_VIEW) {
      return;
    }

    try {
      await this.weatherService.cutTo(MAP_WEATHER_CYCLE);
    } catch (error: unknown) {
      // A game with no map weather is captured under its own at noon, its clock held there.
      this.log.warn("No map weather to capture under, the current one at noon instead:", error);
      this.renderService.seekWeather(LEVEL_WEATHER_NOON);
    }
  }

  /**
   * Puts the view back as it was before the capture; a weather that cannot be played again is kept as `failure`.
   *
   * @param options - Its options then.
   * @param weather - Its weather then.
   */
  private async restoreLook(options: ILevelViewOptions, weather: ILevelWeatherHold): Promise<void> {
    this.viewService.setOptions(options);

    try {
      await this.weatherService.release(weather);
    } catch (error: unknown) {
      this.fail("put the weather back", error);
    } finally {
      this.renderService.seekWeather(weather.time);
    }
  }

  @BoundAction()
  private fail(what: string, error: unknown): void {
    this.log.error(`Failed to ${what}:`, error);
    this.failure = `Failed to ${what}: ${error instanceof Error ? error.message : String(error)}`;
  }
}
