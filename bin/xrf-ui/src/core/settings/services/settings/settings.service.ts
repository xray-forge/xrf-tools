import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed, Observable, RefObservable } from "@wirestate/mobx";
import {
  ERendererPreset,
  ERenderResolution,
  IRendererFeatureChoice,
  IRendererFeatureOverrides,
  IRendererFeatureSettings,
  mergeRendererFeatureOverrides,
  resolveRendererFeatures,
  TFrameRateLimit,
  toFrameRateLimit,
  toRendererFeatureChoice,
  toRenderResolution,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { IRenderSharedSettings } from "@/core/render/lib/settings/render-shared-settings";
import { TCatalogView, toCatalogView } from "@/core/settings/lib/catalog-view";
import { toXrayEngine } from "@/core/settings/lib/xray-engine";
import {
  CATALOG_VIEW_STORAGE_KEY,
  DEV_MODE_STORAGE_KEY,
  ENGINE_STORAGE_KEY,
  FRAME_RATE_LIMIT_STORAGE_KEY,
  GPU_TIMED_STORAGE_KEY,
  LOW_LATENCY_STORAGE_KEY,
  RENDER_RESOLUTION_STORAGE_KEY,
  RENDERER_FEATURES_STORAGE_KEY,
} from "@/core/storage";
import { isDevelopmentBuild } from "@/lib/env";
import {
  getLocalStorageValue,
  parseLocalStorageValueSafe,
  setLocalStorageValue,
  setLocalStorageValueSafe,
} from "@/lib/local-storage";
import { Logger } from "@/lib/logging";

/**
 * Application wide switches that are not tied to any one editor.
 */
@Injectable()
export class SettingsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  /** Surfaces dev traces and captured runtime errors that are otherwise hidden. */
  @Observable()
  public isDevModeEnabled: boolean = SettingsService.readDevModeEnabled();

  /** How the root catalog draws its tools. */
  @Observable()
  public catalogView: TCatalogView = toCatalogView(getLocalStorageValue(CATALOG_VIEW_STORAGE_KEY));

  /** Which engine game configs are read as, where the engines read them differently: every tool reading them asks. */
  @Observable()
  public engine: EXrayEngine = toXrayEngine(getLocalStorageValue(ENGINE_STORAGE_KEY));

  /**
   * Frames a second every viewport is allowed to draw.
   */
  @Observable()
  public frameRateLimit: TFrameRateLimit = toFrameRateLimit(getLocalStorageValue(FRAME_RATE_LIMIT_STORAGE_KEY));

  /** Whether frames wait for the GPU to be at most a frame behind, answering input sooner for fewer frames. */
  @Observable()
  public isLowLatency: boolean = getLocalStorageValue(LOW_LATENCY_STORAGE_KEY) !== String(false);

  /** Whether every viewport times its passes on the GPU: never a preset's, as it costs a frame 1-4% of its rate. */
  @Observable()
  public isGpuTimed: boolean = getLocalStorageValue(GPU_TIMED_STORAGE_KEY) === String(true);

  @Observable()
  public renderResolution: ERenderResolution = toRenderResolution(getLocalStorageValue(RENDER_RESOLUTION_STORAGE_KEY));

  /**
   * The renderer's preset and what was changed on top of it, the same for every viewport. Held by reference, so what
   * it resolves to crosses to the renderer's thread as plain data.
   */
  @RefObservable()
  public rendererChoice: IRendererFeatureChoice = SettingsService.readRendererChoice();

  /** Every renderer feature as the choice sets it. */
  @Computed()
  public get rendererFeatures(): IRendererFeatureSettings {
    return resolveRendererFeatures(this.rendererChoice);
  }

  /** What every viewport draws with alike: its pacing, its timing and its features. */
  @Computed()
  public get sharedRenderSettings(): IRenderSharedSettings {
    return {
      features: this.rendererFeatures,
      isGpuTimed: this.isGpuTimed,
      pacing: { isLowLatency: this.isLowLatency, rateLimit: this.frameRateLimit },
    };
  }

  /**
   * @returns The stored choice, or the default where none was stored or it does not parse.
   */
  private static readRendererChoice(): IRendererFeatureChoice {
    return toRendererFeatureChoice(parseLocalStorageValueSafe(RENDERER_FEATURES_STORAGE_KEY));
  }

  /**
   * @returns The stored choice, or whether this is a development build when there is none.
   */
  private static readDevModeEnabled(): boolean {
    const stored: Nullable<string> = getLocalStorageValue(DEV_MODE_STORAGE_KEY);

    return stored === null ? isDevelopmentBuild() : stored === String(true);
  }

  @OnProvision()
  public async onProvision(provisionId: ProvisionId): Promise<void> {
    this.log.info("Provisioning:", provisionId);
  }

  @OnDeprovision()
  public onDeprovision(provisionId: ProvisionId): void {
    this.log.info("Deprovisioning:", provisionId);
  }

  @BoundAction()
  public setDevModeEnabled(isEnabled: boolean): void {
    this.log.info("Set dev mode:", isEnabled);

    this.isDevModeEnabled = isEnabled;
    setLocalStorageValue(DEV_MODE_STORAGE_KEY, String(isEnabled));
  }

  @BoundAction()
  public setEngine(engine: EXrayEngine): void {
    this.log.info("Set engine:", engine);

    this.engine = engine;
    setLocalStorageValue(ENGINE_STORAGE_KEY, engine);
  }

  @BoundAction()
  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.log.info("Set frame rate limit:", limit);

    this.frameRateLimit = limit;
    setLocalStorageValue(FRAME_RATE_LIMIT_STORAGE_KEY, limit);
  }

  @BoundAction()
  public setLowLatency(isLowLatency: boolean): void {
    this.log.info("Set low latency:", isLowLatency);

    this.isLowLatency = isLowLatency;
    setLocalStorageValue(LOW_LATENCY_STORAGE_KEY, String(isLowLatency));
  }

  @BoundAction()
  public setGpuTimed(isGpuTimed: boolean): void {
    this.log.info("Set GPU timing:", isGpuTimed);

    this.isGpuTimed = isGpuTimed;
    setLocalStorageValue(GPU_TIMED_STORAGE_KEY, String(isGpuTimed));
  }

  @BoundAction()
  public setRenderResolution(resolution: ERenderResolution): void {
    this.log.info("Set render resolution:", resolution);

    this.renderResolution = resolution;
    setLocalStorageValue(RENDER_RESOLUTION_STORAGE_KEY, resolution);
  }

  /**
   * @param preset - The preset every feature follows from now on, whatever was changed on top of the last one.
   */
  @BoundAction()
  public setRendererPreset(preset: ERendererPreset): void {
    this.log.info("Set renderer preset:", preset);

    this.storeRendererChoice({ overrides: {}, preset });
  }

  /**
   * @param overrides - What changes on top of the preset, merged over what already did.
   */
  @BoundAction()
  public setRendererOverrides(overrides: IRendererFeatureOverrides): void {
    this.storeRendererChoice({
      overrides: mergeRendererFeatureOverrides(this.rendererChoice.overrides, overrides),
      preset: this.rendererChoice.preset,
    });
  }

  @BoundAction()
  public setCatalogView(view: TCatalogView): void {
    this.log.info("Set catalog view:", view);

    this.catalogView = view;
    setLocalStorageValue(CATALOG_VIEW_STORAGE_KEY, view);
  }

  private storeRendererChoice(choice: IRendererFeatureChoice): void {
    this.rendererChoice = toRendererFeatureChoice(choice);
    setLocalStorageValueSafe(RENDERER_FEATURES_STORAGE_KEY, JSON.stringify(this.rendererChoice));
  }
}
