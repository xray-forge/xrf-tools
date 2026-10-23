import { Container, ContainerConfig } from "@wirestate/core";
import { ContainerProvider, useContainer } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { Fragment, ReactElement, ReactNode, use, useMemo } from "react";

import { SESSION_TOOL, SessionScopeService } from "@/core/ipc/session";
import { IApplicationDescriptor } from "@/core/routing/application";

interface IApplicationScopeProps {
  application: Nullable<IApplicationDescriptor>;
  children: ReactNode;
}

/**
 * The container the active application's services live in, holding their openings in the application's own scope,
 * released as the application is left.
 */
export function ApplicationScope({ application, children }: IApplicationScopeProps): ReactElement {
  const parent: Container = useContainer();
  const container = application?.load ? use(application.load()).container : application?.container;
  const config: Nullable<ContainerConfig> = useMemo(
    () =>
      container && application
        ? {
            ...container,
            bindings: [
              { token: SESSION_TOOL, value: application.id },
              SessionScopeService,
              ...(container.bindings ?? []),
            ],
            parent,
          }
        : null,
    [application, container, parent]
  );

  return config ? <ContainerProvider config={config}>{children}</ContainerProvider> : <Fragment>{children}</Fragment>;
}
