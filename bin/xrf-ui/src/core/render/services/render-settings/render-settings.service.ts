import { Injectable, OnDeprovision, OnProvision } from "@wirestate/core";
import { BoundAction, comparer, Computed, Observable, reaction, RefObservable } from "@wirestate/mobx";
import { clamp } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { renderCommands } from "@/core/ipc/commands/render";
import {
  RENDER_DEFAULT_PERIODIC_BUDGET,
  RenderBackendAvailability,
  RenderGpu,
  RenderGraphSettings,
  RenderSettings,
} from "@/core/ipc/types/xrf-renderer";
import {
  toRenderBackend,
  toRenderBackendChoice,
  TRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";
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
import { toRenderGpuChoice } from "@/core/render/lib/settings/render-gpu-choice";
import { DEFAULT_RENDER_GRAPH_SETTINGS } from "@/core/render/lib/settings/render-graph-settings";
import { ERenderPreset } from "@/core/render/lib/settings/render-preset";
import {
  ERenderResolution,
  RENDER_OUTPUT_SCALES,
  toRenderOutputScale,
  toRenderResolution,
} from "@/core/render/lib/settings/render-resolution";
import {
  FRAME_RATE_LIMIT_STORAGE_KEY,
  getLocalStorageValue,
  GPU_TIMED_STORAGE_KEY,
  parseLocalStorageValueSafe,
  RENDER_BACKEND_STORAGE_KEY,
  RENDER_GPU_STORAGE_KEY,
  RENDER_OUTPUT_SCALE_STORAGE_KEY,
  RENDER_RESOLUTION_STORAGE_KEY,
  RENDERER_FEATURES_STORAGE_KEY,
  setLocalStorageValue,
  setLocalStorageValueSafe,
  VSYNC_STORAGE_KEY,
} from "@/core/storage";
import { Logger } from "@/lib/logging";

/**
 * What every viewport is drawn with: how often, on which graphics API, at what resolution, and with which features.
 */
@Injectable()
export class RenderSettingsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

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

  /**
   * Milliseconds of GPU time each frame gives the renderer's work drawn once in so many frames: the renderer's default
   * unless set to measure another, for the session only.
   */
  @Observable()
  public periodicBudget: number = RENDER_DEFAULT_PERIODIC_BUDGET;

  /** Whether every viewport's frames wait for the display's refresh: on unless turned off. */
  @Observable()
  public isVsync: boolean = getLocalStorageValue(VSYNC_STORAGE_KEY) !== String(false);

  /** Which graphics API every viewport asks the renderer for, by name or whichever works first. */
  @Observable()
  public renderBackend: TRenderBackendChoice = toRenderBackendChoice(getLocalStorageValue(RENDER_BACKEND_STORAGE_KEY));

  /** Which GPU every viewport asks the renderer for, or null for the one the backend picks automatically. */
  @RefObservable()
  public renderGpu: Nullable<RenderGpu> = toRenderGpuChoice(parseLocalStorageValueSafe(RENDER_GPU_STORAGE_KEY));

  /** What each graphics API can do on this machine, as the renderer last probed it; none before it was. */
  @RefObservable()
  public renderBackends: Nullable<ReadonlyArray<RenderBackendAvailability>> = null;

  @Observable()
  public renderResolution: ERenderResolution = toRenderResolution(getLocalStorageValue(RENDER_RESOLUTION_STORAGE_KEY));
  /** The share of the viewport's sides the automatic resolution resolves at. */
  @Observable()
  public renderOutputScale: number = toRenderOutputScale(getLocalStorageValue(RENDER_OUTPUT_SCALE_STORAGE_KEY));

  /** The renderer's preset and what was changed on top of it, the same for every viewport. */
  @RefObservable()
  public rendererChoice: IRenderFeatureChoice = RenderSettingsService.readRendererChoice();

  /** Stops telling the renderer its settings, while provisioned. */
  private stopConfiguring: Nullable<() => void> = null;

  /** Every renderer feature as the choice sets it. */
  @Computed()
  public get rendererFeatures(): IRenderFeatureSettings {
    return resolveRenderFeatures(this.rendererChoice);
  }

  /**
   * What the renderer draws every viewport with: how often, whether its passes are timed, how its graph compiles, and on
   * which graphics API, the renderer falling back to the first that works where the one asked for does not.
   */
  @Computed()
  public get renderSettings(): RenderSettings {
    return {
      backend: toRenderBackend(this.renderBackend),
      gpu: this.renderGpu === null ? null : { ...this.renderGpu },
      frameRate: toRenderFrameRate(this.frameRateLimit, this.isVsync),
      graph: { ...this.graph },
      isGpuTimed: this.isGpuTimed,
      periodicBudget: this.periodicBudget,
    };
  }

  /**
   * Tells the renderer every viewport's settings at once and again on each change, whether a viewport is attached or
   * not; provisioned again, it replaces its reaction rather than adding a second.
   */
  @OnProvision()
  public onProvision(): void {
    this.stopConfiguring?.();
    this.stopConfiguring = reaction(
      () => this.renderSettings,
      (settings: RenderSettings) =>
        void renderCommands
          .configure(settings)
          .catch((error: unknown) => this.log.warn("The renderer refused its settings:", error)),
      { equals: comparer.structural, fireImmediately: true }
    );
  }

  /** Stops telling the renderer its settings. */
  @OnDeprovision()
  public onDeprovision(): void {
    this.stopConfiguring?.();
    this.stopConfiguring = null;
  }

  /**
   * @returns The stored choice, or the default where none was stored or it does not parse.
   */
  private static readRendererChoice(): IRenderFeatureChoice {
    return toRenderFeatureChoice(parseLocalStorageValueSafe(RENDERER_FEATURES_STORAGE_KEY));
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
  public setPeriodicBudget(periodicBudget: number): void {
    this.log.info("Set periodic budget:", periodicBudget);

    this.periodicBudget = Math.max(0, periodicBudget);
  }

  @BoundAction()
  public setRenderBackend(choice: TRenderBackendChoice): void {
    this.log.info("Set render backend:", choice);

    this.renderBackend = choice;
    setLocalStorageValue(RENDER_BACKEND_STORAGE_KEY, choice);
  }

  @BoundAction()
  public setRenderGpu(gpu: Nullable<RenderGpu>): void {
    this.log.info("Set render GPU:", gpu);

    this.renderGpu = gpu;
    setLocalStorageValueSafe(RENDER_GPU_STORAGE_KEY, gpu === null ? null : JSON.stringify(gpu));
  }

  /** Asks the renderer which graphics APIs it can draw with here, for the settings to offer only those. */
  @BoundAction()
  public async refreshRenderBackends(): Promise<void> {
    try {
      const backends: Array<RenderBackendAvailability> = await renderCommands.listBackends();

      this.setRenderBackends(backends);
    } catch (error) {
      this.log.warn("Failed to probe the graphics backends:", error);
    }
  }

  @BoundAction()
  private setRenderBackends(backends: ReadonlyArray<RenderBackendAvailability>): void {
    this.renderBackends = backends;
  }

  @BoundAction()
  public setRenderResolution(resolution: ERenderResolution): void {
    this.log.info("Set render resolution:", resolution);

    this.renderResolution = resolution;
    setLocalStorageValue(RENDER_RESOLUTION_STORAGE_KEY, resolution);
  }

  @BoundAction()
  public setRenderOutputScale(scale: number): void {
    const kept: number = clamp(scale, RENDER_OUTPUT_SCALES.min, RENDER_OUTPUT_SCALES.max);

    this.log.info("Set render output scale:", kept);

    this.renderOutputScale = kept;
    setLocalStorageValue(RENDER_OUTPUT_SCALE_STORAGE_KEY, String(kept));
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

  private storeRendererChoice(choice: IRenderFeatureChoice): void {
    this.rendererChoice = toRenderFeatureChoice(choice);
    setLocalStorageValueSafe(RENDERER_FEATURES_STORAGE_KEY, JSON.stringify(this.rendererChoice));
  }
}
