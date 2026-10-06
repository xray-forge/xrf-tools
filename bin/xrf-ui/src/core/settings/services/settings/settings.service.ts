import { Injectable, OnDeprovision, OnProvision, ProvisionId } from "@wirestate/core";
import { BoundAction, Computed, Observable, RefObservable } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { RenderGraphSettings, RenderSettings } from "@/core/ipc/types/xrf-renderer";
import {
  IRenderFeatureChoice,
  mergeRenderFeatureOverrides,
  resolveRenderFeatures,
  toRenderFeatureChoice,
} from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import {
  TFrameRateLimit,
  toFrameRateLimit,
  toRenderFrameRate,
} from "@/core/render/lib/settings/render-frame-rate-limit";
import { DEFAULT_RENDER_GRAPH_SETTINGS } from "@/core/render/lib/settings/render-graph-settings";
import { ERenderPreset } from "@/core/render/lib/settings/render-preset";
import { ERenderResolution, toRenderResolution } from "@/core/render/lib/settings/render-resolution";
import { TCatalogView, toCatalogView } from "@/core/settings/lib/catalog-view";
import {
  CATALOG_VIEW_STORAGE_KEY,
  DEV_MODE_STORAGE_KEY,
  FRAME_RATE_LIMIT_STORAGE_KEY,
  GPU_TIMED_STORAGE_KEY,
  RENDER_RESOLUTION_STORAGE_KEY,
  RENDERER_FEATURES_STORAGE_KEY,
  VSYNC_STORAGE_KEY,
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

  /**
   * Frames a second every viewport is allowed to draw.
   */
  @Observable()
  public frameRateLimit: TFrameRateLimit = toFrameRateLimit(getLocalStorageValue(FRAME_RATE_LIMIT_STORAGE_KEY));

  /** Whether every viewport times its passes on the GPU: never a preset's, as it costs a frame 1-4% of its rate. */
  @Observable()
  public isGpuTimed: boolean = getLocalStorageValue(GPU_TIMED_STORAGE_KEY) === String(true);

  /**
   * Which of the frame graph's mechanisms every frame compiles with: all on unless one is turned off to bisect a
   * difference in a capture, for the session only, as a diagnostic.
   */
  @Observable()
  public graph: RenderGraphSettings = { ...DEFAULT_RENDER_GRAPH_SETTINGS };

  /** Whether every viewport's frames wait for the display's refresh: on unless turned off. */
  @Observable()
  public isVsync: boolean = getLocalStorageValue(VSYNC_STORAGE_KEY) !== String(false);

  @Observable()
  public renderResolution: ERenderResolution = toRenderResolution(getLocalStorageValue(RENDER_RESOLUTION_STORAGE_KEY));

  /** The renderer's preset and what was changed on top of it, the same for every viewport. */
  @RefObservable()
  public rendererChoice: IRenderFeatureChoice = SettingsService.readRendererChoice();

  /** Every renderer feature as the choice sets it. */
  @Computed()
  public get rendererFeatures(): IRenderFeatureSettings {
    return resolveRenderFeatures(this.rendererChoice);
  }

  /** What the renderer draws every viewport with: how often, whether its passes are timed, and how its graph compiles. */
  @Computed()
  public get renderSettings(): RenderSettings {
    return {
      frameRate: toRenderFrameRate(this.frameRateLimit, this.isVsync),
      graph: { ...this.graph },
      isGpuTimed: this.isGpuTimed,
    };
  }

  /**
   * @returns The stored choice, or the default where none was stored or it does not parse.
   */
  private static readRendererChoice(): IRenderFeatureChoice {
    return toRenderFeatureChoice(parseLocalStorageValueSafe(RENDERER_FEATURES_STORAGE_KEY));
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
  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.log.info("Set frame rate limit:", limit);

    this.frameRateLimit = limit;
    setLocalStorageValue(FRAME_RATE_LIMIT_STORAGE_KEY, limit);
  }

  @BoundAction()
  public setVsync(isVsync: boolean): void {
    this.log.info("Set vsync:", isVsync);

    this.isVsync = isVsync;
    setLocalStorageValue(VSYNC_STORAGE_KEY, String(isVsync));
  }

  @BoundAction()
  public setGpuTimed(isGpuTimed: boolean): void {
    this.log.info("Set GPU timing:", isGpuTimed);

    this.isGpuTimed = isGpuTimed;
    setLocalStorageValue(GPU_TIMED_STORAGE_KEY, String(isGpuTimed));
  }

  @BoundAction()
  public setGraph(graph: RenderGraphSettings): void {
    this.log.info("Set frame graph:", graph);

    this.graph = { ...graph };
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
  public setRendererPreset(preset: ERenderPreset): void {
    this.log.info("Set renderer preset:", preset);

    this.storeRendererChoice({ overrides: {}, preset });
  }

  /**
   * @param overrides - What changes on top of the preset, merged over what already did.
   */
  @BoundAction()
  public setRendererOverrides(overrides: IRenderFeatureOverrides): void {
    this.storeRendererChoice({
      overrides: mergeRenderFeatureOverrides(this.rendererChoice.overrides, overrides),
      preset: this.rendererChoice.preset,
    });
  }

  @BoundAction()
  public setCatalogView(view: TCatalogView): void {
    this.log.info("Set catalog view:", view);

    this.catalogView = view;
    setLocalStorageValue(CATALOG_VIEW_STORAGE_KEY, view);
  }

  private storeRendererChoice(choice: IRenderFeatureChoice): void {
    this.rendererChoice = toRenderFeatureChoice(choice);
    setLocalStorageValueSafe(RENDERER_FEATURES_STORAGE_KEY, JSON.stringify(this.rendererChoice));
  }
}
