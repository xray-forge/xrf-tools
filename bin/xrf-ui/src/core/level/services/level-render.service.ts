import { CommandBus, inject, Injectable } from "@wirestate/core";
import { BoundAction, comparer, reaction } from "@wirestate/mobx";
import {
  EMPTY_RENDER_FRAME_COST,
  EMPTY_RENDERER_LIGHTS_REPORT,
  EMPTY_RENDERER_STATIC_DRAW_REPORT,
  ERendererAmbientOcclusionQuality,
  ERendererLightShadowFilter,
  IRendererFlyCamera,
  IRendererSettings,
  IRendererViewPoint,
  IRenderFrameCost,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import {
  ERenderAmbientOcclusionQuality,
  ERenderCamera,
  ERenderCameraCommand,
  ERenderLightShadowFilter,
  ERenderTextureState,
  ERenderWeatherPlay,
  RenderCamera,
  RenderCameraPose,
  RenderFrameReport,
  RenderLevelHit,
  RenderLoadReport,
  RenderSurfaceGeometry,
  RenderSurfaceSpan,
  RenderTextureReport,
  RenderTextureState,
  RenderViewOptions,
  RenderWeatherControl,
  RenderWeatherPlay,
  RenderWeatherReport,
} from "@/core/ipc/types/xrf-renderer";
import { ILevelGoTo, toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { LEVEL_PICK_PANELS } from "@/core/level/lib/panels/level-pick-panels";
import { ELevelPick, TLevelPick } from "@/core/level/lib/pick/level-pick";
import { DEFAULT_LEVEL_RENDER_CONFIG, ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { toLevelCameraAt, toLevelRendererSettings } from "@/core/level/lib/render/level-render-view";
import { measureLevelStats } from "@/core/level/lib/stats/level-stats";
import { ELevelSurfaceDressing, ILevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ILevelSurfaceGeometry, ILevelSurfaceSpan } from "@/core/level/lib/surface/level-surface-geometry";
import { ILevelTextureProblem, ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelWeatherEffectRequest } from "@/core/level/lib/weather/level-weather-effect-request";
import { ILevelWeatherSeek } from "@/core/level/lib/weather/level-weather-seek";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { listenRenderClicks } from "@/core/render/lib/frame/render-clicks";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { toXraySpace } from "@/core/render/lib/scene/render-space";
import { SettingsService } from "@/core/settings/services/settings";
import { IPanelSetActiveCommand, PANEL_SET_ACTIVE_COMMAND } from "@/core/shell/panel/panel-messages";
import { Logger } from "@/lib/logging";

/** The settings' occlusion qualities as the native renderer names them. */
const AMBIENT_OCCLUSION_QUALITIES: Record<ERendererAmbientOcclusionQuality, ERenderAmbientOcclusionQuality> = {
  [ERendererAmbientOcclusionQuality.LOW]: ERenderAmbientOcclusionQuality.LOW,
  [ERendererAmbientOcclusionQuality.MEDIUM]: ERenderAmbientOcclusionQuality.MEDIUM,
  [ERendererAmbientOcclusionQuality.HIGH]: ERenderAmbientOcclusionQuality.HIGH,
  [ERendererAmbientOcclusionQuality.ULTRA]: ERenderAmbientOcclusionQuality.ULTRA,
};

/** The settings' light shadow filters as the native renderer names them. */
const LIGHT_SHADOW_FILTERS: Record<ERendererLightShadowFilter, ERenderLightShadowFilter> = {
  [ERendererLightShadowFilter.ENGINE]: ERenderLightShadowFilter.ENGINE,
  [ERendererLightShadowFilter.SOFT]: ERenderLightShadowFilter.SOFT,
};

/**
 * @param report - What a native viewport's recent frames cost.
 * @returns The same, as the level's readouts count a frame: each visible cluster a draw.
 */
export function toLevelFrameCost(report: RenderFrameReport): IRenderFrameCost {
  const cpuTime: number = report.cpuTime ?? 0;

  return {
    ...EMPTY_RENDER_FRAME_COST,
    draws: report.clusters,
    drawnHeight: report.height,
    drawnWidth: report.width,
    drawTime: cpuTime,
    frameTime: report.frameTime ?? 0,
    framesPerSecond: report.framesPerSecond ?? 0,
    renderedHeight: report.height,
    renderedWidth: report.width,
    triangles: report.triangles,
    worstDrawTime: cpuTime,
    worstFrameTime: report.frameTimeMax ?? 0,
  };
}

/** What the toolbar shows of the weather. */
export type TLevelWeatherSwitches = Pick<
  ILevelViewOptions,
  "isClouded" | "isFogged" | "isRainy" | "isSkyHazed" | "isSkyVisible" | "isThundering" | "isWaterVisible" | "isWindy"
>;

/**
 * @param settings - What the level's toolbar and the application's settings come to.
 * @param switches - What the toolbar shows of the weather.
 * @returns What a native viewport draws the level with.
 */
export function toLevelViewOptions(settings: IRendererSettings, switches: TLevelWeatherSwitches): RenderViewOptions {
  const { ambientOcclusion, exposure, lights, shadows, water } = settings.features;

  return {
    ambientOcclusion: {
      isEnabled: ambientOcclusion.isEnabled,
      quality: AMBIENT_OCCLUSION_QUALITIES[ambientOcclusion.quality],
      radius: ambientOcclusion.radius,
      strength: ambientOcclusion.strength,
    },
    exposure: {
      adaptation: exposure.adaptation,
      amount: exposure.amount,
      isEnabled: exposure.isEnabled,
      lowLuminance: exposure.lowLuminance,
      middleGray: exposure.middleGray,
    },
    geometryLod: settings.features.lod.geometryLod,
    hemiStrength: settings.hemiStrength,
    isBumped: settings.isBumped,
    isClouded: switches.isClouded,
    isFogged: switches.isFogged,
    isImpostors: settings.features.lod.isImpostors,
    isLit: settings.isLit,
    isOcclusionCulled: settings.features.isOcclusionCulled,
    isRainy: switches.isRainy,
    isSkyHazed: switches.isSkyHazed,
    isSkyVisible: switches.isSkyVisible,
    isTextured: settings.isTextured,
    isThundering: switches.isThundering,
    isWindy: switches.isWindy,
    lights: {
      isEnabled: lights.isEnabled,
      isLevelLights: lights.isLevelLights,
      isShadowed: lights.isShadowed,
      shadowFilter: LIGHT_SHADOW_FILTERS[lights.shadowFilter],
    },
    shadows: {
      bias: shadows.bias,
      blend: shadows.blend,
      cascades: [...shadows.cascades],
      filter: shadows.filter,
      isEnabled: shadows.isEnabled,
      isStaggered: shadows.isStaggered,
      reach: shadows.reach,
      resolution: shadows.resolution,
    },
    tonemapScale: settings.tonemapScale,
    water: {
      distortion: water.distortion,
      isDistorted: water.isDistorted,
      isEnabled: water.isEnabled && switches.isWaterVisible,
      isSoft: water.isSoft,
      reflection: water.reflection,
      ripple: water.ripple,
      waveHeight: water.waveHeight,
      waveSpeed: water.waveSpeed,
    },
  };
}

/**
 * @param hit - What a native viewport named under a point.
 * @returns It as the level names what is picked, in the level's own coordinates.
 */
export function toLevelPick(hit: RenderLevelHit): TLevelPick {
  const [x, y, z] = hit.point;

  return {
    isImpostor: hit.isImpostor,
    kind: ELevelPick.SURFACE,
    mesh: hit.mesh,
    place: hit.place,
    point: toXraySpace({ x: x ?? 0, y: y ?? 0, z: z ?? 0 }),
    sector: hit.sector,
    shaderId: hit.shaderId,
  };
}

/**
 * @param measured - What a native viewport counted each shader table entry of its level drawing.
 * @returns The same, keyed by shader id.
 */
export function toLevelSurfaceGeometry(measured: Array<RenderSurfaceGeometry>): Map<number, ILevelSurfaceGeometry> {
  return new Map(
    measured.map(({ shaderId, drawables, triangles, span, narrowest }: RenderSurfaceGeometry) => [
      shaderId,
      { drawables, narrowest: toLevelSurfaceSpan(narrowest), span: toLevelSurfaceSpan(span), triangles },
    ])
  );
}

function toLevelSurfaceSpan(span: Nullable<RenderSurfaceSpan>): Nullable<ILevelSurfaceSpan> {
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
  /** Where the camera stands, as the viewport last said. */
  private pose: Nullable<RenderCameraPose> = null;
  private frame: IRenderFrameCost = EMPTY_RENDER_FRAME_COST;
  /** What the viewport holds of the level, as it last said. */
  private load: Nullable<RenderLoadReport> = null;
  /** Bumped by every level opened or closed, so a pick asked of one since replaced notes nothing. */
  private opening: number = 0;
  /** Stops hearing clicks on the viewport, while one is attached. */
  private unlistenClicks: Nullable<() => void> = null;

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    private readonly weatherService: LevelWeatherService = inject(LevelWeatherService),
    private readonly commandBus: CommandBus = inject(CommandBus),
    settingsService: SettingsService = inject(SettingsService)
  ) {
    super(settingsService);
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
   * Says what of the open level is drawn under a point of the viewport, and opens the panel it is chosen in.
   *
   * @param point - Where, in css pixels from the viewport's top left corner.
   * @returns Settles once the pick is noted: what it hit, or nothing.
   */
  public async pick(point: IRendererViewPoint): Promise<void> {
    const { viewport, opening } = this;

    if (!viewport || !this.level) {
      return;
    }

    const hit: Nullable<RenderLevelHit> = await viewport.pick(point.x, point.y);

    // Asked of a viewport or a level since replaced, it names something else now.
    if (viewport !== this.viewport || opening !== this.opening) {
      return;
    }

    const picked: Nullable<TLevelPick> = hit ? toLevelPick(hit) : null;

    this.viewportService.notePicked(picked);

    // Opened here rather than as a view reacts: the panel mounts synchronously, which it cannot do mid-render.
    if (picked) {
      this.commandBus.execute<void, IPanelSetActiveCommand>(PANEL_SET_ACTIVE_COMMAND, LEVEL_PICK_PANELS[picked.kind], {
        optional: true,
      });
    }
  }

  protected start(viewport: NativeViewport): Array<() => void> {
    return [
      reaction(() => this.loadService.level.value?.selected ?? null, this.openLevel, { fireImmediately: true }),
      reaction(() => this.viewService.camera, this.applyCamera),
      reaction(
        () => toLevelViewOptions(this.toSettings(), this.viewService.options),
        (options: RenderViewOptions) => viewport.setViewOptions(options),
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
        (play: Nullable<RenderWeatherPlay>) =>
          viewport.playWeather(play ?? { kind: ERenderWeatherPlay.NONE }, weatherService.transition),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        (): RenderWeatherControl => ({
          ...weatherService.control,
          // The keyframe set by hand stands its sun by its own angles.
          isDynamicSun: weatherService.control.isDynamicSun && !weatherService.isManual,
        }),
        (control: RenderWeatherControl) => viewport.setWeatherControl(control),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        () => weatherService.seek,
        (seek: Nullable<ILevelWeatherSeek>) => seek && viewport.seekWeather(seek.time)
      ),
      reaction(
        () => weatherService.effect,
        (effect: Nullable<ILevelWeatherEffectRequest>) => effect && viewport.playWeatherEffect(effect.name)
      ),
    ];
  }

  protected onAttached(container: HTMLElement): void {
    this.unlistenClicks = listenRenderClicks(container, (point: IRendererViewPoint) => void this.pick(point));
  }

  protected onDetached(): void {
    this.unlistenClicks?.();
    this.unlistenClicks = null;
  }

  protected release(): void {
    this.opening += 1;
    this.level = null;
    this.viewpoint = null;
    this.pose = null;
    this.load = null;
    this.frame = EMPTY_RENDER_FRAME_COST;
  }

  protected onFrame(report: RenderFrameReport): void {
    this.frame = toLevelFrameCost(report);
    this.publish();
  }

  protected onCamera(pose: RenderCameraPose): void {
    this.pose = pose;
    this.publish();
  }

  protected onWeather(report: Nullable<RenderWeatherReport>): void {
    this.weatherService.noteReport(report);
  }

  protected onLoad(report: RenderLoadReport): void {
    this.load = report;

    // Revealed once everything it opens with is resident, so it is never seen half read.
    if (report.isReady && this.level) {
      this.viewportService.reveal();
      void this.describeResident();
    }
  }

  /** Asks what each shader table entry draws and what each texture came to, once the level is resident whole. */
  private async describeResident(): Promise<void> {
    const { viewport, opening } = this;

    if (!viewport) {
      return;
    }

    const [measured, textures] = await Promise.all([viewport.measureSurfaces(), viewport.describeTextures()]);

    // Asked of a viewport or a level since replaced, it describes something else.
    if (viewport === this.viewport && opening === this.opening) {
      this.viewportService.noteSurfaceGeometry(toLevelSurfaceGeometry(measured));
      this.viewportService.noteTextures(toLevelTextureReport(textures));
    }
  }

  /** What the level's toolbar and the application's settings come to. */
  private toSettings(): IRendererSettings {
    return toLevelRendererSettings({
      config: this.config,
      hemiStrength: this.viewService.hemiStrength,
      lod: this.viewService.lod,
      options: this.viewService.options,
      shared: this.settingsService.sharedRenderSettings,
      view: this.viewService.features,
    });
  }

  /** What the frames cost and where the camera is, for the readouts. */
  private publish(): void {
    const pose: Nullable<RenderCameraPose> = this.pose ?? this.toViewpointPose();

    if (!pose) {
      return;
    }

    this.viewportService.report(
      measureLevelStats(
        { bytes: this.load?.bytes ?? 0, sectors: this.load?.sectors ?? 0 },
        this.frame,
        0,
        EMPTY_RENDERER_STATIC_DRAW_REPORT,
        EMPTY_RENDERER_LIGHTS_REPORT,
        0
      ),
      toLevelCameraReading({
        position: [pose.position[0] ?? 0, pose.position[1] ?? 0, pose.position[2] ?? 0],
        target: [pose.target[0] ?? 0, pose.target[1] ?? 0, pose.target[2] ?? 0],
      })
    );
  }

  @BoundAction()
  private openLevel(selected: Nullable<SessionSnapshot<SelectedLevelDescription>>): void {
    const level: Nullable<SelectedLevelDescription> = selected?.value ?? null;

    this.opening += 1;
    this.level = level;
    this.load = null;
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
    viewport?.commandCamera({ kind: ERenderCameraCommand.RESET });
  }

  private toCamera(viewpoint: ILevelViewpoint, options: ILevelCameraOptions): RenderCamera {
    const camera: IRendererFlyCamera = toLevelCameraAt(viewpoint, options, this.config);

    return { ...camera, kind: ERenderCamera.FLY, position: [...camera.position], target: [...camera.target] };
  }

  private toViewpointPose(): Nullable<RenderCameraPose> {
    const viewpoint: Nullable<ILevelViewpoint> = this.viewpoint;

    return viewpoint
      ? {
          position: [viewpoint.position.x, viewpoint.position.y, viewpoint.position.z],
          target: [viewpoint.target.x, viewpoint.target.y, viewpoint.target.z],
        }
      : null;
  }
}
