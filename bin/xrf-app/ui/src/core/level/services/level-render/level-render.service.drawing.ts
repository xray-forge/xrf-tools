import { comparer, Computed, makeObservable, reaction } from "@wirestate/mobx";
import { SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import {
  ERenderDlssAvailability,
  RenderAntialiasing,
  RenderDlssAvailability,
  RenderFrameReport,
  RenderViewOptions,
} from "@xrf/ipc/types/xrf-renderer";
import { WorldToggles } from "@xrf/ipc/types/xrf-world";
import { Maybe, Nullable } from "@xrf/types";

import { ILevelBox, toLevelBox } from "@/core/level/lib/extent/level-extent";
import {
  ILevelFeatureOptions,
  LEVEL_FEATURE_TOGGLES_ON,
  TLevelOutputOptions,
  toLevelOutput,
  toLevelRendererAntialiasing,
  toLevelRendererFeatures,
} from "@/core/level/lib/features/level-feature-options";
import {
  toLevelHelperRenderOptions,
  toLevelListingPlaceMap,
  toLevelSpawnIndices,
} from "@/core/level/lib/objects/level-helper-render";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { toLevelFrameOverlays } from "@/core/level/lib/render/level-render-frame";
import { toLevelViewOptions } from "@/core/level/lib/render/level-render-view";
import { ELevelShading, toShownLevelShading } from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelLookService } from "@/core/level/services/level-look";
import { ILevelRenderContext } from "@/core/level/services/level-render/level-render.service.context";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { toDrawnAntialiasing } from "@/core/render/lib/features/image/render-antialiasing-choices";
import { toDrawnRenderLightLevel } from "@/core/render/lib/features/tone/render-light-level-choices";
import { toNativeWorldToggles } from "@/core/render/lib/native/contract/native-view-options";
import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { toRenderOutput } from "@/core/render/lib/settings/device/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/** What the level is drawn with. */
export interface ILevelRenderDrawingSources {
  /** The open level's extent and its listed objects, whose helpers are drawn. */
  loadService: LevelLoadService;
  /** How the level is exposed, lit and corrected. */
  lookService: LevelLookService;
  /** What the tool sets every viewport's features to. */
  renderSettingsService: RenderSettingsService;
  /** The toolbar's toggles, features, shading and helpers. */
  viewService: LevelViewService;
  /** What is hidden, hovered and selected, which the helpers show. */
  viewportService: LevelViewportService;
  /** How finely the sunshafts step. */
  weatherService: LevelWeatherService;
  /** The overlays' sizes. */
  config: ILevelRenderConfig;
}

/**
 * What the level is drawn with on the GPU drawing it: the toolbar, the look and the settings together, what the world
 * lets play, and the overlays over it.
 */
export class LevelRenderDrawing {
  /**
   * @param context - The frames the viewport reports, which name the GPU drawing it.
   * @param sources - What the level is drawn with.
   */
  public constructor(
    private readonly context: ILevelRenderContext,
    private readonly sources: ILevelRenderDrawingSources
  ) {
    // Composed state is not activated by the container's observable plugin.
    makeObservable(this);
  }

  /** Whether the GPU drawing the view traces rays, taken as yes until a frame names the GPU. */
  @Computed()
  public get isTraceable(): boolean {
    const frame: RenderFrameReport = this.context.getFrame();

    return !frame.adapter || frame.hasRayQueries;
  }

  /** Whether the view's frame is drawn smaller than the viewport. */
  @Computed()
  public get isUpscaled(): boolean {
    const frame: RenderFrameReport = this.context.getFrame();

    return frame.renderWidth > 0 && frame.renderWidth < frame.width;
  }

  /** Whether the GPU drawing the view upscales with DLSS, and what it lacks where not: available until a frame says. */
  @Computed()
  public get dlss(): RenderDlssAvailability {
    const frame: RenderFrameReport = this.context.getFrame();

    return frame.adapter ? frame.dlss : ERenderDlssAvailability.AVAILABLE;
  }

  /** Whether the GPU drawing the view reconstructs rays with DLSS: available until a frame says otherwise. */
  @Computed()
  public get rayReconstruction(): RenderDlssAvailability {
    const frame: RenderFrameReport = this.context.getFrame();

    return frame.adapter ? frame.rayReconstruction : ERenderDlssAvailability.AVAILABLE;
  }

  /** Whether the view's last frame was reconstructed by DLSS ray reconstruction, its traces taken raw. */
  @Computed()
  public get isRayReconstructed(): boolean {
    return this.context.getFrame().isRayReconstructed;
  }

