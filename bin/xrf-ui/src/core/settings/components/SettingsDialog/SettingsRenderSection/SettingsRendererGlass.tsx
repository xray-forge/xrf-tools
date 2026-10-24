import { useInjection } from "@wirestate/react";
import { ERenderGlassMode, RenderGlassMode } from "@xrf/ipc/types/xrf-renderer";
import { ReactElement } from "react";

import {
  explainRenderGlassMode,
  formatGlassStrength,
  RENDER_GLASS_LIMITS,
  RENDER_GLASS_MODE_OPTIONS,
} from "@/core/render/lib/features/render-glass-choices";
import { TRenderGlassSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** How glass is drawn: blended over what is behind it as the game does, or the enhanced glass refracting it. */
export function SettingsRendererGlass(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const glass: TRenderGlassSettings = renderSettingsService.rendererFeatures.glass;

  const onSet = useRendererOverride("glass");

  return (
    <DetailSection title={"Glass"} description={"How windows and other glass are drawn over what is behind them."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Mode"}
          description={explainRenderGlassMode(glass.mode)}
          options={RENDER_GLASS_MODE_OPTIONS}
          value={glass.mode}
          onChange={(mode: RenderGlassMode) => onSet({ mode })}
        />

        <SliderFormRow
          label={"Refraction"}
          description={"How far the glass bends what is behind it: 1 as designed, 0 flat."}
          value={glass.refraction}
          {...RENDER_GLASS_LIMITS.refraction}
          isDisabled={glass.mode === ERenderGlassMode.ENGINE}
          format={formatGlassStrength}
          onChange={(refraction: number) => onSet({ refraction })}
        />
      </div>
    </DetailSection>
  );
}
