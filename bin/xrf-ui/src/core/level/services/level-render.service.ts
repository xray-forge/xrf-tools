import { inject, Injectable } from "@wirestate/core";
import { BoundAction, reaction } from "@wirestate/mobx";
import {
  EMPTY_RENDER_FRAME_COST,
  EMPTY_RENDERER_LIGHTS_REPORT,
  EMPTY_RENDERER_STATIC_DRAW_REPORT,
  IRendererFlyCamera,
  IRendererViewPoint,
  IRenderFrameCost,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import {
  ERenderCamera,
  ERenderCameraCommand,
  RenderCamera,
  RenderCameraPose,
  RenderFrameReport,
} from "@/core/ipc/types/xrf-renderer";
import { ILevelGoTo, toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { DEFAULT_LEVEL_RENDER_CONFIG, ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { toLevelCameraAt } from "@/core/level/lib/render/level-render-view";
import { measureLevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { NativeRenderSurfaceService } from "@/core/render/lib/native/native-render-surface-service";
import { NativeViewport } from "@/core/render/lib/native/native-viewport";
import { SettingsService } from "@/core/settings/services/settings";
import { Logger } from "@/lib/logging";

/**
 * @param report - What a native viewport's recent frames cost.
 * @returns The same, as the level's readouts count a frame.
 */
export function toLevelFrameCost(report: RenderFrameReport): IRenderFrameCost {
  const cpuTime: number = report.cpuTime ?? 0;

  return {
    ...EMPTY_RENDER_FRAME_COST,
    drawnHeight: report.height,
    drawnWidth: report.width,
    drawTime: cpuTime,
    frameTime: report.frameTime ?? 0,
    framesPerSecond: report.framesPerSecond ?? 0,
    renderedHeight: report.height,
    renderedWidth: report.width,
    worstDrawTime: cpuTime,
    worstFrameTime: report.frameTimeMax ?? 0,
  };
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

  public constructor(
    private readonly loadService: LevelLoadService = inject(LevelLoadService),
    private readonly viewService: LevelViewService = inject(LevelViewService),
    private readonly viewportService: LevelViewportService = inject(LevelViewportService),
    settingsService: SettingsService = inject(SettingsService)
  ) {
    super(settingsService);
  }

  /**
   * @returns What each shader table entry draws across the sectors held.
   */
  public measureSurfaceGeometry(): ReadonlyMap<number, ILevelSurfaceGeometry> {
    // todo: Measure the shader table entries the native scene holds, once it holds the level's sectors.
    return new Map();
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
   * @returns Settles once the pick is noted.
   */
  public async pick(point: IRendererViewPoint): Promise<void> {
    // todo: Pick through the native viewport once it draws the level's surfaces and spawned objects.
    this.log.info("Picking is not drawn natively yet:", point);
  }

  protected start(): Array<() => void> {
    return [
      reaction(() => this.loadService.level.value?.selected.value ?? null, this.openLevel, { fireImmediately: true }),
      reaction(() => this.viewService.camera, this.applyCamera),
    ];
  }

  protected release(): void {
    this.level = null;
    this.viewpoint = null;
    this.pose = null;
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

  /** What the frames cost and where the camera is, for the readouts. */
  private publish(): void {
    const pose: Nullable<RenderCameraPose> = this.pose ?? this.toViewpointPose();

    if (!pose) {
      return;
    }

    this.viewportService.report(
      measureLevelStats(
        { bytes: 0, sectors: 0 },
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
  private openLevel(level: Nullable<SelectedLevelDescription>): void {
    this.level = level;
    this.stand(toLevelStartViewpoint(level?.bounds ?? null, level?.start ?? null));
    // Another level's surfaces and objects are numbered afresh.
    this.viewportService.notePicked(null);

    // todo: Hand the open level to the native viewport and reveal it once its scene is resident.
    if (level) {
      this.viewportService.reveal();
    } else {
      this.viewportService.conceal();
    }
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
