import { Nullable } from "@xrf/types";
import { ReactElement } from "react";
import { Route, Routes, useLocation } from "react-router-dom";

import { ToolLauncher } from "@/core/launcher";
import { CurrentToolProvider } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";
import { ApplicationShell } from "@/core/shell/ApplicationShell";
import { NavigationError } from "@/core/shell/error/NavigationError";

import { TOOL_CATALOG } from "./ToolCatalog";

/**
 * Maps urls onto tools inside the window chrome that outlives every route.
 */
export function ApplicationRouter(): ReactElement {
  const { pathname } = useLocation();

  const tool: Nullable<IToolDescriptor> = TOOL_CATALOG.findToolByPath(pathname);

  return (
    <CurrentToolProvider tool={tool}>
      <ApplicationShell>
        <Routes>
          <Route path={"/"} element={<ToolLauncher tools={TOOL_CATALOG.tools} groups={TOOL_CATALOG.groups} />} />

          {TOOL_CATALOG.tools.map(({ path, Component }: IToolDescriptor) => (
            <Route key={path} path={`${path}/*`} element={<Component />} />
          ))}

          <Route path={"*"} element={<NavigationError />} />
        </Routes>
      </ApplicationShell>
    </CurrentToolProvider>
  );
}
