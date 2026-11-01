import { useInjection } from "@wirestate/react";
import { useMemo } from "react";

import {
  ILevelFeatureOptions,
  TLevelOutputOptions,
  toLevelOutput,
} from "@/core/level/lib/features/level-feature-options";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewService } from "@/core/level/services/level-view";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { ERenderResolution } from "@/core/render/lib/settings/device/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/** What a view's features resolve from, and what they resolve to. */
export interface ILevelRenderFeatures {
  /** What the settings draw every viewport with. */
  settings: IRenderFeatureSettings;
  /** What the view sets over them. */
  features: ILevelFeatureOptions;
  /**
   * Every feature as the view draws it with every toolbar toggle on: its own values over the settings', a dev light
   * level drawn as the authored one outside dev mode (`LevelRenderDrawing.features`).
   */
  drawn: IRenderFeatureSettings;
  /** What the view resolves its frame at: its own resolution and scale over the settings'. */
  output: Required<TLevelOutputOptions>;
}

/**
 * @returns The settings' features, the view's own, and what they resolve to, for a popover to show and change.
 */
export function useLevelRenderFeatures(): ILevelRenderFeatures {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const viewService: LevelViewService = useInjection(LevelViewService);
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  const settings: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const features: ILevelFeatureOptions = viewService.features;
  const resolution: ERenderResolution = renderSettingsService.renderResolution;
  const scale: number = renderSettingsService.renderOutputScale;
  const drawn: IRenderFeatureSettings = renderService.drawing.features;
  const output: Required<TLevelOutputOptions> = useMemo(
    () => toLevelOutput(features.output, resolution, scale),
    [features.output, resolution, scale]
  );

  return { drawn, features, output, settings };
}
