import { Channel } from "@tauri-apps/api/core";
import { open as openDialog, save } from "@tauri-apps/plugin-dialog";
import { inject, Injectable, OnDeactivation, OnEvent, WireEvent } from "@wirestate/core";
import { BoundAction, Computed, RefObservable, runInAction } from "@wirestate/mobx";
import { getPathName } from "@xrf/path";
import { assertExhaustive, Nullable } from "@xrf/types";

import {
  CAPTURE_LEVEL_CUBEMAP_KEYBIND_COMMAND,
  CAPTURE_LEVEL_MAP_KEYBIND_COMMAND,
} from "@/applications/level-viewer/commands";
import { ELevelCaptureMode } from "@/applications/level-viewer/lib/capture/level-capture";
import {
  ILevelCubeParameters,
  isLevelCubeCapturable,
  toDefaultLevelCubeParameters,
  toLevelCubeFaceSize,
  toLevelCubeSupersampling,
  toRenderCubeCapture,
} from "@/applications/level-viewer/lib/cube/level-cube-capture";
import {
  DEFAULT_LEVEL_CUBE_EXPORT,
  ILevelCubeExportSettings,
  isLevelCubeExportSingle,
  toCubeExportRequest,
} from "@/applications/level-viewer/lib/cube/level-cube-export";
import {
  toLevelViewerFileStem,
  toLevelViewerLevelName,
} from "@/applications/level-viewer/lib/files/level-viewer-file-stem";
import {
  copyLevelViewerImage,
  ILevelViewerImage,
  readLevelViewerImage,
} from "@/applications/level-viewer/lib/files/level-viewer-image";
import {
  DEFAULT_LEVEL_MAP_ANNOTATIONS,
  ILevelMapAnnotations,
  toMapAnnotationsRequest,
} from "@/applications/level-viewer/lib/map/level-map-annotations";
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
import {
  DEFAULT_LEVEL_MAP_EXPORT,
  ILevelMapExportSettings,
  toLevelMapExportExtension,
  toLevelMapExportSettings,
  toMapExportRequest,
} from "@/applications/level-viewer/lib/map/level-map-export";
import { KeybindCommand } from "@/core/commands";
import { levelsCommands } from "@/core/ipc/commands/levels";
import { systemCommands } from "@/core/ipc/commands/system";
import {
  CaptureOutcome,
  JobKind,
  LevelMapDescription,
  LEVELS_JOB_CAPTURE_CUBE,
  LEVELS_JOB_CAPTURE_MAP,
  SelectedLevelDescription,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { EXrayExtension } from "@/core/ipc/types/xrf-extension";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import { describeJobFailure, IJobNotice, IJobOutcome, IJobSettledPayload, JOB_SETTLED_EVENT } from "@/core/jobs/lib";
import { JobOperation } from "@/core/jobs/lib/job-operation";
import { JobsService } from "@/core/jobs/services/jobs";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LEVEL_WEATHER_NOON } from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherHold } from "@/core/level/lib/weather/level-weather-hold";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { noop } from "@/lib/callbacks/noop";
import { Logger } from "@/lib/logging";
import { call, ExclusiveFlow, LatestFlow, TFlow } from "@/lib/mobx";

/** The weather cycle `CDemoRecord` captures a level map under. */
const MAP_WEATHER_CYCLE: string = "map";

/** What each mode's capture is called, in its notices and failures. */
export const CAPTURE_NAMES: Record<ELevelCaptureMode, string> = {
  [ELevelCaptureMode.MAP]: "level map",
  [ELevelCaptureMode.CUBEMAP]: "cubemap",
};

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

/** A capture written: its picture as the dialog shows it and the mode it was taken in, a map with the ground it covers. */
export type TLevelCapture =
  | { mode: ELevelCaptureMode.MAP; image: ILevelViewerImage; area: ILevelMapRect }
  | { mode: ELevelCaptureMode.CUBEMAP; image: ILevelViewerImage };

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
 * The level viewer's captures of a level map or a cubemap: the dialog's parameters, the capture run as a job under the
 * look asked for with the view put back after, and the exports of the picture it wrote.
 */
