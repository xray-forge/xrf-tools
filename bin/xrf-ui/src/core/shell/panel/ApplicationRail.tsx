import { default as DarkModeIcon } from "@mui/icons-material/DarkMode";
import { default as GitHubIcon } from "@mui/icons-material/GitHub";
import { default as LightModeIcon } from "@mui/icons-material/LightMode";
import { default as SettingsIcon } from "@mui/icons-material/Settings";
import { useColorScheme } from "@mui/material/styles";
import { open } from "@tauri-apps/plugin-shell";
import { ReactElement, useCallback, useState } from "react";

import { REPOSITORY_URL } from "@/core/configs";
import { SettingsDialog } from "@/core/settings/components/SettingsDialog";
import { ApplicationPanelStripe } from "@/core/shell/panel/ApplicationPanelStripe";
import { IEditorPanel } from "@/core/shell/panel/context";
import { RailButton } from "@/core/shell/panel/RailButton";
import { Logger } from "@/lib/logging";
import { Maybe, Nullable } from "@/lib/types/general";

export interface IApplicationRailProps {
  panels: Array<IEditorPanel>;
  activePanelId: Nullable<string>;
  onTogglePanel: (id: string) => void;
}

/**
 * The left edge: the active application's navigation panels, then the window's own controls.
 */
export function ApplicationRail({ panels, activePanelId, onTogglePanel }: IApplicationRailProps): ReactElement {
  const { mode, setMode, systemMode } = useColorScheme();

  const [isSettingsOpen, setSettingsOpen] = useState(false);

  const resolvedMode: Maybe<string> = mode === "system" ? systemMode : mode;
  const isLightMode: boolean = resolvedMode === "light";

  const onOpenGithubLink = useCallback(() => {
    open(REPOSITORY_URL).catch(Logger.error);
  }, []);

  const onToggleTheme = useCallback(() => {
    setMode(isLightMode ? "dark" : "light");
  }, [isLightMode, setMode]);

  return (
    <ApplicationPanelStripe
      side={"left"}
      panels={panels}
      activePanelId={activePanelId}
      footer={
        <>
          <RailButton
            label={isLightMode ? "Dark theme" : "Light theme"}
            icon={isLightMode ? <DarkModeIcon fontSize={"small"} /> : <LightModeIcon fontSize={"small"} />}
            onClick={onToggleTheme}
          />

          <RailButton label={"Source on github"} icon={<GitHubIcon fontSize={"small"} />} onClick={onOpenGithubLink} />

          <RailButton
            label={"Settings"}
            icon={<SettingsIcon fontSize={"small"} />}
            onClick={() => setSettingsOpen(true)}
          />

          <SettingsDialog isOpen={isSettingsOpen} onClose={() => setSettingsOpen(false)} />
        </>
      }
      onTogglePanel={onTogglePanel}
    />
  );
}
