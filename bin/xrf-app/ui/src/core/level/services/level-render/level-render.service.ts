import { inject, Injectable } from "@wirestate/core";
import { BoundAction, Computed } from "@wirestate/mobx";
import { cancelFlow, LatestFlow, TFlow } from "@xrf/flow";
import { levelsCommands } from "@xrf/ipc/commands/levels";
import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { RenderLoadReport, RenderViewportId } from "@xrf/ipc/types/xrf-renderer";
import { WorldCameraPose, WorldMusicCommand, WorldSoundReport, WorldWeatherReport } from "@xrf/ipc/types/xrf-world";
import { Logger } from "@xrf/logging";
import { Nullable } from "@xrf/types";

import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { DEFAULT_LEVEL_RENDER_CONFIG, ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { ILevelResident, toLevelResident } from "@/core/level/lib/render/level-resident";
import { LevelSceneShow } from "@/core/level/lib/render/level-scene-show";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelLookService } from "@/core/level/services/level-look";
import { LevelRenderCamera } from "@/core/level/services/level-render/level-render.service.camera";
import { LevelRenderCaptures } from "@/core/level/services/level-render/level-render.service.captures";
import { ILevelRenderContext } from "@/core/level/services/level-render/level-render.service.context";
import { LevelRenderDrawing } from "@/core/level/services/level-render/level-render.service.drawing";
import { LevelRenderPicks } from "@/core/level/services/level-render/level-render.service.picks";
import { LevelRenderWeather } from "@/core/level/services/level-render/level-render.service.weather";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { CommandOperation } from "@/core/operations/lib/command-operation";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/viewport/native-render-surface-service";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";
import { INativeViewportClick } from "@/core/render/lib/native/viewport/native-viewport-target";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/**
 * Owns the native viewport the open level is drawn in: its lifetime, the scene it shows the level as, and what it is
 * asked of the level once resident. What is said to it otherwise is its parts': the camera, what the level is drawn
 * with, the picks, the captures and the weather, each reached by its field.
 */
@Injectable()
export class LevelRenderService extends NativeRenderSurfaceService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /**
   * What the renderer says of the open level once it is resident whole: idle until then, and failed with why where it
   * could not say, which the panels show.
   */
  public readonly resident: CommandOperation<ILevelResident> = new CommandOperation(this.log);

  /** Where the camera stands, gone to and flown on from. */
  public readonly camera: LevelRenderCamera;
  /** What the level is drawn with on the GPU drawing it, what its world plays, and the overlays over it. */
  public readonly drawing: LevelRenderDrawing;
  /** What of the level is drawn where the viewport is clicked or hovered, and what that changes. */
  public readonly picks: LevelRenderPicks;
  /** What the viewport writes to files. */
  public readonly captures: LevelRenderCaptures;
  /** The level's weather as the viewport plays it. */
  public readonly weather: LevelRenderWeather;

  /**
   * @returns What the level's scene plays and why, as the world schedules it and the audio hears it; null while no
   *   level is shown or no viewport is attached, and for a description that failed.
   */
  public describeSound(): Promise<Nullable<WorldSoundReport>> {
    return (
      this.viewport
        ?.ask("describeSound", (id: RenderViewportId) => levelsCommands.describeSound(id), null)
        .catch(() => null) ?? Promise.resolve(null)
    );
  }

  /**
   * @param command - How the level's music plays from now on: by the game's schedule, one track by hand, or the track
   *   playing from a point.
   * @returns Whether the world could: not while no level is shown, for a track the level does not have, or for a seek
   *   between tracks.
   */
  public commandMusic(command: WorldMusicCommand): Promise<boolean> {
    return (
      this.viewport
        ?.ask("commandMusic", (id: RenderViewportId) => levelsCommands.commandMusic(id, command), false)
        .catch(() => false) ?? Promise.resolve(false)
    );
  }

  /** Which scene shows the open level, and whether it is revealed. */
  private readonly shown: LevelSceneShow = new LevelSceneShow({
    onLeft: () => this.leaveScene(),
    onResident: () => void this.describeResident(),
  });

  /** The level open, whose extent and start frame the camera. */
  private level: Nullable<SelectedLevelDescription> = null;
  /** Stops hearing the viewport's focus, while one is attached. */
  private unlisten: Nullable<() => void> = null;

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly weatherService: LevelWeatherService = inject(LevelWeatherService),
    lookService: LevelLookService = inject(LevelLookService),
    renderSettingsService: RenderSettingsService = inject(RenderSettingsService)
  ) {
    super(renderSettingsService);

    const config: ILevelRenderConfig = DEFAULT_LEVEL_RENDER_CONFIG;
    const context: ILevelRenderContext = {
      getFrame: () => this.frame,
      getLevel: () => this.level,
      getViewport: () => this.viewport,
    };

    this.camera = new LevelRenderCamera(context, { config, loadService, viewService });
    this.drawing = new LevelRenderDrawing(context, {
      config,
      loadService,
      lookService,
      renderSettingsService,
      viewService,
      viewportService,
      weatherService,
    });
    this.picks = new LevelRenderPicks(context, { loadService, viewService, viewportService });
    this.captures = new LevelRenderCaptures(context, viewService);
    this.weather = new LevelRenderWeather(context, weatherService);
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

  protected start(viewport: NativeViewport): Array<() => void> {
    return [
      this.loadService.followLevel(this.openLevel),
      // What is picked stays as the level streams afresh: the parts read again find it anew.
      this.loadService.followScene(this.shown.move),
      ...this.camera.start(),
      ...this.drawing.start(viewport),
      ...this.picks.start(viewport),
      ...this.weather.start(viewport),
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

  protected onClick(click: INativeViewportClick): void {
    this.picks.click(click);
  }

  protected onHover(point: Nullable<IRenderViewPoint>): void {
    this.picks.rest(point);
  }

  protected onFlySpeed(steps: number): void {
    this.camera.stepSpeed(steps);
  }

  protected onDetached(): void {
    this.unlisten?.();
    this.unlisten = null;
  }

  protected release(): void {
    this.level = null;
    this.camera.release();
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

  /**
   * Lets go of what was asked of the scene shown: what it described and what a click picked or the pointer hovered in
   * it name it alone.
   */
  @BoundAction()
  private leaveScene(): void {
    cancelFlow(this, "resident");
    this.picks.leave();
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

    this.camera.standAtStart();
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
}
