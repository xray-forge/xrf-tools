import { useInjection } from "@wirestate/react";
import { useCallback } from "react";

import { IRendererFeatureOverrides } from "@/core/render/lib/contract/renderer-feature-overrides";
import { IRendererFeatureSettings } from "@/core/render/lib/contract/renderer-feature-settings";
import { SettingsService } from "@/core/settings/services/settings";

/**
 * @param key - The renderer feature group a section sets.
 * @returns Sets part of the group over the settings' preset, the rest as it was.
 */
export function useRendererOverride<K extends keyof IRendererFeatureSettings>(
  key: K
): (part: NonNullable<IRendererFeatureOverrides[K]>) => void {
  const settingsService: SettingsService = useInjection(SettingsService);

  return useCallback(
    (part: NonNullable<IRendererFeatureOverrides[K]>) => {
      const overrides: IRendererFeatureOverrides = {};

      overrides[key] = part;
      settingsService.setRendererOverrides(overrides);
    },
    [key, settingsService]
  );
}
