import { useInjection } from "@wirestate/react";
import { ERenderBloomMode, RenderBloomMode } from "@xrf/ipc/types/xrf-renderer";
import { ChoiceFormRow } from "@xrf/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@xrf/ui/form/SliderFormRow";
import { DetailSection } from "@xrf/ui/layout/DetailSection";
import { ReactElement } from "react";

import {
  explainRenderBloomMode,
  formatBloomStrength,
  RENDER_BLOOM_MODE_OPTIONS,
  RENDER_ENHANCED_BLOOM_LIMITS,
} from "@/core/render/lib/features/render-bloom-choices";
import { TRenderEnhancedBloomSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** The glow around the frame's bright parts: the game's bloom, or the enhanced bloom in its place and its strengths. */
export function SettingsRendererBloom(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const bloom: TRenderEnhancedBloomSettings = renderSettingsService.rendererFeatures.enhancedBloom;
  const isEngine: boolean = bloom.mode === ERenderBloomMode.ENGINE;

  const onSet = useRendererOverride("enhancedBloom");

  return (
    <DetailSection title={"Bloom"} description={"The glow around the frame's bright parts."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <ChoiceFormRow
          label={"Mode"}
          description={explainRenderBloomMode(bloom.mode)}
          options={RENDER_BLOOM_MODE_OPTIONS}
          value={bloom.mode}
          onChange={(mode: RenderBloomMode) => onSet({ mode })}
        />

        <SliderFormRow
          label={"Threshold"}
          description={"How bright a part must be to glow: the higher, the less glows."}
          value={bloom.threshold}
          {...RENDER_ENHANCED_BLOOM_LIMITS.threshold}
          isDisabled={isEngine}
          format={formatBloomStrength}
          onChange={(threshold: number) => onSet({ threshold })}
        />

        <SliderFormRow
          label={"Brightness"}
          description={"How bright the glow is."}
          value={bloom.exposure}
          {...RENDER_ENHANCED_BLOOM_LIMITS.exposure}
          isDisabled={isEngine}
          format={formatBloomStrength}
          onChange={(exposure: number) => onSet({ exposure })}
        />

        <SliderFormRow
          label={"Blur"}
          description={"How wide the glow spreads."}
          value={bloom.blur}
          {...RENDER_ENHANCED_BLOOM_LIMITS.blur}
          isDisabled={isEngine}
          format={formatBloomStrength}
          onChange={(blur: number) => onSet({ blur })}
        />

        <SliderFormRow
          label={"Vibrance"}
          description={"How saturated the glow is: 1 as the frame's colour, 0 grey."}
          value={bloom.vibrance}
          {...RENDER_ENHANCED_BLOOM_LIMITS.vibrance}
          isDisabled={isEngine}
          format={formatBloomStrength}
          onChange={(vibrance: number) => onSet({ vibrance })}
        />

        <SliderFormRow
          label={"Sky"}
          description={"How much the sky glows against the rest of the frame."}
          value={bloom.sky}
          {...RENDER_ENHANCED_BLOOM_LIMITS.sky}
          isDisabled={isEngine}
          format={formatBloomStrength}
          onChange={(sky: number) => onSet({ sky })}
        />
      </div>
    </DetailSection>
  );
}
