import { Channel } from "@tauri-apps/api/core";
import { inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, Computed, reaction } from "@wirestate/mobx";
import { call, cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { CaptureOutcome } from "@xrf/ipc/types/xrf-app-render";
import { JobProgress } from "@xrf/ipc/types/xrf-job";
import {
  ERenderDlssAvailability,
  RenderAntialiasing,
  RenderCubeCapture,
  RenderDlssAvailability,
  RenderLevelHit,
  RenderLoadReport,
  RenderMapCapture,
  RenderViewOptions,
  RenderViewportId,
} from "@xrf/ipc/types/xrf-renderer";
import {
  EWorldCameraCommand,
  EWorldWeatherPlay,
  WorldCamera,
  WorldCameraPose,
  WorldToggles,
  WorldWeatherControl,
  WorldWeatherPlay,
  WorldWeatherReport,
} from "@xrf/ipc/types/xrf-world";
import { Logger } from "@xrf/logging";
import { Maybe, Nullable } from "@xrf/types";

import { ILevelGoTo, toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelBox, toLevelBox } from "@/core/level/lib/extent/level-extent";
import {
  ILevelFeatureOptions,
  TLevelOutputOptions,
  toLevelFocusDistance,
  toLevelOutput,
  toLevelRendererAntialiasing,
} from "@/core/level/lib/features/level-feature-options";
import { toLevelHelperRenderOptions, toLevelSpawnIndices } from "@/core/level/lib/objects/level-helper-render";
import {
  ILevelPickParts,
  relocateLevelObjects,
  relocateLevelPick,
  TLevelPick,
  toLevelPick,
} from "@/core/level/lib/pick/level-pick";
import { toLevelPickSelection } from "@/core/level/lib/pick/level-pick-selection";
import { DEFAULT_LEVEL_RENDER_CONFIG, ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { toLevelFrameOverlays } from "@/core/level/lib/render/level-render-frame";
import { toLevelCameraAt, toLevelViewOptions } from "@/core/level/lib/render/level-render-view";
import { ILevelResident, toLevelResident } from "@/core/level/lib/render/level-resident";
import { LevelSceneShow } from "@/core/level/lib/render/level-scene-show";
import { ELevelShading, toShownLevelShading } from "@/core/level/lib/view/level-shading";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelViewService } from "@/core/level/services/level-view";
import { ELevelSelectionMode, LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { toDrawnAntialiasing } from "@/core/render/lib/features/render-antialiasing-choices";
import { toDrawnRenderLightLevel } from "@/core/render/lib/features/render-light-level-choices";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { toNativeWorldToggles } from "@/core/render/lib/native/native-view-options";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { toRenderOutput } from "@/core/render/lib/settings/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/**
 * Owns the native viewport the open level is drawn in: everything said to it, the scene it shows the level as, and what
 * it is asked of the level once resident.
 */
@Injectable()
export class LevelRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /**
   * What the renderer says of the open level once it is resident whole: idle until then, and failed with why where it
   * could not say, which the panels show.
   */
  public readonly resident: CommandOperation<ILevelResident> = new CommandOperation(this.log);

  private readonly config: ILevelRenderConfig = DEFAULT_LEVEL_RENDER_CONFIG;

  /** Which scene shows the open level, and whether it is revealed. */
  private readonly shown: LevelSceneShow = new LevelSceneShow({
    onLeft: () => this.leaveScene(),
    onResident: () => void this.describeResident(),
  });

  /** The level open, whose extent and start frame the camera. */
  private level: Nullable<SelectedLevelDescription> = null;
  /**
   * Where the camera was last stood: the level's start, or a place gone to. The toolbar's speeds and lens are sent
   * with it, so changing one leaves the camera where it has flown rather than taking it back.
   */
  private viewpoint: Nullable<ILevelViewpoint> = null;
  /** Stops hearing the viewport's focus, while one is attached. */
  private unlisten: Nullable<() => void> = null;

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly weatherService: LevelWeatherService = inject(LevelWeatherService),
    private readonly lookService: LevelLookService = inject(LevelLookService),
    renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {
    super(renderSettingsService);
  }

  /**
   * Whether the viewport shows the open level: not until a frame has been drawn with everything the level opens with, so
   * a level never assembles in front of whoever opened it.
   */
  @Computed()
  public get isRevealed(): boolean {
    return this.shown.isRevealed;
  }

  /**
   * The sun or moon the sky draws now, by its `suns.ltx` section, or null for neither: apart from the rest of what the
   * renderer applied, which changes with every report while the weather plays.
   */
  @Computed()
  public get drawnSun(): Nullable<string> {
    return this.applied?.sun ?? null;
  }

  /** Whether the GPU drawing the view traces rays, taken as yes until a frame names the GPU. */
  @Computed()
  public get isTraceable(): boolean {
    return !this.frame.adapter || this.frame.hasRayQueries;
  }

  /** Whether the view's frame is drawn smaller than the viewport. */
  @Computed()
  public get isUpscaled(): boolean {
    return this.frame.renderWidth > 0 && this.frame.renderWidth < this.frame.width;
  }

  /** Whether the GPU drawing the view upscales with DLSS, and what it lacks where not: available until a frame says. */
  @Computed()
  public get dlss(): RenderDlssAvailability {
    return this.frame.adapter ? this.frame.dlss : ERenderDlssAvailability.AVAILABLE;
  }

  /** Whether the GPU drawing the view reconstructs rays with DLSS: available until a frame says otherwise. */
  @Computed()
  public get rayReconstruction(): RenderDlssAvailability {
    return this.frame.adapter ? this.frame.rayReconstruction : ERenderDlssAvailability.AVAILABLE;
  }

  /** Whether the view's last frame was reconstructed by DLSS ray reconstruction, its traces taken raw. */
  @Computed()
  public get isRayReconstructed(): boolean {
    return this.frame.isRayReconstructed;
  }

  /** What smooths the view's frame: the toolbar's antialiasing over the settings', FSR for DLSS on a GPU without it. */
  @Computed()
  public get antialiasing(): RenderAntialiasing {
    return toDrawnAntialiasing(
      toLevelRendererAntialiasing(
        this.renderSettingsService.rendererFeatures.antialiasing,
        this.viewService.features,
        this.viewService.options.isAntialiased
      ),
      this.dlss
    );
  }

  /** What the viewport shows of its surfaces: the view's shading, the final frame for FSR's while FSR is not drawing. */
  @Computed()
  public get shading(): ELevelShading {
    return toShownLevelShading(this.viewService.shading, this.antialiasing);
  }

  /** What the level is asked to be drawn with: the toolbar, the look and the settings together. */
  @Computed()
  public get viewOptions(): RenderViewOptions {
    const output: Required<TLevelOutputOptions> = toLevelOutput(
      this.viewService.features.output,
      this.renderSettingsService.renderResolution,
      this.renderSettingsService.renderOutputScale
    );

    const options: RenderViewOptions = toLevelViewOptions({
      features: this.renderSettingsService.rendererFeatures,
      hemiStrength: this.viewService.hemiStrength,
      lod: this.viewService.lod,
      look: this.lookService.look,
      options: this.viewService.options,
      output: toRenderOutput(output.resolution, output.scale),
      shading: this.shading,
      sunShafts: this.weatherService.sunShafts,
      view: {
        ...this.viewService.features,
        lightLevel:
          this.viewService.features.lightLevel === null
            ? null
            : toDrawnRenderLightLevel(
                this.viewService.features.lightLevel,
                this.renderSettingsService.isDevLightLevelAvailable
              ),
      },
    });
    const { hidden, selected } = this.viewportService;

    return {
      ...options,
      helpers: toLevelHelperRenderOptions(
        this.viewService.helpers,
        selected,
        hidden,
        this.loadService.objects?.objects ?? []
      ),
    };
  }

  /** What of the level's world the toolbar lets play, less the spawned objects hidden one by one. */
  @Computed()
  public get worldToggles(): WorldToggles {
    return toNativeWorldToggles(this.viewService.options, toLevelSpawnIndices(this.viewportService.hidden));
  }

  /**
   * Stands the camera at a place, facing the way asked.
   *
   * @param goTo - Where, as the readout states it.
   */
  @BoundAction()
  public goTo(goTo: ILevelGoTo): void {
    this.stand(toLevelGoToViewpoint(goTo));
  }

  /**
   * Says what of the open level is drawn under a point of the viewport, which is then what is selected. The scene shown
   * let go meanwhile cancels it, as what it hit names something else now.
   *
   * @param point - Where, in css pixels from the viewport's top left corner.
   * @param isThrough - Whether glass is passed by, picking what stands behind it.
   * @param mode - How what it picks changes the selection.
   */
  @LatestFlow()
  public *pick(
    point: IRenderViewPoint,
    isThrough: boolean,
    mode: ELevelSelectionMode = ELevelSelectionMode.REPLACE
  ): TFlow {
    const viewport: Nullable<NativeViewport> = this.viewport;

    if (!viewport || !this.level) {
      return;
    }

    const hit: Nullable<RenderLevelHit> = yield* call(viewport.pick({ x: point.x, y: point.y, isThrough }));

    this.viewportService.notePicked(hit ? toLevelPick(hit, this.loadService.objects) : null, mode);
  }

  /**
   * Focuses the view's lens on what is drawn under a point of the viewport, its depth of field turned on, and stops
   * picking the focus; a click meeting nothing of the level leaves the focus as it was.
   *
   * @param point - Where, in css pixels from the viewport's top left corner.
   * @param isThrough - Whether glass is passed by, focusing on what stands behind it.
   */
  @LatestFlow()
  public *focusAt(point: IRenderViewPoint, isThrough: boolean): TFlow {
    const viewport: Nullable<NativeViewport> = this.viewport;

    this.viewportService.setFocusPicking(false);

    if (!viewport || !this.level) {
      return;
    }

    const hit: Nullable<RenderLevelHit> = yield* call(viewport.pick({ x: point.x, y: point.y, isThrough }));
    const depth: Nullable<number> = hit?.depth ?? null;

    if (depth !== null && depth > 0) {
      const features: ILevelFeatureOptions = this.viewService.features;

      this.viewService.setFeatures({
        ...features,
        camera: { ...features.camera, focusDistance: toLevelFocusDistance(depth), isDefocused: true },
      });
    }
  }

  /**
   * @param object - A spawned object, by its index among the level's.
   * @returns Its bounding sphere in renderer space, centre then radius; null until its model is drawn or with no
   *   viewport attached.
   */
  public async locateSpawnObject(object: number): Promise<Nullable<[number, number, number, number]>> {
    return this.viewport ? this.viewport.locateSpawnObject(object) : null;
  }

  /**
   * @param path - The PNG file to write the viewport's next finished frame to, without its overlays.
   * @returns Whether it was written, which it is not with no viewport attached; rejects for a screenshot that failed.
   */
  public async saveScreenshot(path: string): Promise<boolean> {
    return this.viewport
      ? this.viewport.ask(
          "saveScreenshot",
          async (id: RenderViewportId) => {
            await levelsCommands.saveScreenshot(id, path);

            return true;
          },
          false
        )
      : false;
  }

  /**
   * @param capture - The level map asked for.
   * @param path - The PNG file to write it to.
   * @param job - The job it runs as, by its id, and where its progress is told.
   * @returns How it ended, or null with no viewport attached; rejects for a capture that failed.
   */
  public async captureMap(
    capture: RenderMapCapture,
    path: string,
    job: [string, Channel<JobProgress>]
  ): Promise<Nullable<CaptureOutcome>> {
    return this.viewport
      ? this.viewport.ask<Nullable<CaptureOutcome>>(
          "captureMap",
          (id: RenderViewportId) => levelsCommands.captureMap({ viewport: id, capture, path }, ...job),
          null
        )
      : null;
  }

  /**
   * @param capture - The cubemap asked for, around the camera.
   * @param path - The PNG file to write its cross to.
   * @param job - The job it runs as, by its id, and where its progress is told.
   * @returns How it ended, or null with no viewport attached; rejects for a capture that failed.
   */
  public async captureCube(
    capture: RenderCubeCapture,
    path: string,
    job: [string, Channel<JobProgress>]
  ): Promise<Nullable<CaptureOutcome>> {
    return this.viewport
      ? this.viewport.ask<Nullable<CaptureOutcome>>(
          "captureCube",
          (id: RenderViewportId) => levelsCommands.captureCube({ viewport: id, capture, path }, ...job),
          null
        )
      : null;
  }

  /**
   * Plays the level's weather on from a time of day, which is where its clock stands from then on.
   *
   * @param time - Seconds since midnight.
   */
  @BoundAction()
  public seekWeather(time: number): void {
    this.weatherService.seekTo(time);
    this.viewport?.seekWeather(time);
  }

  /**
   * @param name - The weather effect to play over the cycle from the clock's time, or null to end the one playing.
   */
  @BoundAction()
  public playWeatherEffect(name: Nullable<string>): void {
    this.viewport?.playWeatherEffect(name);
  }

  /** Plays a weather ambient effect near the camera at once, ending the one playing; none plays indoors. */
  @BoundAction()
  public playAmbientEffect(): void {
    this.viewport?.playAmbientEffect();
  }

  protected start(viewport: NativeViewport): Array<() => void> {
    return [
      this.loadService.followLevel(this.openLevel),
      // What is picked stays as the level streams afresh: the parts read again find it anew (`relocatePick`).
      this.loadService.followScene(this.shown.move),
      reaction(() => this.viewService.cameraOptions, this.applyCamera),
      reaction(
        () => this.viewOptions,
        (options: RenderViewOptions) => viewport.setViewOptions(options),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        () => this.worldToggles,
        (toggles: WorldToggles) => viewport.setWorldToggles(toggles),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): ILevelPickParts => ({ objects: this.loadService.objects, surfaces: this.loadService.surfaces.value }),
        this.relocatePick,
        { equals: comparer.shallow }
      ),
      reaction(
        () => this.viewportService.picked,
        (picked: Nullable<TLevelPick>) => viewport.setSelection(toLevelPickSelection(picked)),
        { fireImmediately: true }
      ),
      // Keyed by the extent and the switches alone: the overlays themselves are long arrays.
      reaction(
        () => {
          const level: Maybe<SelectedLevelDescription> = this.loadService.level.value?.selected.value;
          const { isAxesVisible, isGridVisible, isSunMarked } = this.viewService.options;
          const box: Nullable<ILevelBox> = level ? toLevelBox(level.bounds) : null;

          return { box, isAxesVisible, isGridVisible, isSunMarked };
        },
        ({ box, ...options }) => viewport.setOverlays(toLevelFrameOverlays(box, options, this.config)),
        { equals: comparer.structural, fireImmediately: true }
      ),
      ...this.watchWeather(viewport),
    ];
  }

  /**
   * The level's weather played by the viewport, and how it plays, as either changes.
   *
   * @param viewport - The viewport it plays in.
   * @returns What stops watching.
   */
  private watchWeather(viewport: NativeViewport): Array<() => void> {
    const { weatherService } = this;

    return [
      reaction(
        () => weatherService.weather,
        (play: Nullable<WorldWeatherPlay>, previous: Maybe<Nullable<WorldWeatherPlay>>) => {
          viewport.playWeather(play ?? { kind: EWorldWeatherPlay.NONE }, weatherService.transition);

          // The renderer's clock starts at noon: a level's first weather plays on from where its own clock stands.
          if (play && !previous) {
            viewport.seekWeather(weatherService.time);
          }
        },
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): WorldWeatherControl => ({
          ...weatherService.control,
          // The keyframe set by hand stands its sun by its own angles.
          isDynamicSun: weatherService.control.isDynamicSun && !weatherService.isManual,
        }),
        (control: WorldWeatherControl) => viewport.setWeatherControl(control),
        { equals: comparer.structural, fireImmediately: true }
      ),
    ];
  }

  protected onAttached(container: HTMLElement): void {
    const onFocus = (): void => this.viewportService.noteSceneFocused(true);
    const onBlur = (): void => this.viewportService.noteSceneFocused(false);

    container.addEventListener("focus", onFocus);
    container.addEventListener("blur", onBlur);

    this.unlisten = () => {
      container.removeEventListener("focus", onFocus);
      container.removeEventListener("blur", onBlur);
      this.viewportService.noteSceneFocused(false);
    };
  }

  protected onClick(point: IRenderViewPoint, isThrough: boolean, isToggling: boolean): void {
    if (this.viewportService.isFocusPicking) {
      void this.focusAt(point, isThrough);
    } else {
      void this.pick(point, isThrough, isToggling ? ELevelSelectionMode.TOGGLE : ELevelSelectionMode.REPLACE);
    }
  }

  protected onDetached(): void {
    this.unlisten?.();
    this.unlisten = null;
  }

  protected release(): void {
    this.level = null;
    this.viewpoint = null;
    this.shown.release();
  }

  protected onCamera(pose: WorldCameraPose): void {
    this.viewportService.noteCamera(toLevelCameraReading(pose));
  }

  protected onWeather(report: Nullable<WorldWeatherReport>): void {
    this.weatherService.noteReport(report);
  }

  protected onLoad(report: RenderLoadReport): void {
    this.viewportService.noteLoad(report);
    this.shown.noteLoad(report);
  }

  /**
   * Asks what each shader table entry draws, what each texture came to and what the level could not draw, once it is
   * resident whole; the scene shown let go meanwhile cancels it, as it describes something else.
   */
  @LatestFlow("resident")
  private *describeResident(): TFlow {
    const viewport: Nullable<NativeViewport> = this.viewport;

    if (!viewport) {
      return;
    }

    yield* this.resident.run({
      name: "Describe resident level",
      command: () =>
        Promise.all([viewport.measureSurfaces(), viewport.describeTextureLoads(), viewport.describeProblems()]).then(
          ([measured, textures, problems]) => toLevelResident(measured, textures, problems)
        ),
      failure: "Could not describe the resident level",
    });
  }

  /** Lets go of what was asked of the scene shown: what it described and what a click picked in it name it alone. */
  @BoundAction()
  private leaveScene(): void {
    cancelFlow(this, "resident");
    cancelFlow(this, "pick");
    cancelFlow(this, "focusAt");
    this.resident.reset();
  }

  @BoundAction()
  private openLevel(selected: Nullable<SessionSnapshot<SelectedLevelDescription>>): void {
    const level: Nullable<SelectedLevelDescription> = selected?.value ?? null;
    const sessionId: Nullable<string> = selected?.sessionId ?? null;

    this.level = level;

    // A level read again in its session stays shown: the scene it streams as moves apart (`shown.move`).
    if (!this.shown.open(sessionId, this.loadService.scene)) {
      return;
    }

    this.stand(toLevelStartViewpoint(level?.bounds ?? null, this.loadService.start.value));
    // Another level's surfaces and objects are numbered afresh, and its load starts from nothing read.
    this.viewportService.notePicked(null);
    this.viewportService.revealAll();
    this.viewportService.noteLoad(null);

    this.show(
      (viewport: NativeViewport) =>
        viewport.ask("showLevel", (id: RenderViewportId) => levelsCommands.showLevel(id, sessionId), null),
      this.shown.noteShown
    );
  }

  /**
   * Keeps what is picked, selected and hidden across the open level's parts read anew, each found by what it is, or
   * drops it with what it named.
   *
   * @param after - The parts as read now.
   * @param before - The parts it was picked among.
   */
  @BoundAction()
  private relocatePick(after: ILevelPickParts, before: ILevelPickParts): void {
    const { hidden, picked, selected } = this.viewportService;
    const relocated: Nullable<TLevelPick> = picked ? relocateLevelPick(picked, before, after) : null;

    if (after.objects === before.objects) {
      if (relocated !== picked) {
        this.viewportService.notePickRelocated(relocated);
      }

      return;
    }

    this.viewportService.notePickRelocated(
      relocated,
      relocateLevelObjects(selected, before.objects, after.objects),
      relocateLevelObjects(hidden, before.objects, after.objects)
    );
  }

  /** The toolbar's speeds and lens, from the same place: the camera keeps where it has flown. */
  @BoundAction()
  private applyCamera(camera: ILevelCameraOptions): void {
    const viewpoint: ILevelViewpoint =
      this.viewpoint ?? toLevelStartViewpoint(this.level?.bounds ?? null, this.loadService.start.value);

    this.viewport?.setCamera(this.toCamera(viewpoint, camera));
  }

  /**
   * Stands the camera at a place, anew: the same place again moves nothing a description compares, and the camera has
   * flown on since.
   *
   * @param viewpoint - Where it stands and what it looks at.
   */
  private stand(viewpoint: ILevelViewpoint): void {
    const viewport: Nullable<NativeViewport> = this.viewport;

    this.viewpoint = viewpoint;
    viewport?.setCamera(this.toCamera(viewpoint, this.viewService.cameraOptions));
    viewport?.commandCamera({ kind: EWorldCameraCommand.RESET });
  }

  private toCamera(viewpoint: ILevelViewpoint, options: ILevelCameraOptions): WorldCamera {
    return toLevelCameraAt(viewpoint, options, this.config);
  }
}
