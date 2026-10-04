import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, RefObservable } from "@wirestate/mobx";

import { ERenderDebugView } from "@/core/ipc/types/xrf-renderer";
import { ILevelCameraOptions, toLevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelFeatureOptions, toLevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_LEVEL_LOD_OPTIONS, ILevelLodOptions } from "@/core/level/lib/lod/level-lod-options";
import {
  DEFAULT_LEVEL_HEMI_STRENGTH,
  DEFAULT_LEVEL_VIEW_OPTIONS,
  ILevelViewOptions,
} from "@/core/level/lib/view/level-view-options";
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

  /** How much the baked hemisphere term darkens the ambient, `0` ignoring it and `1` applying it whole. */
  @RefObservable()
  public hemiStrength: number = DEFAULT_LEVEL_HEMI_STRENGTH;

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

  /** Which picture the viewport shows: the frame, or one of the targets it was built from. */
  @RefObservable()
  public debugView: ERenderDebugView = ERenderDebugView.FINAL;

  @BoundAction()
  public setOptions(options: ILevelViewOptions): void {
    this.options = options;
  }

  @BoundAction()
  public setHemiStrength(hemiStrength: number): void {
    this.hemiStrength = hemiStrength;
  }

  @BoundAction()
  public setCamera(camera: ILevelCameraOptions): void {
    this.camera = camera;
    setLocalStorageValueSafe(LEVEL_CAMERA_STORAGE_KEY, JSON.stringify(camera));
  }

  @BoundAction()
  public setDebugView(debugView: ERenderDebugView): void {
    this.debugView = debugView;
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
    this.hemiStrength = DEFAULT_LEVEL_HEMI_STRENGTH;
    this.lod = DEFAULT_LEVEL_LOD_OPTIONS;
    this.debugView = ERenderDebugView.FINAL;
  }
}
