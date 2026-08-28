import { Box, Checkbox, FormControlLabel, ToggleButton, ToggleButtonGroup } from "@mui/material";
import { useColorScheme } from "@mui/material/styles";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback } from "react";

import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { SettingsService } from "@/core/settings/services/settings";
import { COLOR_SCHEME_MODES, ColorSchemeMode, DEFAULT_COLOR_SCHEME_MODE } from "@/core/theme";
import { Nullable } from "@/lib/types/general";

const COLOR_SCHEME_MODE_LABELS: Record<ColorSchemeMode, string> = {
  light: "Light",
  dark: "Dark",
  system: "System",
};

/** Switches that belong to the application rather than to any one editor. */
export function SettingsGeneralSection(): ReactElement {
  const settingsService: SettingsService = useInjection(SettingsService);

  const { mode, setMode } = useColorScheme();

  const onChangeDevMode = useCallback(
    (_: ChangeEvent<HTMLInputElement>, checked: boolean) => settingsService.setDevModeEnabled(checked),
    [settingsService]
  );

  const onChangeMode = useCallback(
    (_: unknown, value: Nullable<ColorSchemeMode>) => {
      if (value) {
        setMode(value);
      }
    },
    [setMode]
  );

  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 3 }}>
      <SettingsSection
        title={"Appearance"}
        description={"Follow the system theme, or pin the application to one."}
      >
        <ToggleButtonGroup exclusive size={"small"} value={mode ?? DEFAULT_COLOR_SCHEME_MODE} onChange={onChangeMode}>
          {COLOR_SCHEME_MODES.map((it: ColorSchemeMode) => (
            <ToggleButton key={it} value={it}>
              {COLOR_SCHEME_MODE_LABELS[it]}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </SettingsSection>

      <SettingsSection
        title={"Diagnostics"}
        description={
          "Show tracing and captured runtime errors in the notifications panel. Recorded either way, so turning this " +
          "on also reveals what happened before it was switched."
        }
      >
        <FormControlLabel
          control={<Checkbox checked={settingsService.isDevModeEnabled} onChange={onChangeDevMode} />}
          label={"Developer mode"}
        />
      </SettingsSection>
    </Box>
  );
}
