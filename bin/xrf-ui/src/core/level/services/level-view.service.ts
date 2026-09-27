import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { ILevelGoTo } from "@/core/level/lib/camera/level-camera-goto";
import { ILevelCameraOptions, toLevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelFeatureOptions, toLevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { DEFAULT_LEVEL_LOD_OPTIONS, ILevelLodOptions } from "@/core/level/lib/lod/level-lod-options";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LEVEL_CAMERA_STORAGE_KEY, LEVEL_FEATURES_STORAGE_KEY } from "@/core/storage";
import { parseLocalStorageValueSafe, setLocalStorageValueSafe } from "@/lib/local-storage";

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

  /** What the camera sees and how it answers input: a preference, kept over runs. */
  @RefObservable()
  public camera: ILevelCameraOptions = toLevelCameraOptions(parseLocalStorageValueSafe(LEVEL_CAMERA_STORAGE_KEY));

  /** How far trees are drawn in full before their impostors take over. */
  @RefObservable()
  public lod: ILevelLodOptions = DEFAULT_LEVEL_LOD_OPTIONS;

  /**
   * What the view sets over the renderer's features for itself: antialiasing, shadows, occlusion, grass and lights. A
   * preference, kept over runs.
   */
  @RefObservable()
  public features: ILevelFeatureOptions = toLevelFeatureOptions(parseLocalStorageValueSafe(LEVEL_FEATURES_STORAGE_KEY));

  /** The last place asked to be gone to, a new object each time so the same place asked again is asked again. */
  @RefObservable()
  public goTo: Nullable<Readonly<ILevelGoTo>> = null;

  /**
   * @param goTo - Where the camera is asked to stand, as the readout states it.
   */
  @BoundAction()
  public requestGoTo(goTo: ILevelGoTo): void {
    this.goTo = { ...goTo };
  }

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
    setLocalStorageValueSafe(LEVEL_CAMERA_STORAGE_KEY, JSON.stringify(camera));
  }

  @BoundAction()
  public setLod(lod: ILevelLodOptions): void {
    this.lod = lod;
  }

  @BoundAction()
  public setFeatures(features: ILevelFeatureOptions): void {
    this.features = features;
    setLocalStorageValueSafe(LEVEL_FEATURES_STORAGE_KEY, JSON.stringify(features));
  }

  /**
   * Back to the defaults, so a viewer opened again does not inherit the last level's toggles; the camera and the
   * features are preferences and stay.
   */
  @OnDeactivation()
  @BoundAction()
  public clear(): void {
    this.options = DEFAULT_LEVEL_VIEW_OPTIONS;
    this.lighting = DEFAULT_LEVEL_LIGHTING;
    this.lod = DEFAULT_LEVEL_LOD_OPTIONS;
    this.goTo = null;
  }
}
