import { Injectable, OnDeactivation } from "@wirestate/core";
import { BoundAction, Computed, RefObservable } from "@wirestate/mobx";
import { LevelObjectKind } from "@xrf/ipc/types/xrf-app-levels";
import { Logger } from "@xrf/logging";

import { KeybindCommand } from "@/core/commands";
import { TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND } from "@/core/level/commands";
import { ILevelCameraOptions, toLevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelFeatureOptions, toLevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_LEVEL_LOD_OPTIONS, ILevelLodOptions } from "@/core/level/lib/lod/level-lod-options";
import {
  ILevelHelperKindsChange,
  ILevelHelperOptions,
  toLevelHelperOptions,
} from "@/core/level/lib/objects/level-helper-options";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import {
  DEFAULT_LEVEL_HEMI_STRENGTH,
  DEFAULT_LEVEL_VIEW_OPTIONS,
  ILevelViewOptions,
} from "@/core/level/lib/view/level-view-options";
import { listUnkeptRenderSettings } from "@/core/render/lib/settings/render-feature-choice";
import {
  jsonCodec,
  LEVEL_CAMERA_STORAGE_KEY,
  LEVEL_FEATURES_STORAGE_KEY,
  LEVEL_HELPERS_STORAGE_KEY,
  StoredValue,
} from "@/core/storage";

/**
 * How a level is drawn: what the viewer has switched on, rather than what the level is.
 */
