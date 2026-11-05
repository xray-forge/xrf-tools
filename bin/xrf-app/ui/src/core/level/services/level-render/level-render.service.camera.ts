import { BoundAction, makeObservable, reaction } from "@wirestate/mobx";
import { EWorldCameraCommand, WorldCamera } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

import { ILevelGoTo, toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions, stepLevelCameraSpeed } from "@/core/level/lib/camera/level-camera-options";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { toLevelCameraAt } from "@/core/level/lib/render/level-render-view";
import { LevelLoadService } from "@/core/level/services/level-load";
import { ILevelRenderContext } from "@/core/level/services/level-render/level-render.service.context";
import { LevelViewService } from "@/core/level/services/level-view";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

/** What the level's camera is stood by. */
export interface ILevelRenderCameraSources {
  /** The open level's start, where the camera stands first. */
  loadService: LevelLoadService;
  /** The toolbar's speeds and lens. */
  viewService: LevelViewService;
  /** The near and far planes. */
  config: ILevelRenderConfig;
}

/**
 * Where the level's camera stands: at the level's start as it opens, at a place gone to, and flown on from there with
 * the toolbar's speeds and lens.
 */
export class LevelRenderCamera {
  /**
   * Where the camera was last stood: the level's start, or a place gone to. The toolbar's speeds and lens are sent
   * with it, so changing one leaves the camera where it has flown rather than taking it back.
   */
  private viewpoint: Nullable<ILevelViewpoint> = null;

  /**
   * @param context - The viewport and the level open.
   * @param sources - What the camera is stood by.
   */
  public constructor(
    private readonly context: ILevelRenderContext,
    private readonly sources: ILevelRenderCameraSources
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
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
   * Sends the toolbar's speeds and lens to a viewport just attached as they change.
   *
   * @returns What stops sending them.
   */
  public start(): Array<() => void> {
    return [reaction(() => this.sources.viewService.cameraOptions, this.applyOptions)];
  }

  /**
   * Steps the toolbar's speed, as the wheel does while the secondary button flies the camera; the camera keeps where it
   * has flown.
   *
   * @param steps - Whole steps, positive faster.
   */
  public stepSpeed(steps: number): void {
    const options: ILevelCameraOptions = this.sources.viewService.cameraOptions;

    this.sources.viewService.setCameraOptions({ ...options, speed: stepLevelCameraSpeed(options.speed, steps) });
  }

  /** Stands the camera at the open level's start, anew. */
  public standAtStart(): void {
    this.stand(this.toStart());
  }

  /** Forgets where the camera stood, with the viewport it stood in. */
  public release(): void {
    this.viewpoint = null;
  }

  /** The toolbar's speeds and lens, from the same place: the camera keeps where it has flown. */
  @BoundAction()
  private applyOptions(options: ILevelCameraOptions): void {
    this.context.getViewport()?.setCamera(this.toCamera(this.viewpoint ?? this.toStart(), options));
  }

  /**
   * Stands the camera at a place, anew: the same place again moves nothing a description compares, and the camera has
   * flown on since.
   *
   * @param viewpoint - Where it stands and what it looks at.
   */
  private stand(viewpoint: ILevelViewpoint): void {
    const viewport: Nullable<NativeViewport> = this.context.getViewport();

    this.viewpoint = viewpoint;
    viewport?.setCamera(this.toCamera(viewpoint, this.sources.viewService.cameraOptions));
    viewport?.commandCamera({ kind: EWorldCameraCommand.RESET });
  }

  private toStart(): ILevelViewpoint {
    return toLevelStartViewpoint(this.context.getLevel()?.bounds ?? null, this.sources.loadService.start.value);
  }

  private toCamera(viewpoint: ILevelViewpoint, options: ILevelCameraOptions): WorldCamera {
    return toLevelCameraAt(viewpoint, options, this.sources.config);
  }
}
