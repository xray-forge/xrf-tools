import { Container, ContainerConfig } from "@wirestate/core";
import { ContainerProvider, useContainer } from "@wirestate/react";
import { ReactElement, useMemo } from "react";

import { LauncherKeybindsService } from "@/core/launcher/services/launcher-keybinds";

import { ApplicationLauncherScreen, IApplicationLauncherProps } from "./ApplicationLauncherScreen";

/**
 * The searchable home surface for launching applications.
 */
export function ApplicationLauncher(props: IApplicationLauncherProps): ReactElement {
  const parent: Container = useContainer();
  const config: ContainerConfig = useMemo(() => ({ bindings: [LauncherKeybindsService], parent }), [parent]);

  return (
    <ContainerProvider config={config}>
      <ApplicationLauncherScreen {...props} />
    </ContainerProvider>
  );
}
