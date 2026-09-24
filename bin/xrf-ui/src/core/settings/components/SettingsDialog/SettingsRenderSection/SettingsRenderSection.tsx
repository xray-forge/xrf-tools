import { useInjection } from "@wirestate/react";
import { ERenderResolution, FRAME_RATE_LIMITS, RENDER_RESOLUTIONS, TFrameRateLimit } from "@xrf/renderer";
import { ReactElement } from "react";

import { SettingsService } from "@/core/settings/services/settings";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";

import { SettingsRendererFeatures } from "./SettingsRendererFeatures";
import { SettingsRendererLod } from "./SettingsRendererLod";
import { SettingsRendererShadows } from "./SettingsRendererShadows";

const FRAME_RATE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TFrameRateLimit>> = FRAME_RATE_LIMITS.map((value) => ({
  value,
  label: value === "unlimited" ? "Unlimited" : `${value} fps`,
}));

const RESOLUTION_LABELS: Record<ERenderResolution, string> = {
  [ERenderResolution.HEIGHT_1080]: "1080p",
  [ERenderResolution.HEIGHT_1440]: "1440p",
  [ERenderResolution.HEIGHT_2160]: "4K",
  [ERenderResolution.HEIGHT_720]: "720p",
  [ERenderResolution.WINDOW]: "Window",
};

const RESOLUTION_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ERenderResolution>> = RENDER_RESOLUTIONS.map((value) => ({
  value,
  label: RESOLUTION_LABELS[value],
}));

/** How every viewport draws, which is neither application chrome nor any one editor's business. */
export function SettingsRenderSection(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  return (
    <div className={"flex flex-col gap-6"}>
      <ChoiceFormRow
        label={"Frame rate limit"}
        description={"How often a viewport redraws. A display faster than this costs power for frames nobody sees."}
        options={FRAME_RATE_OPTIONS}
        value={settingsService.frameRateLimit}
        onChange={settingsService.setFrameRateLimit}
      />

      <ChoiceFormRow
        label={"Resolution"}
        description={
          "How many pixels a viewport draws, whatever size the window is. Below the window it costs less and " +
          "reads softer; above it, more, and edges read sharper."
        }
        options={RESOLUTION_OPTIONS}
        value={settingsService.renderResolution}
        onChange={settingsService.setRenderResolution}
      />

      <SettingsRendererFeatures />

      <SettingsRendererShadows />

      <SettingsRendererLod />
    </div>
  );
}
