import { Container, ContainerConfig } from "@wirestate/core";
import { ContainerProvider, useContainer } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { LauncherKeybindsService } from "@/core/launcher/services/launcher-keybinds.service";

import { IToolLauncherProps, ToolLauncherScreen } from "./ToolLauncherScreen";

/**
 * The searchable home surface for launching tools.
 */
export function ToolLauncher(props: IToolLauncherProps): ReactElement {
  const parent: Container = useContainer();
  const config: ContainerConfig = useMemo(() => ({ bindings: [LauncherKeybindsService], parent }), [parent]);

  return (
    <ContainerProvider config={config}>
      <ToolLauncherScreen {...props} />
    </ContainerProvider>
  );
}
