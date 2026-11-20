import { BoundAction, makeObservable, reaction } from "@wirestate/mobx";
import { EWorldCameraCommand, WorldCamera } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

import { ILevelGoTo, toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions, stepLevelCameraSpeed } from "@/core/level/lib/camera/level-camera-options";
import { ILevelViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { toLevelCameraAt } from "@/core/level/lib/render/level-render-view";
import { ILevelRenderContext } from "@/core/level/services/level-render/level-render.service.context";
import { LevelViewService } from "@/core/level/services/level-view";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";

/** What the camera is stood by. */
export interface ILevelRenderCameraSources {
  /** The toolbar's speeds and lens. */
  viewService: LevelViewService;
  /** The near and far planes. */
  config: ILevelRenderConfig;
  /** Where the camera stands first over what is open now: a level's start, or a scene framed. */
  toStart: () => ILevelViewpoint;
}

/**
 * Where the camera over a level or a scene stands: at its start as it opens, at a place gone to, and flown on from
 * there with the toolbar's speeds and lens.
 */
export class LevelRenderCamera {
  /** Where the camera was last stood: the start, or a place gone to. */
  private viewpoint: Nullable<ILevelViewpoint> = null;

  public constructor(
    private readonly context: Pick<ILevelRenderContext, "getViewport">,
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
   * Steps the toolbar's speed, as the wheel does while the secondary button flies the camera.
   *
   * @param steps - Whole steps, positive faster.
   */
  public stepSpeed(steps: number): void {
    const options: ILevelCameraOptions = this.sources.viewService.cameraOptions;

    this.sources.viewService.setCameraOptions({ ...options, speed: stepLevelCameraSpeed(options.speed, steps) });
  }

  /** Stands the camera at the start of what is open, anew. */
  public standAtStart(): void {
    this.stand(this.sources.toStart());
  }

  /** Forgets where the camera stood, with the viewport it stood in. */
  public release(): void {
    this.viewpoint = null;
  }

  /** The toolbar's speeds and lens, from the same place: the camera keeps where it has flown. */
  @BoundAction()
  private applyOptions(options: ILevelCameraOptions): void {
    this.context.getViewport()?.setCamera(this.toCamera(this.viewpoint ?? this.sources.toStart(), options));
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

  private toCamera(viewpoint: ILevelViewpoint, options: ILevelCameraOptions): WorldCamera {
    return toLevelCameraAt(viewpoint, options, this.sources.config);
  }
}
