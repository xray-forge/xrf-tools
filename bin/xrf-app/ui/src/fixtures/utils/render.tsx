import { render, RenderResult } from "@testing-library/react";
import { Container, ContainerConfig } from "@wirestate/core";
import { ContainerProvider } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { Fragment, PropsWithChildren, ReactElement, ReactNode } from "react";
import { MemoryRouter, useLocation } from "react-router-dom";

import { ApplicationProvider } from "@/ApplicationProvider";
import { createContainerPlugins, ROOT_BINDINGS } from "@/core/container";
import { CurrentToolProvider } from "@/core/routing/current-tool.context";
import { IToolDescriptor } from "@/core/routing/tool";
import { IShellContributed, useShellContributions } from "@/core/shell/contributions";
import { EditorLeaveDialog } from "@/core/shell/editor-lifecycle";
import { IEditorPanel, selectPanelsOnSide } from "@/core/shell/editor-shell";
import { TOOL_CATALOG } from "@/ToolCatalog";

export interface IRenderOptions {
  /** Initial route. Components resolve their tool name from it, so it is rarely irrelevant. */
  route?: string;
  /** Services to provide, for components reading them through `useInjection`. */
  bindings?: ContainerConfig["bindings"];
  /** Existing service container to provision and provide instead of creating one from `bindings`. */
  container?: Container;
  /** The subject renders shell outlets itself; omit the fixture's panel and leave-dialog hosts. */
  hasShell?: boolean;
  /** Mount everything twice, the way a development build does, for subjects that hold something across a remount. */
  isStrict?: boolean;
}

/**
 * Renders whatever the subject publishes to the left, for the open tool or the shell, standing in for
 * `ApplicationPanelSlot`.
 */
function LeftPanelsOutlet(): ReactElement {
  const { shell, tool }: IShellContributed<IEditorPanel> = useShellContributions("panels");

  return (
    <>
      {selectPanelsOnSide([...tool, ...shell], "left").map((panel: IEditorPanel) => (
        <Fragment key={panel.id}>{panel.render()}</Fragment>
      ))}
    </>
  );
}

/**
 * Renders content with the tool's test providers.
 *
 * @param ui - Content to render.
 * @param options - Initial route and container bindings.
 * @param options.route - Initial route for the memory router.
 * @param options.bindings - Service bindings added to the test container.
 * @param options.container - Existing service container to provide instead of creating one from bindings.
 * @param options.hasShell - Whether the subject supplies its own shell outlets.
 * @param options.isStrict - Whether to mount twice, as a development build does.
 * @returns The Testing Library render result.
 */
export function renderWithProviders(
  ui: ReactNode,
  { route = "/", bindings = [], container, hasShell = false, isStrict = false }: IRenderOptions = {}
): RenderResult {
  const config: ContainerConfig = {
    bindings: [...ROOT_BINDINGS, ...bindings.filter((it) => !ROOT_BINDINGS.includes(it))],
    plugins: createContainerPlugins(),
  };

  function TestRouter({ children }: PropsWithChildren): ReactElement {
    return <MemoryRouter initialEntries={[route]}>{children}</MemoryRouter>;
  }

  function TestCurrentTool({ children }: PropsWithChildren): ReactElement {
    const { pathname } = useLocation();
    const tool: Nullable<IToolDescriptor> = TOOL_CATALOG.findToolByPath(pathname);

    return <CurrentToolProvider tool={tool}>{children}</CurrentToolProvider>;
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
        <TestCurrentTool>
          <TestContainer>
            {children}

            {hasShell ? null : <LeftPanelsOutlet />}

            {hasShell ? null : <EditorLeaveDialog />}
          </TestContainer>
        </TestCurrentTool>
      </ApplicationProvider>
    );
  }

  const result: RenderResult = render(<>{ui}</>, { wrapper: Wrapper, reactStrictMode: isStrict });

  // Re-wrapped the way the first render was: handed the subject bare, React would see a different element in that
  // position and remount it, losing whatever the subject was keeping across the update being tested.
  return { ...result, rerender: (next: ReactNode): void => result.rerender(<>{next}</>) };
}