@Injectable()
export class LevelViewService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** What the toolbar has switched on. */
  @RefObservable()
  public options: ILevelViewOptions = DEFAULT_LEVEL_VIEW_OPTIONS;

  /** How much the baked hemisphere term darkens the ambient, `0` ignoring it and `1` applying it whole. */
  @RefObservable()
  public hemiStrength: number = DEFAULT_LEVEL_HEMI_STRENGTH;

  /** How far trees are drawn in full before their impostors take over. */
  @RefObservable()
  public lod: ILevelLodOptions = DEFAULT_LEVEL_LOD_OPTIONS;

  /** What the viewport shows of its surfaces: the frame as dressed, as clay or by shader, or one of its targets. */
  @RefObservable()
  public shading: ELevelShading = ELevelShading.FINAL;

  private readonly storedCameraOptions: StoredValue<ILevelCameraOptions> = new StoredValue(
    LEVEL_CAMERA_STORAGE_KEY,
    jsonCodec(toLevelCameraOptions)
  );

  private readonly storedFeatures: StoredValue<ILevelFeatureOptions> = new StoredValue(
    LEVEL_FEATURES_STORAGE_KEY,
    jsonCodec(LevelViewService.readFeatures)
  );

  private readonly storedHelpers: StoredValue<ILevelHelperOptions> = new StoredValue(
    LEVEL_HELPERS_STORAGE_KEY,
    jsonCodec(toLevelHelperOptions)
  );

  /** What the camera sees and how it answers input: a preference, kept over runs. */
  @Computed()
  public get cameraOptions(): ILevelCameraOptions {
    return this.storedCameraOptions.value;
  }

  /** Which helpers are drawn over the level objects: a preference, kept over runs. */
  @Computed()
  public get helpers(): ILevelHelperOptions {
    return this.storedHelpers.value;
  }

  /**
   * @param helpers - Which helpers are drawn from now on, and how, as a view's snapshot gives them back.
   */
  @BoundAction()
  public setHelpers(helpers: ILevelHelperOptions): void {
    this.storedHelpers.set(helpers);
  }

  /** Draws the helpers or leaves them all out, as `G` does, each kind keeping its choice. */
  @KeybindCommand(TOGGLE_LEVEL_HELPERS_KEYBIND_COMMAND)
  @BoundAction()
  public toggleHelpers(): void {
    this.storedHelpers.set({ ...this.helpers, isShown: !this.helpers.isShown });
  }

  /**
   * @param helpers - How the helpers are drawn from now on, the switches aside.
   */
  @BoundAction()
  public setHelperLook(
    helpers: Partial<
      Pick<ILevelHelperOptions, "distanceScale" | "isRestFaded" | "isScreenshotHelped" | "occludedStrength">
    >
  ): void {
    this.storedHelpers.set({ ...this.helpers, ...helpers });
  }

  /**
   * @param change - Kinds of level object, and whether their helpers are drawn from now on, while helpers are.
   */
  @BoundAction()
  public setHelperKinds(change: ILevelHelperKindsChange): void {
    const { kinds, isOn } = change;

    this.storedHelpers.set({
      ...this.helpers,
      kinds: { ...this.helpers.kinds, ...Object.fromEntries(kinds.map((kind: LevelObjectKind) => [kind, isOn])) },
    });
  }

  /**
   * What the view sets over the renderer's features for itself: antialiasing, shadows, occlusion, grass and lights. A
   * preference, kept over runs.
   */
  @Computed()
  public get features(): ILevelFeatureOptions {
    return this.storedFeatures.value;
  }

  @BoundAction()
  public setOptions(options: ILevelViewOptions): void {
    this.options = options;
  }

  /**
   * @param option - One of the toolbar's switches.
   * @param isOn - Whether it is on from now on.
   */
  @BoundAction()
  public setOption(option: keyof ILevelViewOptions, isOn: boolean): void {
    this.options = { ...this.options, [option]: isOn };
  }

  /**
   * @param option - One of the toolbar's switches, turned over.
   */
  @BoundAction()
  public toggleOption(option: keyof ILevelViewOptions): void {
    this.setOption(option, !this.options[option]);
  }

  @BoundAction()
  public setHemiStrength(hemiStrength: number): void {
    this.hemiStrength = hemiStrength;
  }

  @BoundAction()
  public setCameraOptions(cameraOptions: ILevelCameraOptions): void {
    this.storedCameraOptions.set(cameraOptions);
  }

  @BoundAction()
  public setShading(shading: ELevelShading): void {
    this.shading = shading;
  }

  @BoundAction()
  public setLod(lod: ILevelLodOptions): void {
    this.lod = lod;
  }

  /**
   * @param features - What the view sets, held as a stored choice is read back: a setting the schema does not take is
   *   dropped, with a warning, rather than kept until the next start.
   */
  @BoundAction()
  public setFeatures(features: ILevelFeatureOptions): void {
    const kept: ILevelFeatureOptions = toLevelFeatureOptions(features);
    const unkept: Array<string> = listUnkeptRenderSettings(features, kept);

    if (unkept.length) {
      this.log.warn("View features the schema does not take, left to the settings:", unkept);
    }

    this.storedFeatures.set(kept);
  }

  /**
   * @param keys - The features the view leaves to the settings again, as a popover's reset leaves its own; the rest
   *   stay as the view sets them.
   */
  @BoundAction()
  public resetFeatures(keys: ReadonlyArray<keyof ILevelFeatureOptions>): void {
    const unset: ILevelFeatureOptions = toLevelFeatureOptions(null);

    this.setFeatures({
      ...this.features,
      ...Object.fromEntries(keys.map((key: keyof ILevelFeatureOptions) => [key, unset[key]])),
    });
  }

  /**
   * @param stored - The parsed stored features, `null` when none were stored.
   * @returns The stored features, or none where none were stored; a setting the schema does not take is dropped.
   */
  private static readFeatures(stored: unknown): ILevelFeatureOptions {
    const features: ILevelFeatureOptions = toLevelFeatureOptions(stored);
    const unkept: Array<string> = listUnkeptRenderSettings(stored, features);

    if (unkept.length) {
      Logger.warn("Stored view features the schema does not take, left to the settings:", unkept);
    }

    return features;
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
    this.shading = ELevelShading.FINAL;
  }
}
