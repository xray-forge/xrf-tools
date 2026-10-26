import { useInjection } from "@wirestate/react";

import { ILevelFeatureOptions } from "@/core/level/lib/features/level-feature-options";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";

/** What a view's features resolve from: the settings every viewport draws with, and what the view sets over them. */
export interface ILevelRenderFeatures {
  settings: IRenderFeatureSettings;
  features: ILevelFeatureOptions;
}

/**
 * @returns The settings' features and the view's own, for a popover to resolve the group it shows.
 */
export function useLevelRenderFeatures(): ILevelRenderFeatures {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);
  const viewService: LevelViewService = useInjection(LevelViewService);

  return { features: viewService.features, settings: renderSettingsService.rendererFeatures };
}
