import { useInjection } from "@wirestate/react";
import { Maybe, Nullable } from "@xrf/types";
import { DarkModeIcon, LightModeIcon, SettingsIcon } from "@xrf/ui/icons";
import { useColorScheme } from "@xrf/ui/material";
import { RailButton } from "@xrf/ui/shell/panel/rail/RailButton";
import { ReactElement, useCallback } from "react";

import { ApplicationHelp } from "@/core/help/components/ApplicationHelp/ApplicationHelp";
import { SettingsDialogService } from "@/core/settings/services/settings-dialog";
import { IEditorPanel } from "@/core/shell/editor-shell";
import { ApplicationPanelStripe } from "@/core/shell/panel/ApplicationPanelStripe";

interface IApplicationRailProps {
  panels: Array<IEditorPanel>;
  activePanelId: Nullable<string>;
  onTogglePanel: (id: string) => void;
}

/**
 * The left edge: the active application's navigation panels, then the window's own controls.
 */
export function ApplicationRail({ panels, activePanelId, onTogglePanel }: IApplicationRailProps): ReactElement {
  const { mode, setMode, systemMode } = useColorScheme();

  const settingsDialogService: SettingsDialogService = useInjection(SettingsDialogService);

  const resolvedMode: Maybe<string> = mode === "system" ? systemMode : mode;
  const isLightMode: boolean = resolvedMode === "light";

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
            icon={isLightMode ? <DarkModeIcon /> : <LightModeIcon />}
            appearance={isLightMode ? "secondary" : "primary"}
            onClick={onToggleTheme}
          />

          <ApplicationHelp />

          <RailButton label={"Settings"} icon={<SettingsIcon />} onClick={settingsDialogService.open} />
        </>
      }
      onTogglePanel={onTogglePanel}
    />
  );
}
