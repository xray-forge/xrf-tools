import { Box } from "@mui/material";
import { useColorScheme } from "@mui/material/styles";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

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

/** Switches that belong to the application rather than to any one editor. */
export function SettingsGeneralSection(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const { mode, setMode } = useColorScheme();

  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
      <ChoiceFormRow
        label={"Appearance"}
        description={"Follow the system theme, or pin the application to one."}
        options={COLOR_SCHEME_OPTIONS}
        value={mode ?? DEFAULT_COLOR_SCHEME_MODE}
        onChange={setMode}
      />

      <CheckboxFormRow
        label={"Developer mode"}
        description={
          "Show tracing and captured runtime errors in the notifications panel. Recorded either way, so turning this " +
          "on also reveals what happened before it was switched."
        }
        isChecked={settingsService.isDevModeEnabled}
        onChange={settingsService.setDevModeEnabled}
      />
    </Box>
  );
}
