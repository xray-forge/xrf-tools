import { Container, ContainerConfig } from "@wirestate/core";
import { ContainerProvider, useContainer } from "@wirestate/react";
import { SESSION_TOOL, SessionScopeService } from "@xrf/ipc/session";
import { Nullable } from "@xrf/types";
import { Fragment, ReactElement, ReactNode, use, useMemo } from "react";

import { IToolDescriptor } from "@/core/routing/tool";

interface IToolScopeProps {
  tool: Nullable<IToolDescriptor>;
  children: ReactNode;
}

/**
 * The container the active tool's services live in, holding their openings in the tool's own scope,
 * released as the tool is left.
 */
export function ToolScope({ tool, children }: IToolScopeProps): ReactElement {
  const parent: Container = useContainer();
  const container = tool?.load ? use(tool.load()).container : tool?.container;
  const config: Nullable<ContainerConfig> = useMemo(
    () =>
      container && tool
        ? {
            ...container,
            bindings: [{ token: SESSION_TOOL, value: tool.id }, SessionScopeService, ...(container.bindings ?? [])],
            parent,
          }
        : null,
    [tool, container, parent]
  );

  return config ? <ContainerProvider config={config}>{children}</ContainerProvider> : <Fragment>{children}</Fragment>;
}
