import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { RenderPassTimingFormRow } from "@/core/render/components/controls/RenderPassTimingFormRow";
import { RENDER_FRAME_RATE_OPTIONS, RENDER_RESOLUTION_OPTIONS } from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";

/** How often and how large every viewport draws, and whether its passes are timed: nothing a preset sets. */
export function SettingsRendererDisplay(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  return (
    <div className={"flex flex-col gap-6"}>
      <ChoiceFormRow
        label={"Frame rate limit"}
        description={"Maximum frames per second a viewport draws."}
        options={RENDER_FRAME_RATE_OPTIONS}
        value={settingsService.frameRateLimit}
        onChange={settingsService.setFrameRateLimit}
      />

      <CheckboxFormRow
        label={"Low latency"}
        description={
          "Keeps the GPU at most one frame behind. Reduces input lag when GPU-bound, at a small frame rate cost."
        }
        isChecked={settingsService.isLowLatency}
        onChange={settingsService.setLowLatency}
      />

      <ChoiceFormRow
        label={"Resolution"}
        description={"Internal resolution of every viewport. Window matches the viewport's own size."}
        options={RENDER_RESOLUTION_OPTIONS}
        value={settingsService.renderResolution}
        onChange={settingsService.setRenderResolution}
      />

      <RenderPassTimingFormRow isChecked={settingsService.isGpuTimed} onChange={settingsService.setGpuTimed} />
    </div>
  );
}