@Injectable()
export class LevelViewerCaptureService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** The capture's job: its progress, and stopping it. */
  public readonly operation: JobOperation<CaptureOutcome>;

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

  /** What was last captured while the dialog is open, until it is deleted, left or the dialog closes. */
  @RefObservable()
  public captured: Nullable<TLevelCapture> = null;

  /** What of a map's annotations is drawn over it, and the SVG drawn of them, as an object URL, until it changes. */
  @RefObservable()
  public mapAnnotations: ILevelMapAnnotations = DEFAULT_LEVEL_MAP_ANNOTATIONS;

  @RefObservable()
  public annotationsUrl: Nullable<string> = null;

  /** How the next export of a map and of a cubemap is written. */
  @RefObservable()
  public mapExport: ILevelMapExportSettings = DEFAULT_LEVEL_MAP_EXPORT;

  @RefObservable()
  public cubeExport: ILevelCubeExportSettings = DEFAULT_LEVEL_CUBE_EXPORT;

  /** Where the last export was written, its first file for six, until another capture is shown. */
  @RefObservable()
  public exported: Nullable<string> = null;

  @RefObservable()
  public isExporting: boolean = false;

  /** Why the last thing done failed, until another is done. */
  @RefObservable()
  public failure: Nullable<string> = null;

  /** Whether a capture has the view, from its look applied until the view is put back as it was. */
  @RefObservable()
  private isViewHeld: boolean = false;

  /**
   * The last of the annotations' writes beside the map, each sent once the one before it ended, so the file on disk is
   * the last drawn, which an export reads.
   */
  private annotationsWritten: Promise<unknown> = Promise.resolve();

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly renderService: LevelRenderService = inject(LevelRenderService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly weatherService: LevelWeatherService = inject(LevelWeatherService),
    jobsService: JobsService = inject(JobsService)
  ) {
    this.operation = new JobOperation(jobsService, [LEVELS_JOB_CAPTURE_MAP, LEVELS_JOB_CAPTURE_CUBE]);
  }

  @OnDeactivation()
  public onDeactivation(): void {
    this.show(null);
  }

  /** The level open, or null for none. */
  @Computed()
  public get level(): Nullable<SelectedLevelDescription> {
    return this.loadService.level.value?.selected.value ?? null;
  }

  /** The backend's opening of the level, which its annotations are drawn from, or null for none. */
  @Computed()
  public get sessionId(): Nullable<string> {
    return this.loadService.level.value?.selected.sessionId ?? null;
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
    if (!this.isCapturing && !this.isExporting) {
      this.isOpen = false;
      this.show(null);
    }
  }

  /** Goes back to the parameters from what was captured, which stays in its folder; nothing while it is exported. */
  @BoundAction()
  public back(): void {
    if (!this.isExporting) {
      this.show(null);
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
   * @param annotations - What of a map's annotations is drawn from now on, which draws them again.
   */
  @BoundAction()
  public setMapAnnotations(annotations: ILevelMapAnnotations): void {
    this.mapAnnotations = annotations;
    void this.drawAnnotations();
  }

  /** Draws the annotations asked for over the map shown and writes them beside it, then shows them while it is. */
  @LatestFlow()
  public *drawAnnotations(): TFlow {
    const captured: Nullable<TLevelCapture> = this.captured;
    const sessionId: Nullable<string> = this.sessionId;

    if (captured?.mode !== ELevelCaptureMode.MAP || !sessionId) {
      return;
    }

    const request = toMapAnnotationsRequest(this.mapAnnotations, captured.image.path, captured.area);
    const writing: Promise<SessionSnapshot<string>> = this.annotationsWritten.then(() =>
      levelsCommands.writeMapAnnotations(sessionId, request)
    );

    this.annotationsWritten = writing.catch(noop);

    try {
      const { value } = yield* call(writing);

      if (this.captured === captured) {
        this.showAnnotations(URL.createObjectURL(new Blob([value], { type: "image/svg+xml" })));
      }
    } catch (error: unknown) {
      this.fail("draw the map's annotations", error);
    }
  }

  /**
   * @param patch - What of the map's export changes, kept to an export the backend writes.
   */
  @BoundAction()
  public setMapExport(patch: Partial<ILevelMapExportSettings>): void {
    this.mapExport = toLevelMapExportSettings(this.mapExport, patch);
  }

  /**
   * @param patch - What of the cubemap's export changes.
   */
  @BoundAction()
  public setCubeExport(patch: Partial<ILevelCubeExportSettings>): void {
    this.cubeExport = { ...this.cubeExport, ...patch };
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
    const name: string = CAPTURE_NAMES[mode];
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

      const { result, error } = yield* this.operation.run({
        kind: CAPTURE_JOB_KINDS[mode],
        invoke: async (id, progress): Promise<CaptureOutcome> => {
          const outcome: Nullable<CaptureOutcome> = await this.captureShots(request, path, [id, progress]);

          if (!outcome) {
            throw new Error(`The level viewer has no viewport attached to capture a ${name} in`);
          }

          return outcome;
        },
        describe: (outcome: IJobOutcome<CaptureOutcome>): IJobNotice => this.describe(mode, path, outcome),
      });

      if (error) {
        this.failure = `Failed to capture the ${name}: ${error.message}`;
      } else if (result?.outcome === "completed") {
        this.show(toLevelCapture(request, yield* call(readLevelViewerImage(path))));
        this.log.info(`Captured a ${name}:`, path);
        void this.drawAnnotations();
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
   * Writes what was captured again as its export settings ask, where a save dialog answers: a file, or the folder a
   * cubemap's six faces are written into.
   */
  @ExclusiveFlow()
  public *exportAs(): TFlow {
    const captured: Nullable<TLevelCapture> = this.captured;

    if (!captured) {
      return;
    }

    const { mode, image } = captured;
    const name: string = CAPTURE_NAMES[mode];
    const level: Nullable<string> = toLevelViewerLevelName(this.level?.source);
    const target: Nullable<string> = yield* call(this.askExportTarget(mode, level ? getPathName(level) : "level"));

    if (!target || this.captured !== captured) {
      return;
    }

    this.isExporting = true;
    this.failure = null;

    try {
      let exported: string = target;

      if (mode === ELevelCaptureMode.MAP) {
        const request = toMapExportRequest(this.mapExport, image.path, target, Math.max(image.width, image.height));

        // The annotations drawn last are on disk before the export reads them.
        yield* call(this.annotationsWritten);
        yield* call(levelsCommands.exportMap(request));
      } else {
        const request = toCubeExportRequest(this.cubeExport, image.path, target, toLevelCubeFaceSize(image.width));
        const written: Array<string> = yield* call(levelsCommands.exportCube(request));

        exported = written[0] ?? target;
      }

      if (this.captured === captured) {
        this.exported = exported;
      }

      this.log.info(`Exported a ${name}:`, target);
    } catch (error: unknown) {
      this.fail(`export the ${name}`, error);
    } finally {
      this.isExporting = false;
    }
  }

  /** Deletes what was captured for good, with its annotations once they are written; nothing while it is exported. */
  public async remove(): Promise<void> {
    if (this.isExporting) {
      return;
    }

    await this.attempt("delete the capture", async ({ image }: TLevelCapture) => {
      await this.annotationsWritten;
      await levelsCommands.deleteScreenshot(image.path);
      this.log.info("Deleted a capture:", image.path);
      this.show(null);
    });
  }

  /** Shows what was captured selected in its folder. */
  public reveal(): Promise<void> {
    return this.attempt("show the capture", ({ image }: TLevelCapture) => systemCommands.revealPath(image.path));
  }

  /** Shows the last export selected in its folder. */
  public async revealExport(): Promise<void> {
    if (this.exported) {
      await systemCommands.revealPath(this.exported).catch((error: unknown) => this.fail("show the export", error));
    }
  }

  /** Puts the capture's path on the clipboard. */
  public copyPath(): Promise<void> {
    return this.attempt("copy the capture's path", ({ image }: TLevelCapture) =>
      navigator.clipboard.writeText(image.path)
    );
  }

  /** Puts the capture's image on the clipboard. */
  public copyImage(): Promise<void> {
    return this.attempt("copy the capture", ({ image }: TLevelCapture) => copyLevelViewerImage(image));
  }

  @OnEvent(JOB_SETTLED_EVENT)
  public onJobSettled(event: WireEvent<IJobSettledPayload>): void {
    this.operation.adopt(event.payload);
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
      this.show(null);
      this.failure = null;
      this.mode = mode;
      this.mapExport = DEFAULT_LEVEL_MAP_EXPORT;
      this.cubeExport = DEFAULT_LEVEL_CUBE_EXPORT;
      this.mapAnnotations = DEFAULT_LEVEL_MAP_ANNOTATIONS;
      this.mapParameters = volume ? toDefaultLevelMapParameters(this.levelMapRect, volume) : null;
      this.cubeParameters = toDefaultLevelCubeParameters();
      this.isOpen = true;
    });
  }

  /**
   * @param mode - What was captured.
   * @param level - The open level's name, which the file is named after.
   * @returns Where its export goes, as the save or folder dialog answers, or null where it was dismissed.
   */
  private async askExportTarget(mode: ELevelCaptureMode, level: string): Promise<Nullable<string>> {
    if (mode === ELevelCaptureMode.MAP) {
      const extension: string = toLevelMapExportExtension(this.mapExport.kind);

      return save({
        defaultPath: `map_${level}.${extension}`,
        filters: [{ extensions: [extension], name: `${extension.toUpperCase()} image` }],
        title: "Export level map",
      });
    }

    if (isLevelCubeExportSingle(this.cubeExport.kind)) {
      return save({
        defaultPath: `cube_${level}.${EXrayExtension.DDS}`,
        filters: [{ extensions: [EXrayExtension.DDS], name: "DDS cube texture" }],
        title: "Export cubemap",
      });
    }

    const folder: Nullable<string | Array<string>> = await openDialog({
      directory: true,
      title: "Export the cubemap's faces into a folder",
    });

    return typeof folder === "string" ? folder : null;
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

  /** What the notification centre says when a capture ends. */
  private describe(mode: ELevelCaptureMode, path: string, outcome: IJobOutcome<CaptureOutcome>): IJobNotice {
    const { result, error } = outcome;
    const name: string = CAPTURE_NAMES[mode];

    if (error) {
      return describeJobFailure(`Could not capture the ${name}`, path, error);
    }

    if (result?.outcome === "cancelled") {
      return {
        details: `Nothing was written: the ${name} is written only once every ${
          mode === ELevelCaptureMode.MAP ? "tile" : "face"
        } is back.`,
        severity: ENotificationSeverity.INFO,
        title: `Capturing the ${name} was stopped`,
      };
    }

    return {
      details: path,
      severity: ENotificationSeverity.SUCCESS,
      title:
        mode === ELevelCaptureMode.MAP
          ? `Captured a ${result?.width ?? 0} × ${result?.height ?? 0} level map`
          : `Captured a cubemap of ${toLevelCubeFaceSize(result?.width ?? 0)} pixel faces`,
    };
  }

  /** Does `action` with what was captured, if anything; a failure is kept as `failure`, as `what` says. */
  private async attempt(what: string, action: (captured: TLevelCapture) => Promise<unknown>): Promise<void> {
    if (this.captured) {
      await action(this.captured).catch((error: unknown) => this.fail(what, error));
    }
  }

  @BoundAction()
  private fail(what: string, error: unknown): void {
    this.log.error(`Failed to ${what}:`, error);
    this.failure = `Failed to ${what}: ${error instanceof Error ? error.message : String(error)}`;
  }

  /** Shows `captured`, or nothing, letting what it replaces go and the export written of it be forgotten. */
  @BoundAction()
  private show(captured: Nullable<TLevelCapture>): void {
    if (this.captured) {
      URL.revokeObjectURL(this.captured.image.url);
    }

    this.captured = captured;
    this.exported = null;
    this.showAnnotations(null);
  }

  /** Shows the annotations drawn at `url`, or none, letting the ones they replace go. */
  @BoundAction()
  private showAnnotations(url: Nullable<string>): void {
    if (this.annotationsUrl) {
      URL.revokeObjectURL(this.annotationsUrl);
    }

    this.annotationsUrl = url;
  }
}
