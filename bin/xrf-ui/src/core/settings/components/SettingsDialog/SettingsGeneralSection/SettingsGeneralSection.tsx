import { useColorScheme } from "@mui/material/styles";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { canRenderOffscreen } from "@/core/render/lib/frame/offscreen-render-target";
import { FRAME_RATE_LIMITS, TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { SettingsService } from "@/core/settings/services/settings";
import { COLOR_SCHEME_MODES, ColorSchemeMode, DEFAULT_COLOR_SCHEME_MODE } from "@/core/theme";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow, IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow";

const COLOR_SCHEME_MODE_LABELS: Record<ColorSchemeMode, string> = {
  light: "Light",
  dark: "Dark",
  system: "System",
};

const COLOR_SCHEME_OPTIONS: ReadonlyArray<IChoiceFormRowOption<ColorSchemeMode>> = COLOR_SCHEME_MODES.map((value) => ({
  value,
  label: COLOR_SCHEME_MODE_LABELS[value],
}));

const FRAME_RATE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<TFrameRateLimit>> = FRAME_RATE_LIMITS.map((value) => ({
  value,
  label: value === "unlimited" ? "Unlimited" : `${value} fps`,
}));

/** Switches that belong to the application rather than to any one editor. */
export function SettingsGeneralSection(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const { mode, setMode } = useColorScheme();

  return (
    <div className={"flex flex-col gap-6"}>
      <ChoiceFormRow
        label={"Appearance"}
        description={"Follow the system theme, or pin the application to one."}
        options={COLOR_SCHEME_OPTIONS}
        value={mode ?? DEFAULT_COLOR_SCHEME_MODE}
        onChange={setMode}
      />

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
          "Keeps the interface responsive while a level streams. The same picture either way, and off unless " +
          "this display can hand a canvas to another thread."
        }
        isChecked={settingsService.isOffscreenRenderEnabled && canRenderOffscreen()}
        isDisabled={!canRenderOffscreen()}
        onChange={settingsService.setOffscreenRenderEnabled}
      />

      <CheckboxFormRow
        label={"Developer mode"}
        description={"Show tracing and captured runtime errors, advanced debug tools."}
        isChecked={settingsService.isDevModeEnabled}
        onChange={settingsService.setDevModeEnabled}
      />
    </div>
  );
}
