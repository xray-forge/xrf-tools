import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { RENDER_FRAME_RATE_OPTIONS, RENDER_RESOLUTION_OPTIONS } from "@/core/render/lib/features";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";

import { SettingsRendererAmbientOcclusion } from "./SettingsRendererAmbientOcclusion";
import { SettingsRendererFeatures } from "./SettingsRendererFeatures";
import { SettingsRendererGrass } from "./SettingsRendererGrass";
import { SettingsRendererLights } from "./SettingsRendererLights";
import { SettingsRendererLod } from "./SettingsRendererLod";
import { SettingsRendererShadows } from "./SettingsRendererShadows";

/** How every viewport draws, which is neither application chrome nor any one editor's business. */
export function SettingsRenderSection(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  return (
    <div className={"flex flex-col gap-6"}>
      <ChoiceFormRow
        label={"Frame rate limit"}
        description={"How often a viewport redraws. A display faster than this costs power for frames nobody sees."}
        options={RENDER_FRAME_RATE_OPTIONS}
        value={settingsService.frameRateLimit}
        onChange={settingsService.setFrameRateLimit}
      />

      <CheckboxFormRow
        label={"Low latency"}
        description={
          "Waits for the GPU to be at most a frame behind before drawing another, so the camera answers a frame " +
          "sooner where the GPU is what holds the frame rate back, for about a tenth fewer frames."
        }
        isChecked={settingsService.isLowLatency}
        onChange={settingsService.setLowLatency}
      />

      <ChoiceFormRow
        label={"Resolution"}
        description={
          "How many pixels a viewport draws, whatever size the window is. Below the window it costs less and " +
          "reads softer; above it, more, and edges read sharper."
        }
        options={RENDER_RESOLUTION_OPTIONS}
        value={settingsService.renderResolution}
        onChange={settingsService.setRenderResolution}
      />

      <SettingsRendererFeatures />

      <SettingsRendererShadows />

      <SettingsRendererLights />

      <SettingsRendererAmbientOcclusion />

      <SettingsRendererGrass />

      <SettingsRendererLod />
    </div>
  );
}
