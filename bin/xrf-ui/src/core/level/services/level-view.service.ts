import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable } from "@wirestate/mobx";

import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { DEFAULT_LEVEL_FEATURE_OPTIONS, ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { DEFAULT_LEVEL_LOD_OPTIONS, ILevelLodOptions } from "@/core/level/lib/lod/level-lod-options";
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

  /** How far trees are drawn in full before their impostors take over. */
  @RefObservable()
  public lod: ILevelLodOptions = DEFAULT_LEVEL_LOD_OPTIONS;

  /** What the view draws its shadows and antialiasing with, over the renderer's settings. */
  @RefObservable()
  public features: ILevelFeatureOptions = DEFAULT_LEVEL_FEATURE_OPTIONS;

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

  @BoundAction()
  public setLod(lod: ILevelLodOptions): void {
    this.lod = lod;
  }

  @BoundAction()
  public setFeatures(features: ILevelFeatureOptions): void {
    this.features = features;
  }

  /** Back to the defaults, so a viewer opened again does not inherit the last level's toggles. */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    this.options = DEFAULT_LEVEL_VIEW_OPTIONS;
    this.lighting = DEFAULT_LEVEL_LIGHTING;
    this.camera = DEFAULT_LEVEL_CAMERA_OPTIONS;
    this.lod = DEFAULT_LEVEL_LOD_OPTIONS;
    this.features = DEFAULT_LEVEL_FEATURE_OPTIONS;
  }
}
