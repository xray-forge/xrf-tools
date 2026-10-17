import { inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, Computed, reaction } from "@wirestate/mobx";
import { AsyncState } from "@xrf/async";
import { Maybe, Nullable } from "@xrf/types";

import { transformError } from "@/core/error/lib";
import { SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import {
  ERenderDlssAvailability,
  RenderAntialiasing,
  RenderDlssAvailability,
  RenderLevelHit,
  RenderLoadReport,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import {
  EWorldCameraCommand,
  EWorldWeatherPlay,
  WorldCamera,
  WorldCameraPose,
  WorldToggles,
  WorldWeatherControl,
  WorldWeatherPlay,
  WorldWeatherReport,
} from "@/core/ipc/types/xrf-world";
import { ILevelGoTo, toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelBox, toLevelBox } from "@/core/level/lib/extent/level-extent";
import { toLevelRendererAntialiasing } from "@/core/level/lib/features/level-feature-options";
import { TLevelPick, toLevelPick } from "@/core/level/lib/pick/level-pick";
import { toLevelPickSelection } from "@/core/level/lib/pick/level-pick-selection";
import { DEFAULT_LEVEL_RENDER_CONFIG, ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { toLevelFrameOverlays } from "@/core/level/lib/render/level-render-frame";
import { toLevelCameraAt, toLevelViewOptions } from "@/core/level/lib/render/level-render-view";
import { toLevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { toLevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { ELevelShading, toShownLevelShading } from "@/core/level/lib/view/level-shading";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { toDrawnAntialiasing } from "@/core/render/lib/features/render-antialiasing-choices";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { toNativeWorldToggles } from "@/core/render/lib/native/native-view-options";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { toRenderOutput } from "@/core/render/lib/settings/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { Logger } from "@/lib/logging";

/**
 * Owns the native viewport the open level is drawn in, and everything said to it.
 */
@Injectable()
export class LevelRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly config: ILevelRenderConfig = DEFAULT_LEVEL_RENDER_CONFIG;

  /** The level open, whose extent and start frame the camera. */
  private level: Nullable<SelectedLevelDescription> = null;
  /**
   * Where the camera was last stood: the level's start, or a place gone to. The toolbar's speeds and lens are sent
   * with it, so changing one leaves the camera where it has flown rather than taking it back.
   */
  private viewpoint: Nullable<ILevelViewpoint> = null;
  /** Bumped by every level opened or closed, so a pick asked of one since replaced notes nothing. */
  private opening: number = 0;
  /** The scene the level open is drawn as, once the renderer has named it: only its load reports reveal the level. */
  private scene: Nullable<number> = null;
  /** The last load report, of whichever scene the viewport drew. */
  private loaded: Nullable<RenderLoadReport> = null;
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
    return toLevelViewOptions({
      features: this.renderSettingsService.rendererFeatures,
      hemiStrength: this.viewService.hemiStrength,
      lod: this.viewService.lod,
      look: this.lookService.look,
      options: this.viewService.options,
      output: toRenderOutput(
        this.viewService.features.output.resolution ?? this.renderSettingsService.renderResolution,
        this.viewService.features.output.scale ?? this.renderSettingsService.renderOutputScale
      ),
      shading: this.shading,
      sunShafts: this.weatherService.sunShafts,
      view: this.viewService.features,
    });
  }

  /** What of the level's world the toolbar lets play. */
  @Computed()
  public get worldToggles(): WorldToggles {
    return toNativeWorldToggles(this.viewService.options);
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
   * Says what of the open level is drawn under a point of the viewport, which is then what is selected.
   *
   * @param point - Where, in css pixels from the viewport's top left corner.
   * @param isThrough - Whether glass is passed by, picking what stands behind it.
   * @returns Settles once the pick is noted: what it hit, or nothing.
   */
  public async pick(point: IRenderViewPoint, isThrough: boolean): Promise<void> {
    const { viewport, opening } = this;

    if (!viewport || !this.level) {
      return;
    }

    const hit: Nullable<RenderLevelHit> = await viewport.pick({ x: point.x, y: point.y, isThrough });

    // Asked of a viewport or a level since replaced, it names something else now.
    if (viewport !== this.viewport || opening !== this.opening) {
      return;
    }

    const picked: Nullable<TLevelPick> = hit ? toLevelPick(hit, this.loadService.spawn) : null;

    this.viewportService.notePicked(picked);
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

  protected onClick(point: IRenderViewPoint, isThrough: boolean): void {
    void this.pick(point, isThrough);
  }

  protected onDetached(): void {
    this.unlisten?.();
    this.unlisten = null;
  }

  protected release(): void {
    this.opening += 1;
    this.level = null;
    this.viewpoint = null;
    this.scene = null;
    this.loaded = null;
  }

  protected onCamera(pose: WorldCameraPose): void {
    this.viewportService.noteCamera(toLevelCameraReading(pose));
  }

  protected onWeather(report: Nullable<WorldWeatherReport>): void {
    this.weatherService.noteReport(report);
  }

  protected onLoad(report: RenderLoadReport): void {
    this.viewportService.noteLoad(report);
    this.loaded = report;
    this.revealWhenReady();
  }

  /**
   * Reveals the level once everything it opens with is resident, so it is never seen half read; a report of the level
   * drawn before it, still loading when this one was asked for, reveals nothing.
   */
  private revealWhenReady(): void {
    const report: Nullable<RenderLoadReport> = this.loaded;

    if (report?.isReady && this.level && this.scene !== null && report.scene === this.scene) {
      this.viewportService.reveal();
      void this.describeResident();
    }
  }

  /**
   * Asks what each shader table entry draws, what each texture came to and what the level could not draw, once it is
   * resident whole; a failure is held for the panels to say.
   */
  private async describeResident(): Promise<void> {
    const { viewport, opening } = this;

    if (!viewport) {
      return;
    }

    // Asked of a viewport or a level since replaced, it describes something else.
    const isCurrent = (): boolean => viewport === this.viewport && opening === this.opening;

    this.viewportService.noteDescription(AsyncState.loading());

    try {
      const [measured, textures, problems] = await Promise.all([
        viewport.measureSurfaces(),
        viewport.describeTextures(),
        viewport.describeProblems(),
      ]);

      if (isCurrent()) {
        this.viewportService.noteSurfaceGeometry(toLevelSurfaceGeometry(measured));
        this.viewportService.noteTextures(toLevelTextureReport(textures));
        this.viewportService.noteProblems(problems);
        this.viewportService.noteDescription(AsyncState.ready<void>(null));
      }
    } catch (error: unknown) {
      if (isCurrent()) {
        this.log.error("Failed to describe the resident level:", transformError(error));
        this.viewportService.noteDescription(AsyncState.failed(transformError(error)));
      }
    }
  }

  @BoundAction()
  private openLevel(selected: Nullable<SessionSnapshot<SelectedLevelDescription>>): void {
    const level: Nullable<SelectedLevelDescription> = selected?.value ?? null;

    this.opening += 1;
    this.level = level;
    this.scene = null;
    this.stand(toLevelStartViewpoint(level?.bounds ?? null, level?.start ?? null));
    // Another level's surfaces and objects are numbered afresh.
    this.viewportService.notePicked(null);
    this.viewportService.conceal();

    this.show(
      (viewport: NativeViewport) => viewport.showLevel(selected?.sessionId ?? null),
      (scene: Nullable<number>) => {
        this.scene = scene;
        this.revealWhenReady();
      }
    );
  }

  /** The toolbar's speeds and lens, from the same place: the camera keeps where it has flown. */
  @BoundAction()
  private applyCamera(camera: ILevelCameraOptions): void {
    const viewpoint: ILevelViewpoint =
      this.viewpoint ?? toLevelStartViewpoint(this.level?.bounds ?? null, this.level?.start ?? null);

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