  /** What smooths the view's frame: the toolbar's antialiasing over the settings', FSR for DLSS on a GPU without it. */
  @Computed()
  public get antialiasing(): RenderAntialiasing {
    const { renderSettingsService, viewService } = this.sources;

    return toDrawnAntialiasing(
      toLevelRendererAntialiasing(
        renderSettingsService.rendererFeatures.antialiasing,
        viewService.features,
        viewService.options.isAntialiased
      ),
      this.dlss
    );
  }

  /** What the viewport shows of its surfaces: the view's shading, the final frame for FSR's while FSR is not drawing. */
  @Computed()
  public get shading(): ELevelShading {
    return toShownLevelShading(this.sources.viewService.shading, this.antialiasing);
  }

  /**
   * What the view sets over the settings as it is drawn: a dev light level it keeps drawn as the authored one outside
   * dev mode, as the settings' own is (`RenderSettingsService.rendererFeatures`).
   */
  @Computed()
  public get viewFeatures(): ILevelFeatureOptions {
    const features: ILevelFeatureOptions = this.sources.viewService.features;

    return features.lightLevel === null
      ? features
      : {
          ...features,
          lightLevel: toDrawnRenderLightLevel(
            features.lightLevel,
            this.sources.renderSettingsService.isDevLightLevelAvailable
          ),
        };
  }

  /** Every feature as the view draws it with every toolbar toggle on, which its popovers show and its snapshots keep. */
  @Computed()
  public get features(): IRenderFeatureSettings {
    return toLevelRendererFeatures(
      this.sources.renderSettingsService.rendererFeatures,
      this.viewFeatures,
      LEVEL_FEATURE_TOGGLES_ON
    );
  }

  /** What the level is asked to be drawn with: the toolbar, the look and the settings together. */
  @Computed()
  public get options(): RenderViewOptions {
    const { lookService, renderSettingsService, viewService, viewportService, weatherService } = this.sources;
    const output: Required<TLevelOutputOptions> = toLevelOutput(
      viewService.features.output,
      renderSettingsService.renderResolution,
      renderSettingsService.renderOutputScale
    );

    const options: RenderViewOptions = toLevelViewOptions({
      features: renderSettingsService.rendererFeatures,
      hemiStrength: viewService.hemiStrength,
      lod: viewService.lod,
      look: lookService.look,
      options: viewService.options,
      output: toRenderOutput(output.resolution, output.scale),
      shading: this.shading,
      sunShafts: weatherService.sunShafts,
      view: this.viewFeatures,
    });
    const { hidden, hovered, selected } = viewportService;

    return {
      ...options,
      helpers: toLevelHelperRenderOptions(viewService.helpers, { hidden, hovered, selected }, this.listingPlaces),
    };
  }

  /** Each listed level object's place in the listing by its key, which names its helper: built once a listing. */
  @Computed()
  public get listingPlaces(): ReadonlyMap<string, number> {
    return toLevelListingPlaceMap(this.sources.loadService.objects?.objects ?? []);
  }

  /** What of the level's world the toolbar lets play, less the spawned objects hidden one by one, and what is heard. */
  @Computed()
  public get worldToggles(): WorldToggles {
    const options: ILevelViewOptions = this.sources.viewService.options;

    return toNativeWorldToggles(options, toLevelSpawnIndices(this.sources.viewportService.hidden), {
      isAmbience: options.isAmbienceHeard,
      isHeard: options.isSoundHeard,
      isMusic: options.isMusicHeard,
      isWorld: options.isWorldHeard,
    });
  }

  /**
   * Tells a viewport just attached what the level is drawn with, what its world plays and the overlays over it, and
   * again as any of them changes.
   *
   * @param viewport - The viewport just attached.
   * @returns What stops telling it.
   */
  public start(viewport: NativeViewport): Array<() => void> {
    const { loadService, viewService, config } = this.sources;

    return [
      reaction(
        () => this.options,
        (options: RenderViewOptions) => viewport.setViewOptions(options),
        { equals: comparer.structural, fireImmediately: true }
      ),
      reaction(
        () => this.worldToggles,
        (toggles: WorldToggles) => viewport.setWorldToggles(toggles),
        { equals: comparer.structural, fireImmediately: true }
      ),
      // Keyed by the extent and the switches alone: the overlays themselves are long arrays.
      reaction(
        () => {
          const level: Maybe<SelectedLevelDescription> = loadService.level.value?.selected.value;
          const { isAxesVisible, isGridVisible, isSunMarked } = viewService.options;
          const box: Nullable<ILevelBox> = level ? toLevelBox(level.bounds) : null;

          return { box, isAxesVisible, isGridVisible, isSunMarked };
        },
        ({ box, ...options }) => viewport.setOverlays(toLevelFrameOverlays(box, options, config)),
        { equals: comparer.structural, fireImmediately: true }
      ),
    ];
  }
}
