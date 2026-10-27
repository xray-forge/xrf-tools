import { useOnCommand, useOnQuery } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { EditorToolbarHostContext } from "@xrf/ui/editor/toolbar";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode, Suspense, useCallback, useState } from "react";
import { flushSync } from "react-dom";
import { useLocation } from "react-router-dom";

import { ErrorBoundary, IErrorBoundaryFallbackProps } from "@/core/error/components/ErrorBoundary";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { useCurrentTool } from "@/core/routing/current-tool.context";
import { APPLICATION_SOURCE, IToolDescriptor } from "@/core/routing/tool";
import { ToolLoader } from "@/core/shell/chrome/ToolLoader";
import { ToolCrash } from "@/core/shell/error/ToolCrash";
import { ToolScope } from "@/core/shell/frame/ToolScope";
import { useShellToolRoute } from "@/core/shell/frame/use-shell-tool-route";
import { useIsToolBusy } from "@/core/shell/lifecycle";
import { useShellOwnMenus } from "@/core/shell/menu/use-shell-own-menus";
import { ApplicationPanelSlot } from "@/core/shell/panels/ApplicationPanelSlot";
import { ApplicationPanelStripe } from "@/core/shell/panels/ApplicationPanelStripe";
import {
  IPanelSetActiveCommand,
  IPanelSideCommand,
  PANEL_ACTIVE_QUERY,
  PANEL_CLOSE_COMMAND,
  PANEL_SET_ACTIVE_COMMAND,
} from "@/core/shell/panels/panel-messages";
import { ApplicationRail } from "@/core/shell/panels/rail";
import { TFrameSides, useFrameSides } from "@/core/shell/panels/use-frame-sides";
import { useShellOwnPanels } from "@/core/shell/panels/use-shell-own-panels";
import { ApplicationStatusBar } from "@/core/shell/status-bar/ApplicationStatusBar";
import { ApplicationTitleBar } from "@/core/shell/title-bar/ApplicationTitleBar";

interface IApplicationShellFrameProps extends BaseComponentProps {
  children: ReactNode;
}

/**
 * The window frame itself: rail and its panel on the left, panel and stripe on the right, status bar
 * along the bottom.
 */
export function ApplicationShellFrame({
  "data-testid": dataTestId = "application-shell-frame",
  id = "application-shell-frame",
  className,
  children,
}: IApplicationShellFrameProps): ReactElement {
  const tool: Nullable<IToolDescriptor> = useCurrentTool();
  const notify: TEmitNotification = useEmitNotification();
  const sides: TFrameSides = useFrameSides();

  const { pathname } = useLocation();

  const isBusy: boolean = useIsToolBusy();

  // The element the routed content portals its toolbar into. Held here rather than in a provider of
  // its own: the frame hands it down and never reads it back.
  const [toolbarHost, setToolbarHost] = useState<Nullable<HTMLElement>>(null);

  const toolPath: string = tool?.path ?? "root";

  const onError = useCallback((props: IErrorBoundaryFallbackProps) => <ToolCrash {...props} />, []);

  const onCaught = useCallback(
    (error: Error, componentStack: Nullable<string>) =>
      notify({
        details: componentStack ? `${error.message}\n${componentStack}` : error.message,
        severity: ENotificationSeverity.ERROR,
        source: tool?.id ?? APPLICATION_SOURCE,
        title: "The interface crashed and was replaced",
      }),
    [tool, notify]
  );

  // The shell contributes its own entries and panels the way a tool does, under its own key.
  useShellOwnPanels();
  useShellOwnMenus();
  useShellToolRoute();

  // Flushed so the command is synchronous for its caller: whoever opens a panel to reach something inside it finds
  // that content mounted when the dispatch returns, with no frame to wait for and no retry to write.
  useOnCommand(PANEL_SET_ACTIVE_COMMAND, ({ side, panelId }: IPanelSetActiveCommand) =>
    flushSync(() => sides[side].onOpenPanel(panelId))
  );

  // Closing a side is toggling whatever it has open, which is what `usePanelSelection` already means by it.
  useOnCommand(PANEL_CLOSE_COMMAND, ({ side }: IPanelSideCommand) => {
    const { activePanelId, onTogglePanel } = sides[side];

    if (activePanelId) {
      onTogglePanel(activePanelId);
    }
  });

  useOnQuery(PANEL_ACTIVE_QUERY, ({ side }: IPanelSideCommand) => sides[side].activePanelId);

  return (
    <EditorToolbarHostContext.Provider value={toolbarHost}>
      <div
        data-testid={dataTestId}
        id={id}
        className={cn("flex h-full w-full flex-col flex-nowrap surface-frame", className)}
      >
        <ApplicationTitleBar toolbarRef={setToolbarHost} isBusy={isBusy} />

        <div className={"flex min-h-0 grow flex-nowrap"}>
          <ApplicationRail frameSide={sides.left} />

          <div className={"flex min-h-0 min-w-0 grow overflow-hidden rounded-surface border border-divider"}>
            <ErrorBoundary resetKey={pathname} fallback={onError} onCaught={onCaught}>
              <Suspense key={toolPath} fallback={<ToolLoader />}>
                <ToolScope tool={tool}>
                  <ApplicationPanelSlot frameSide={sides.left} />

                  <div className={"flex min-h-0 min-w-0 grow overflow-hidden"}>
                    <ErrorBoundary resetKey={pathname} fallback={onError} onCaught={onCaught}>
                      {children}
                    </ErrorBoundary>
                  </div>

                  <ApplicationPanelSlot frameSide={sides.right} />
                </ToolScope>
              </Suspense>
            </ErrorBoundary>
          </div>

          <ApplicationPanelStripe frameSide={sides.right} />
        </div>

        <ApplicationStatusBar />
      </div>
    </EditorToolbarHostContext.Provider>
  );
}
