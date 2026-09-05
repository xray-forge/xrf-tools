import { Container, ContainerConfig } from "@wirestate/core";
import { ContainerProvider, useContainer } from "@wirestate/react";
import { Fragment, ReactElement, ReactNode, useMemo } from "react";

import { IApplicationDescriptor } from "@/core/routing/application";
import { Nullable } from "@/lib/types/general";

interface IApplicationScopeProps {
  application: Nullable<IApplicationDescriptor>;
  children: ReactNode;
}

/**
 * The container the active application's services live in.
 */
export function ApplicationScope({ application, children }: IApplicationScopeProps): ReactElement {
  const parent: Container = useContainer();
  const config: Nullable<ContainerConfig> = useMemo(
    () => (application?.container ? { ...application.container, parent } : null),
    [application, parent]
  );

  return config ? <ContainerProvider config={config}>{children}</ContainerProvider> : <Fragment>{children}</Fragment>;
}
