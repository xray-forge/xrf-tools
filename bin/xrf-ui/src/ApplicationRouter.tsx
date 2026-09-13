import { ReactElement } from "react";
import { Route, Routes, useLocation } from "react-router-dom";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { ApplicationLauncher } from "@/core/launcher";
import { IApplicationDescriptor } from "@/core/routing/application";
import { CurrentApplicationProvider } from "@/core/routing/current-application.context";
import { ApplicationShell } from "@/core/shell/ApplicationShell";
import { NavigationError } from "@/core/shell/error/NavigationError";
import { Nullable } from "@/lib/types/general";

/**
 * Maps urls onto applications inside the window chrome that outlives every route.
 */
export function ApplicationRouter(): ReactElement {
  const { pathname } = useLocation();

  const application: Nullable<IApplicationDescriptor> = APPLICATION_CATALOG.findApplicationByPath(pathname);

  return (
    <CurrentApplicationProvider application={application}>
      <ApplicationShell>
        <Routes>
          <Route
            path={"/"}
            element={
              <ApplicationLauncher
                applications={APPLICATION_CATALOG.applications}
                groups={APPLICATION_CATALOG.groups}
              />
            }
          />

          {APPLICATION_CATALOG.applications.map(({ path, Component }: IApplicationDescriptor) => (
            <Route key={path} path={`${path}/*`} element={<Component />} />
          ))}

          <Route path={"*"} element={<NavigationError />} />
        </Routes>
      </ApplicationShell>
    </CurrentApplicationProvider>
  );
}
