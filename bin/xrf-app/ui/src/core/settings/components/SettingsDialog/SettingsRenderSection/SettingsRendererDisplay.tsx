import { useInjection } from "@wirestate/react";
import { CheckboxFormRow } from "@xrf/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@xrf/ui/form/ChoiceFormRow";
import { ReactElement, useEffect } from "react";

import { RenderBackendFormRow } from "@/core/render/components/controls/RenderBackendFormRow";
import { RenderGpuFormRow } from "@/core/render/components/controls/RenderGpuFormRow";
import { RenderPassTimingFormRow } from "@/core/render/components/controls/RenderPassTimingFormRow";
import { RENDER_FRAME_RATE_OPTIONS } from "@/core/render/lib/features/render-display-choices";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

/**
 * On which graphics API and how often every viewport draws, and whether its passes are timed: nothing a preset sets.
 */
export function SettingsRendererDisplay(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  useEffect(() => {
    void renderSettingsService.refreshRenderBackends();
  }, [renderSettingsService]);

  return (
    <div className={"flex flex-col gap-6"}>
      <RenderBackendFormRow
        value={renderSettingsService.renderBackend}
        gpu={renderSettingsService.renderGpu}
        availability={renderSettingsService.renderBackends}
        onChange={renderSettingsService.setRenderBackend}
      />

      <RenderGpuFormRow
        value={renderSettingsService.renderGpu}
        backend={renderSettingsService.renderBackend}
        availability={renderSettingsService.renderBackends}
        onChange={renderSettingsService.setRenderGpu}
      />

      <ChoiceFormRow
        label={"Frame rate limit"}
        description={
          "The most frames a second a viewport draws. No cap draws as fast as VSync allows, or the GPU can with it off."
        }
        options={RENDER_FRAME_RATE_OPTIONS}
        value={renderSettingsService.frameRateLimit}
        onChange={renderSettingsService.setFrameRateLimit}
      />

      <CheckboxFormRow
        label={"VSync"}
        description={
          "Presents each frame at the display's refresh. Off, frames show as soon as they are drawn, which can tear."
        }
        isChecked={renderSettingsService.isVsync}
        onChange={renderSettingsService.setVsync}
      />

      <RenderPassTimingFormRow
        isChecked={renderSettingsService.isGpuTimed}
        onChange={renderSettingsService.setGpuTimed}
      />
    </div>
  );
}
