import { useInjection } from "@wirestate/react";
import { useCallback } from "react";

import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";

/**
 * @param key - The renderer feature group a section sets.
 * @returns Sets part of the group over the settings' preset, the rest as it was.
 */
export function useRendererOverride<K extends keyof IRenderFeatureSettings>(
  key: K
): (part: NonNullable<IRenderFeatureOverrides[K]>) => void {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  return useCallback(
    (part: NonNullable<IRenderFeatureOverrides[K]>) => {
      const overrides: IRenderFeatureOverrides = {};

      overrides[key] = part;
      renderSettingsService.setRendererOverrides(overrides);
    },
    [key, renderSettingsService]
  );
}
