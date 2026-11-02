import { ContainerConfig } from "@wirestate/core";
import { ContainerProvider } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ThemeRoot } from "@xrf/ui/theme/root";
import { ComponentType, PropsWithChildren, ReactElement, ReactNode, useMemo } from "react";
import { BrowserRouter } from "react-router-dom";

import { createContainerPlugins, ROOT_BINDINGS } from "@/core/container";
import { THEME_STORAGE_KEY } from "@/core/storage";
import { isDevelopmentBuild } from "@/lib/env/build-mode";

interface IApplicationProviderProps {
  router?: ComponentType<PropsWithChildren>;
  children: ReactNode;
}

export function ApplicationProvider({
  router: Router = BrowserRouter,
  children,
}: IApplicationProviderProps): ReactElement {
  const config: Nullable<ContainerConfig> = useMemo(
    () => ({
      bindings: ROOT_BINDINGS,
      plugins: createContainerPlugins(isDevelopmentBuild()),
    }),
    []
  );

  return (
    <ContainerProvider config={config}>
      <ThemeRoot colorSchemeStorageKey={THEME_STORAGE_KEY}>
        <Router>{children}</Router>
      </ThemeRoot>
    </ContainerProvider>
  );
}
