import { useInjection } from "@wirestate/react";
import { Maybe, Nullable } from "@xrf/types";
import { DarkModeIcon } from "@xrf/ui/icons/dark-mode";
import { LightModeIcon } from "@xrf/ui/icons/light-mode";
import { SettingsIcon } from "@xrf/ui/icons/settings";
import { useColorScheme } from "@xrf/ui/material/styles";
import { RailButton } from "@xrf/ui/shell/rail-button";
import { ReactElement, useCallback } from "react";

import { ToolHelp } from "@/core/help/components/ToolHelp/ToolHelp";
import { SettingsDialogService } from "@/core/settings/services/settings-dialog";
import { IEditorPanel } from "@/core/shell/editor-shell";
import { ApplicationPanelStripe } from "@/core/shell/panel/ApplicationPanelStripe";

interface IApplicationRailProps {
  panels: ReadonlyArray<IEditorPanel>;
  shellPanels: ReadonlyArray<IEditorPanel>;
  activePanelId: Nullable<string>;
  onTogglePanel: (id: string) => void;
}

/**
 * The left edge: the active tool's navigation panels, then the shell's own panels and the window's own controls.
 */
export function ApplicationRail({
  panels,
  shellPanels,
  activePanelId,
  onTogglePanel,
}: IApplicationRailProps): ReactElement {
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
      shellPanels={shellPanels}
      activePanelId={activePanelId}
      footer={
        <>
          <RailButton
            label={isLightMode ? "Dark theme" : "Light theme"}
            icon={isLightMode ? <DarkModeIcon /> : <LightModeIcon />}
            appearance={isLightMode ? "secondary" : "primary"}
            onClick={onToggleTheme}
          />

          <ToolHelp />

          <RailButton label={"Settings"} icon={<SettingsIcon />} onClick={settingsDialogService.open} />
        </>
      }
      onTogglePanel={onTogglePanel}
    />
  );
}
