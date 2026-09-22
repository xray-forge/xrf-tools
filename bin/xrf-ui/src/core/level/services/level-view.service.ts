import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable } from "@wirestate/mobx";

import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";

/**
 * How a level is drawn: what the viewer has switched on, rather than what the level is.
 */
@Injectable()
export class LevelViewService {
  /** What the toolbar has switched on. */
  @RefObservable()
  public options: ILevelViewOptions = DEFAULT_LEVEL_VIEW_OPTIONS;

  /** What the viewer is lighting with, which is its own answer rather than anything the level carries. */
  @RefObservable()
  public lighting: ILevelLighting = DEFAULT_LEVEL_LIGHTING;

  /** What the camera sees and how it answers input. */
  @RefObservable()
  public camera: ILevelCameraOptions = DEFAULT_LEVEL_CAMERA_OPTIONS;

  @BoundAction()
  public setOptions(options: ILevelViewOptions): void {
    this.options = options;
  }

  @BoundAction()
  public setLighting(lighting: ILevelLighting): void {
    this.lighting = lighting;
  }

  @BoundAction()
  public setCamera(camera: ILevelCameraOptions): void {
    this.camera = camera;
  }

  /** Back to the defaults, so a viewer opened again does not inherit the last level's toggles. */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    this.options = DEFAULT_LEVEL_VIEW_OPTIONS;
    this.lighting = DEFAULT_LEVEL_LIGHTING;
    this.camera = DEFAULT_LEVEL_CAMERA_OPTIONS;
  }
}
