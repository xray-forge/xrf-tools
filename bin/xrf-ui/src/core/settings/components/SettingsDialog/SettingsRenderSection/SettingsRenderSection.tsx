import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { FRAME_RATE_LIMITS, TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { SettingsService } from "@/core/settings/services/settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";
import { canRenderOffscreen } from "@/lib/dom/canvas";

const FRAME_RATE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TFrameRateLimit>> = FRAME_RATE_LIMITS.map((value) => ({
  value,
  label: value === "unlimited" ? "Unlimited" : `${value} fps`,
}));

/** How every viewport draws, which is neither application chrome nor any one editor's business. */
export function SettingsRenderSection(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const isOffscreenAvailable: boolean = canRenderOffscreen();

  return (
    <div className={"flex flex-col gap-6"}>
      <ChoiceFormRow
        label={"Frame rate limit"}
        description={"How often a viewport redraws. A display faster than this costs power for frames nobody sees."}
        options={FRAME_RATE_OPTIONS}
        value={settingsService.frameRateLimit}
        onChange={settingsService.setFrameRateLimit}
      />

      <CheckboxFormRow
        label={"Draw levels on their own thread"}
        description={
          "Keeps the interface responsive while a level streams, for the same picture either way. Turn it off to " +
          "draw on the thread the interface runs on."
        }
        isChecked={settingsService.isOffscreenRenderEnabled && isOffscreenAvailable}
        isDisabled={!isOffscreenAvailable}
        onChange={settingsService.setOffscreenRenderEnabled}
      />
    </div>
  );
}
