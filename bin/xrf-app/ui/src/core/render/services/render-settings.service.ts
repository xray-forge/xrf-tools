import { inject, Injectable, OnDeprovision, OnProvision } from "@wirestate/core";
import { BoundAction, comparer, Computed, Observable, reaction, RefObservable, runInAction } from "@wirestate/mobx";
import { renderCommands } from "@xrf/ipc/commands/render";
import {
  RENDER_DEFAULT_PERIODIC_BUDGET,
  RenderBackendAvailability,
  RenderGpu,
  RenderGraphSettings,
  RenderSettings,
} from "@xrf/ipc/types/xrf-renderer";
import { Logger } from "@xrf/logging";
import { clamp } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { toDrawnRenderLightLevel } from "@/core/render/lib/features/render-light-level-choices";
import {
  toRenderBackend,
  toRenderBackendChoice,
  TRenderBackendChoice,
} from "@/core/render/lib/settings/render-backend-choice";
import {
  IRenderFeatureChoice,
  listUnkeptRenderSettings,
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
import { SettingsService } from "@/core/settings/services/settings";
import {
  booleanCodec,
  FRAME_RATE_LIMIT_STORAGE_KEY,
  GPU_TIMED_STORAGE_KEY,
  jsonCodec,
  RENDER_BACKEND_STORAGE_KEY,
  RENDER_GPU_STORAGE_KEY,
  RENDER_OUTPUT_SCALE_STORAGE_KEY,
  RENDER_RESOLUTION_STORAGE_KEY,
  RENDERER_FEATURES_STORAGE_KEY,
  StoredValue,
  textCodec,
  VSYNC_STORAGE_KEY,
} from "@/core/storage";

/**
 * What every viewport is drawn with: how often, on which graphics API, at what resolution, and with which features.
 */
@Injectable()
export class RenderSettingsService {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly storedFrameRateLimit: StoredValue<TFrameRateLimit> = new StoredValue(
    FRAME_RATE_LIMIT_STORAGE_KEY,
    textCodec(toFrameRateLimit)
  );

  private readonly storedGpuTimed: StoredValue<boolean> = new StoredValue(GPU_TIMED_STORAGE_KEY, booleanCodec(false));

  private readonly storedVsync: StoredValue<boolean> = new StoredValue(VSYNC_STORAGE_KEY, booleanCodec(true));

  private readonly storedRenderBackend: StoredValue<TRenderBackendChoice> = new StoredValue(
    RENDER_BACKEND_STORAGE_KEY,
    textCodec(toRenderBackendChoice)
  );

  private readonly storedRenderGpu: StoredValue<Nullable<RenderGpu>> = new StoredValue(
    RENDER_GPU_STORAGE_KEY,
    jsonCodec(toRenderGpuChoice)
  );

  private readonly storedRenderResolution: StoredValue<ERenderResolution> = new StoredValue(
    RENDER_RESOLUTION_STORAGE_KEY,
    textCodec(toRenderResolution)
  );

  private readonly storedRenderOutputScale: StoredValue<number> = new StoredValue(
    RENDER_OUTPUT_SCALE_STORAGE_KEY,
    textCodec(toRenderOutputScale)
  );

  private readonly storedRendererChoice: StoredValue<IRenderFeatureChoice> = new StoredValue(
    RENDERER_FEATURES_STORAGE_KEY,
    jsonCodec(RenderSettingsService.readRendererChoice)
  );

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

  /** What each graphics API can do on this machine, as the renderer last probed it; none before it was. */
  @RefObservable()
  public renderBackends: Nullable<ReadonlyArray<RenderBackendAvailability>> = null;

  /** How many holds time every viewport's passes on the GPU for now, whatever `isGpuTimed` keeps. */
  @Observable()
  private gpuTimingHolds: number = 0;

  /** Stops telling the renderer its settings, while provisioned. */
  private stopConfiguring: Nullable<() => void> = null;

  public constructor(private readonly settingsService: SettingsService = inject(SettingsService)) {}

  /** Frames a second every viewport is allowed to draw. */
  @Computed()
  public get frameRateLimit(): TFrameRateLimit {
    return this.storedFrameRateLimit.value;
  }

  /** Whether every viewport times its passes on the GPU: never a preset's, as it costs a frame 1-4% of its rate. */
  @Computed()
  public get isGpuTimed(): boolean {
    return this.storedGpuTimed.value;
  }

  /** Whether every viewport's frames wait for the display's refresh: on unless turned off. */
  @Computed()
  public get isVsync(): boolean {
    return this.storedVsync.value;
  }

  /** Which graphics API every viewport asks the renderer for, by name or whichever works first. */
  @Computed()
  public get renderBackend(): TRenderBackendChoice {
    return this.storedRenderBackend.value;
  }

  /** Which GPU every viewport asks the renderer for, or null for the one the backend picks automatically. */
  @Computed()
  public get renderGpu(): Nullable<RenderGpu> {
    return this.storedRenderGpu.value;
  }

  /** What every viewport resolves its frame at: its own size times the output scale, or a height in its shape. */
  @Computed()
  public get renderResolution(): ERenderResolution {
    return this.storedRenderResolution.value;
  }

  /** The share of the viewport's sides the automatic resolution resolves at. */
  @Computed()
  public get renderOutputScale(): number {
    return this.storedRenderOutputScale.value;
  }

  /** The renderer's preset and what was changed on top of it, the same for every viewport. */
  @Computed()
  public get rendererChoice(): IRenderFeatureChoice {
    return this.storedRendererChoice.value;
  }

  /** Whether the physical light levels may be chosen and drawn: in dev mode only. */
  @Computed()
  public get isPhysicalLightLevelAvailable(): boolean {
    return this.settingsService.isDevModeEnabled;
  }

  /** Every renderer feature as the choice sets it, a physical light level drawn as the authored one outside dev mode. */
  @Computed()
  public get rendererFeatures(): IRenderFeatureSettings {
    const features: IRenderFeatureSettings = resolveRenderFeatures(this.rendererChoice);

    return {
      ...features,
      lightLevel: toDrawnRenderLightLevel(features.lightLevel, this.isPhysicalLightLevelAvailable),
    };
  }

  /**
   * What the renderer draws every viewport with: how often, whether its passes are timed, how its graph compiles, on
   * which graphics API (the renderer falling back to the first that works where the one asked for does not), and how its
   * textures are filtered and loaded.
   */
  @Computed()
  public get renderSettings(): RenderSettings {
    return {
      backend: toRenderBackend(this.renderBackend),
      gpu: this.renderGpu === null ? null : { ...this.renderGpu },
      frameRate: toRenderFrameRate(this.frameRateLimit, this.isVsync),
      graph: { ...this.graph },
      isGpuTimed: this.isGpuTimed || this.gpuTimingHolds > 0,
      periodicBudget: this.periodicBudget,
      // The one texture cache every viewport samples is filtered and loaded alike for all of them.
      textureFiltering: this.rendererFeatures.textureFiltering,
      textureLod: this.rendererFeatures.textureLod,
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
   * @param stored - The parsed stored choice, `null` when none was stored.
   * @returns The choice, or the default where none was stored.
   */
  private static readRendererChoice(stored: unknown): IRenderFeatureChoice {
    const choice: IRenderFeatureChoice = toRenderFeatureChoice(stored);
    const unkept: Array<string> = listUnkeptRenderSettings(
      (stored as Nullable<{ overrides?: unknown }>)?.overrides,
      choice.overrides
    );

    if (unkept.length) {
      Logger.warn("Stored renderer settings the schema does not take, left to the preset:", unkept);
    }

    return choice;
  }

  @BoundAction()
  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.log.info("Set frame rate limit:", limit);

    this.storedFrameRateLimit.set(limit);
  }

  @BoundAction()
  public setVsync(isVsync: boolean): void {
    this.log.info("Set vsync:", isVsync);

    this.storedVsync.set(isVsync);
  }

  @BoundAction()
  public setGpuTimed(isGpuTimed: boolean): void {
    this.log.info("Set GPU timing:", isGpuTimed);

    this.storedGpuTimed.set(isGpuTimed);
  }

  /**
   * Times every viewport's passes on the GPU until released, for this session only and leaving the setting as it is.
   *
   * @returns Releases the hold; called again, it does nothing.
   */
  @BoundAction()
  public holdGpuTiming(): () => void {
    let isHeld: boolean = true;

    this.gpuTimingHolds += 1;

    return () => {
      if (isHeld) {
        isHeld = false;
        runInAction(() => {
          this.gpuTimingHolds -= 1;
        });
      }
    };
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

    this.storedRenderBackend.set(choice);
  }

  @BoundAction()
  public setRenderGpu(gpu: Nullable<RenderGpu>): void {
    this.log.info("Set render GPU:", gpu);

    this.storedRenderGpu.set(gpu);
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

    this.storedRenderResolution.set(resolution);
  }

  @BoundAction()
  public setRenderOutputScale(scale: number): void {
    const kept: number = clamp(scale, RENDER_OUTPUT_SCALES.min, RENDER_OUTPUT_SCALES.max);

    this.log.info("Set render output scale:", kept);

    this.storedRenderOutputScale.set(kept);
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

    const unkept: Array<string> = listUnkeptRenderSettings(overrides, this.rendererChoice.overrides);

    if (unkept.length) {
      this.log.warn("Renderer settings the schema does not take, left to the preset:", unkept);
    }
  }

  private storeRendererChoice(choice: IRenderFeatureChoice): void {
    this.storedRendererChoice.set(toRenderFeatureChoice(choice));
  }
}
