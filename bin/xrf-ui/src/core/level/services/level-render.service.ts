import { inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, Computed, reaction } from "@wirestate/mobx";
import { Maybe, Nullable } from "@xrf/types";

import {
  LevelSpawnObject,
  LevelSpawnObjectsDescription,
  SelectedLevelDescription,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import {
  ERenderLevelHit,
  ERenderTextureState,
  RenderLevelHit,
  RenderLoadReport,
  RenderTextureReport,
  RenderTextureState,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import {
  EWorldCameraCommand,
  EWorldWeatherPlay,
  WorldCamera,
  WorldCameraPose,
  WorldSurfaceGeometry,
  WorldSurfaceSpan,
  WorldToggles,
  WorldWeatherControl,
  WorldWeatherPlay,
  WorldWeatherReport,
} from "@/core/ipc/types/xrf-world";
import { ILevelGoTo, toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { ILevelPoint } from "@/core/level/lib/camera/level-point";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelBox, toLevelBox } from "@/core/level/lib/extent/level-extent";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { toLevelPickSelection } from "@/core/level/lib/pick/level-pick-selection";
import { DEFAULT_LEVEL_RENDER_CONFIG, ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { toLevelFrameOverlays } from "@/core/level/lib/render/level-render-frame";
import { toLevelCameraAt, toLevelViewOptions } from "@/core/level/lib/render/level-render-view";
import { ELevelSurfaceDressing, ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelSurfaceGeometry, ILevelSurfaceSpan } from "@/core/level/lib/surface/level-surface-geometry";
import { ILevelTextureProblem, ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { ILevelWeatherEffectRequest } from "@/core/level/lib/weather/level-weather-effect-request";
import { ILevelWeatherSeek } from "@/core/level/lib/weather/level-weather-seek";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelLookService } from "@/core/level/services/level-look.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { listenRenderClicks } from "@/core/render/lib/frame/render-clicks";
import { IRenderViewPoint } from "@/core/render/lib/frame/render-view-point";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { toNativeRenderHeight, toNativeWorldToggles } from "@/core/render/lib/native/native-view-options";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { toXraySpace } from "@/core/render/lib/scene/render-space";
import { SettingsService } from "@/core/settings/services/settings";
import { Logger } from "@/lib/logging";

/**
 * @param hit - What a native viewport named under a point.
 * @param spawn - The level's spawned objects as held, which a picked object is found among.
 * @returns It as the level names what is picked, in the level's own coordinates; null for an object not held.
 */
export function toLevelPick(hit: RenderLevelHit, spawn: Nullable<LevelSpawnObjectsDescription>): Nullable<TLevelPick> {
  const [x, y, z] = hit.point;
  const point: ILevelPoint = toXraySpace({ x: x ?? 0, y: y ?? 0, z: z ?? 0 });

  if (hit.kind === ERenderLevelHit.SPAWN) {
    const object: Maybe<LevelSpawnObject> = spawn?.objects.find((it: LevelSpawnObject) => it.index === hit.object);

    return object ? { kind: ELevelPick.SPAWN, object, point, visual: spawn?.visuals[object.visual] ?? "" } : null;
  }

  return {
    isImpostor: hit.isImpostor,
    kind: ELevelPick.SURFACE,
    mesh: hit.mesh,
    place: hit.place,
    point,
    sector: hit.sector,
    shaderId: hit.shaderId,
  };
}

/**
 * @param measured - What a native viewport counted each shader table entry of its level drawing.
 * @returns The same, keyed by shader id.
 */
export function toLevelSurfaceGeometry(measured: Array<WorldSurfaceGeometry>): Map<number, ILevelSurfaceGeometry> {
  return new Map(
    measured.map(({ shaderId, drawables, triangles, span, narrowest }: WorldSurfaceGeometry) => [
      shaderId,
      { drawables, narrowest: toLevelSurfaceSpan(narrowest), span: toLevelSurfaceSpan(span), triangles },
    ])
  );
}

function toLevelSurfaceSpan(span: Nullable<WorldSurfaceSpan>): Nullable<ILevelSurfaceSpan> {
  return span ? { uMax: span.uMax ?? 0, uMin: span.uMin ?? 0, vMax: span.vMax ?? 0, vMin: span.vMin ?? 0 } : null;
}

/**
 * @param described - What a native viewport said became of each texture its level samples.
 * @returns The same, as the level's panels read it.
 */
export function toLevelTextureReport(described: Array<RenderTextureReport>): ILevelTextureReport {
  const dressing: Map<string, ILevelSurfaceDressing> = new Map(
    described.map(({ reference, state }: RenderTextureReport) => [reference, toLevelSurfaceDressing(reference, state)])
  );
  const problems: Array<ILevelTextureProblem> = [];
  let uploaded: number = 0;

  dressing.forEach(({ reason, reference, state }: ILevelSurfaceDressing) => {
    if (reason) {
      problems.push({ reason, reference });
    }

    // A stand-in is uploaded as much as a file is; a load on its way is neither yet.
    if (state !== ELevelSurfaceDressing.FETCHING) {
      uploaded += 1;
    }
  });

  return { dressing, problems, uploaded };
}

function toLevelSurfaceDressing(reference: string, state: RenderTextureState): ILevelSurfaceDressing {
  switch (state.kind) {
    case ERenderTextureState.LOADING:
      return { reason: null, reference, state: ELevelSurfaceDressing.FETCHING, upload: null };

    case ERenderTextureState.LOADED: {
      const levels: string = `${state.levels} ${state.levels === 1 ? "level" : "levels"}`;
      const expanded: string = state.isExpanded ? " · expanded" : "";

      return {
        reason: null,
        reference,
        state: ELevelSurfaceDressing.UPLOADED,
        upload: `${state.width}×${state.height} · ${state.layout} · ${levels}${expanded}`,
      };
    }

    case ERenderTextureState.MISSING:
      return {
        reason: "Nothing in the mounted roots answers to it",
        reference,
        state: ELevelSurfaceDressing.STOOD_IN,
        upload: null,
      };

    default:
      return { reason: state.reason, reference, state: ELevelSurfaceDressing.STOOD_IN, upload: null };
  }
}

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
  /** Stops hearing clicks on the viewport and its focus, while one is attached. */
  private unlisten: Nullable<() => void> = null;

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly weatherService: LevelWeatherService = inject(LevelWeatherService),
    private readonly lookService: LevelLookService = inject(LevelLookService),
    settingsService: SettingsService = inject(SettingsService)
  ) {
    super(settingsService);
  }

  /**
   * The sun or moon the sky draws now, by its `suns.ltx` section, or null for neither: apart from the rest of what the
   * renderer applied, which changes with every report while the weather plays.
   */
  @Computed()
  public get drawnSun(): Nullable<string> {
    return this.applied?.sun ?? null;
  }

  /** What the level is asked to be drawn with: the toolbar, the look and the settings together. */
  @Computed()
  public get viewOptions(): RenderViewOptions {
    return toLevelViewOptions({
      features: this.settingsService.rendererFeatures,
      hemiStrength: this.viewService.hemiStrength,
      lod: this.viewService.lod,
      look: this.lookService.look,
      options: this.viewService.options,
      renderHeight: toNativeRenderHeight(this.settingsService.renderResolution),
      shading: this.viewService.shading,
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
   * @returns Settles once the pick is noted: what it hit, or nothing.
   */
  public async pick(point: IRenderViewPoint): Promise<void> {
    const { viewport, opening } = this;

    if (!viewport || !this.level) {
      return;
    }

    const hit: Nullable<RenderLevelHit> = await viewport.pick(point.x, point.y);

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

  protected start(viewport: NativeViewport): Array<() => void> {
    return [
      reaction(() => this.loadService.level.value?.selected ?? null, this.openLevel, { fireImmediately: true }),
      reaction(() => this.viewService.camera, this.applyCamera),
      reaction(
        () => this.loadService.level.value?.selected ?? null,
        (selected) => void this.lookService.open(selected),
        { fireImmediately: true }
      ),
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
   * The level's weather, read once it opens and played by the viewport, and how it plays, as either changes.
   *
   * @param viewport - The viewport it plays in.
   * @returns What stops watching.
   */
  private watchWeather(viewport: NativeViewport): Array<() => void> {
    const { weatherService } = this;

    return [
      reaction(
        () => this.loadService.level.value?.selected ?? null,
        (selected) => void weatherService.open(selected),
        { fireImmediately: true }
      ),
      reaction(
        () => weatherService.weather,
        (play: Nullable<WorldWeatherPlay>) =>
          viewport.playWeather(play ?? { kind: EWorldWeatherPlay.NONE }, weatherService.transition),
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
      reaction(
        () => weatherService.seek,
        (seek: Nullable<ILevelWeatherSeek>) => {
          if (seek) {
            viewport.seekWeather(seek.time);
          }
        }
      ),
      reaction(
        () => weatherService.effect,
        (effect: Nullable<ILevelWeatherEffectRequest>) => {
          if (effect) {
            viewport.playWeatherEffect(effect.name);
          }
        }
      ),
      reaction(
        () => weatherService.ambientPlays,
        (plays: number) => {
          if (plays > 0) {
            viewport.playAmbientEffect();
          }
        }
      ),
    ];
  }

  protected onAttached(container: HTMLElement): void {
    const unlistenClicks: () => void = listenRenderClicks(
      container,
      (point: IRenderViewPoint) => void this.pick(point)
    );
    const onFocus = (): void => this.viewportService.noteSceneFocused(true);
    const onBlur = (): void => this.viewportService.noteSceneFocused(false);

    container.addEventListener("focus", onFocus);
    container.addEventListener("blur", onBlur);

    this.unlisten = () => {
      unlistenClicks();
      container.removeEventListener("focus", onFocus);
      container.removeEventListener("blur", onBlur);
      this.viewportService.noteSceneFocused(false);
    };
  }

  protected onDetached(): void {
    this.unlisten?.();
    this.unlisten = null;
  }

  protected release(): void {
    this.opening += 1;
    this.level = null;
    this.viewpoint = null;
  }

  protected onCamera(pose: WorldCameraPose): void {
    this.viewportService.noteCamera(toLevelCameraReading(pose));
  }

  protected onWeather(report: Nullable<WorldWeatherReport>): void {
    this.weatherService.noteReport(report);
  }

  protected onLoad(report: RenderLoadReport): void {
    this.viewportService.noteLoad(report);

    // Revealed once everything it opens with is resident, so it is never seen half read.
    if (report.isReady && this.level) {
      this.viewportService.reveal();
      void this.describeResident();
    }
  }

  /**
   * Asks what each shader table entry draws, what each texture came to and what the level could not draw, once it is
   * resident whole.
   */
  private async describeResident(): Promise<void> {
    const { viewport, opening } = this;

    if (!viewport) {
      return;
    }

    const [measured, textures, problems] = await Promise.all([
      viewport.measureSurfaces(),
      viewport.describeTextures(),
      viewport.describeProblems(),
    ]);

    // Asked of a viewport or a level since replaced, it describes something else.
    if (viewport === this.viewport && opening === this.opening) {
      this.viewportService.noteSurfaceGeometry(toLevelSurfaceGeometry(measured));
      this.viewportService.noteTextures(toLevelTextureReport(textures));
      this.viewportService.noteProblems(problems);
    }
  }

  @BoundAction()
  private openLevel(selected: Nullable<SessionSnapshot<SelectedLevelDescription>>): void {
    const level: Nullable<SelectedLevelDescription> = selected?.value ?? null;

    this.opening += 1;
    this.level = level;
    this.stand(toLevelStartViewpoint(level?.bounds ?? null, level?.start ?? null));
    // Another level's surfaces and objects are numbered afresh.
    this.viewportService.notePicked(null);
    this.viewportService.conceal();
    this.viewport?.showLevel(selected?.sessionId ?? null);
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
    viewport?.setCamera(this.toCamera(viewpoint, this.viewService.camera));
    viewport?.commandCamera({ kind: EWorldCameraCommand.RESET });
  }

  private toCamera(viewpoint: ILevelViewpoint, options: ILevelCameraOptions): WorldCamera {
    return toLevelCameraAt(viewpoint, options, this.config);
  }
}
