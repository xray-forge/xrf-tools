import { render, RenderResult } from "@testing-library/react";
import { Container, ContainerConfig } from "@wirestate/core";
import { ContainerProvider } from "@wirestate/react";
import { Fragment, PropsWithChildren, ReactElement, ReactNode } from "react";
import { MemoryRouter, useLocation } from "react-router-dom";

import { APPLICATION_CATALOG } from "@/ApplicationCatalog";
import { ApplicationProvider } from "@/ApplicationProvider";
import { createContainerPlugins } from "@/core/container";
import { NotificationsService } from "@/core/notifications/services";
import { IApplicationDescriptor } from "@/core/routing/application";
import { CurrentApplicationProvider } from "@/core/routing/current-application.context";
import { SettingsService } from "@/core/settings/services/settings";
import { EditorBusyProvider } from "@/core/shell/EditorBusyContext";
import { EditorStatusProvider } from "@/core/shell/EditorStatusContext";
import {
  EditorPanelsProvider,
  IEditorPanel,
  selectPanelsOnSide,
  useEditorPanelsRegistry,
} from "@/core/shell/panel/context";
import { Nullable } from "@/lib/types/general";

export interface IRenderOptions {
  /** Initial route. Components resolve their application name from it, so it is rarely irrelevant. */
  route?: string;
  /** Services to provide, for components reading them through `useInjection`. */
  bindings?: ContainerConfig["bindings"];
  /** Existing service container to provision and provide instead of creating one from `bindings`. */
  container?: Container;
}

/**
 * Renders whatever the subject publishes to the left, standing in for `ApplicationPanelSlot`.
 */
function LeftPanelsOutlet(): ReactElement {
  const panels: ReadonlyArray<IEditorPanel> = useEditorPanelsRegistry();

  return (
    <>
      {selectPanelsOnSide(panels, "left").map((panel: IEditorPanel) => (
        <Fragment key={panel.id}>{panel.render()}</Fragment>
      ))}
    </>
  );
}

/**
 * Renders content with the application's test providers.
 *
 * @param ui - Content to render.
 * @param options - Initial route and container bindings.
 * @param options.route - Initial route for the memory router.
 * @param options.bindings - Service bindings added to the test container.
 * @param options.container - Existing service container to provide instead of creating one from bindings.
 * @returns The Testing Library render result.
 */
export function renderWithProviders(
  ui: ReactNode,
  { route = "/", bindings = [], container }: IRenderOptions = {}
): RenderResult {
  const config: ContainerConfig = {
    bindings: [NotificationsService, SettingsService, ...bindings],
    plugins: createContainerPlugins(),
  };

  function TestRouter({ children }: PropsWithChildren): ReactElement {
    return <MemoryRouter initialEntries={[route]}>{children}</MemoryRouter>;
  }

  function TestCurrentApplication({ children }: PropsWithChildren): ReactElement {
    const { pathname } = useLocation();
    const application: Nullable<IApplicationDescriptor> = APPLICATION_CATALOG.findApplicationByPath(pathname);

    return <CurrentApplicationProvider application={application}>{children}</CurrentApplicationProvider>;
  }

  function TestContainer({ children }: PropsWithChildren): ReactElement {
    return container ? (
      <ContainerProvider container={container}>{children}</ContainerProvider>
    ) : (
      <ContainerProvider config={config}>{children}</ContainerProvider>
    );
  }

  function Wrapper({ children }: PropsWithChildren): ReactElement {
    return (
      <ApplicationProvider router={TestRouter}>
        <TestCurrentApplication>
          <TestContainer>
            <EditorBusyProvider>
              <EditorStatusProvider>
                <EditorPanelsProvider>
                  {children}
                  <LeftPanelsOutlet />
                </EditorPanelsProvider>
              </EditorStatusProvider>
            </EditorBusyProvider>
          </TestContainer>
        </TestCurrentApplication>
      </ApplicationProvider>
    );
  }

  return render(<>{ui}</>, { wrapper: Wrapper });
}
