import { Box, Checkbox, Divider, FormControlLabel, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { useColorScheme } from "@mui/material/styles";
import { open } from "@tauri-apps/plugin-dialog";
import { useInjection } from "@wirestate/react";
import { ChangeEvent, ReactElement, useCallback } from "react";

import { SettingsPathField } from "@/core/settings/components/SettingsPathField";
import { ProjectService } from "@/core/settings/services/project";
import { SettingsService } from "@/core/settings/services/settings";
import { COLOR_SCHEME_MODES, ColorSchemeMode, DEFAULT_COLOR_SCHEME_MODE } from "@/core/theme";
import { Logger, useLogger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

const COLOR_SCHEME_MODE_LABELS: Record<ColorSchemeMode, string> = {
  light: "Light",
  dark: "Dark",
  system: "System",
};

export function SettingsForm(): ReactElement {
  const log: Logger = useLogger(__MODULE_NAME__);

  const projectService: ProjectService = useInjection(ProjectService);
  const settingsService: SettingsService = useInjection(SettingsService);

  const { mode, setMode } = useColorScheme();

  const onChangeDevMode = useCallback(
    (_: ChangeEvent<HTMLInputElement>, checked: boolean) => settingsService.setDevModeEnabled(checked),
    [settingsService]
  );

  const onSelectProjectPath = useCallback(async () => {
    const newXrfProjectPath: Nullable<string> = await open({
      title: "Select project directory",
      directory: true,
    });

    if (newXrfProjectPath) {
      log.info("Selected new project path:", newXrfProjectPath);

      projectService.setXrfProjectPath(newXrfProjectPath);
    }
  }, [log, projectService]);

  const onClearProjectPath = useCallback(() => projectService.setXrfProjectPath(null), [projectService]);

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
      <Box>
        <Typography variant={"subtitle2"}>Appearance</Typography>

        <Typography variant={"caption"} sx={{ display: "block", color: "text.secondary", marginBottom: 1 }}>
          Follow the system theme, or pin the application to one.
        </Typography>

        <ToggleButtonGroup exclusive size={"small"} value={mode ?? DEFAULT_COLOR_SCHEME_MODE} onChange={onChangeMode}>
          {COLOR_SCHEME_MODES.map((it: ColorSchemeMode) => (
            <ToggleButton key={it} value={it}>
              {COLOR_SCHEME_MODE_LABELS[it]}
            </ToggleButton>
          ))}
        </ToggleButtonGroup>
      </Box>

      <Divider />

      <SettingsPathField
        label={"Project"}
        description={"Root of the xrf project. Tools use it to guess a starting path when they have none."}
        value={projectService.xrfProjectPath}
        onSelect={onSelectProjectPath}
        onClear={onClearProjectPath}
      />

      <Divider />

      <Box>
        <Typography variant={"subtitle2"}>Diagnostics</Typography>

        <Typography variant={"caption"} sx={{ display: "block", color: "text.secondary", marginBottom: 1 }}>
          Show tracing and captured runtime errors in the notifications panel. Recorded either way, so turning this on
          also reveals what happened before it was switched.
        </Typography>

        <FormControlLabel
          control={<Checkbox checked={settingsService.isDevModeEnabled} onChange={onChangeDevMode} />}
          label={"Developer mode"}
        />
      </Box>
    </Box>
  );
}
