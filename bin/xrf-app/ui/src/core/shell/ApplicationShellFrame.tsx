import { useInjection, useOnCommand, useOnQuery } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { EditorToolbarHostContext } from "@xrf/ui/editor/toolbar";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { RailButton } from "@xrf/ui/shell/rail-button";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode, Suspense, useCallback, useState } from "react";
import { flushSync } from "react-dom";
import { NavigateFunction, useLocation, useNavigate } from "react-router-dom";

import { ErrorBoundary, IErrorBoundaryFallbackProps } from "@/core/error/components/ErrorBoundary";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { useCurrentTool } from "@/core/routing/current-tool.context";
import { APPLICATION_SOURCE, IToolDescriptor } from "@/core/routing/tool";
import { SettingsService } from "@/core/settings/services/settings";
import { useIsEditorBusy } from "@/core/shell/editor-lifecycle";
import { IEditorPanel, selectPanelsOnSide, useEditorPanelsRegistry } from "@/core/shell/editor-shell";
import { ToolCrash } from "@/core/shell/error/ToolCrash";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { ToolLoader } from "@/core/shell/loading/ToolLoader";
import { ApplicationPanelSlot } from "@/core/shell/panel/ApplicationPanelSlot";
import { ApplicationPanelStripe } from "@/core/shell/panel/ApplicationPanelStripe";
import { JOBS_PANEL } from "@/core/shell/panel/jobs/jobs-panel";
import { NOTIFICATIONS_PANEL } from "@/core/shell/panel/notifications/notification-panel";
import {
  IPanelSetActiveCommand,
  IPanelSideCommand,
  PANEL_ACTIVE_QUERY,
  PANEL_CLOSE_COMMAND,
  PANEL_SET_ACTIVE_COMMAND,
} from "@/core/shell/panel/panel-messages";
import { ApplicationRail } from "@/core/shell/panel/rail";
import { IPanelSelection, usePanelSelection } from "@/core/shell/panel/use-panel-selection";
import { IPanelWidth, usePanelWidth } from "@/core/shell/panel/use-panel-width";
import { CLOSE_TOOL_COMMAND, TOOL_OPEN_QUERY } from "@/core/shell/shell-messages";
import { ApplicationTitleBar } from "@/core/shell/title-bar/ApplicationTitleBar";
import { ToolScope } from "@/core/shell/ToolScope";

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
  const navigate: NavigateFunction = useNavigate();
  const panels: ReadonlyArray<IEditorPanel> = useEditorPanelsRegistry();

  const { pathname } = useLocation();

  const isBusy: boolean = useIsEditorBusy();
  const settingsService: SettingsService = useInjection(SettingsService);

  // The element the routed content portals its toolbar into. Held here rather than in a provider of
  // its own: the frame hands it down and never reads it back.
  const [toolbarHost, setToolbarHost] = useState<Nullable<HTMLElement>>(null);

  const toolPath: string = tool?.path ?? "root";

  const leftPanels: Array<IEditorPanel> = selectPanelsOnSide(panels, "left");
  const toolRightPanels: Array<IEditorPanel> = selectPanelsOnSide(panels, "right");
  // Registered rather than merely hidden, so a build without dev mode never mounts a panel that polls the backend.
  const rightPanels: Array<IEditorPanel> = [
    ...toolRightPanels,
    ...(settingsService.isDevModeEnabled ? [JOBS_PANEL] : []),
    NOTIFICATIONS_PANEL,
  ];

  const leftSelection: IPanelSelection = usePanelSelection("left", leftPanels, tool?.id ?? "root");
  const rightSelection: IPanelSelection = usePanelSelection("right", rightPanels, "global");

  const openCount: number = (leftSelection.activePanel ? 1 : 0) + (rightSelection.activePanel ? 1 : 0);
  const leftSizing: IPanelWidth = usePanelWidth("left", openCount);
  const rightSizing: IPanelWidth = usePanelWidth("right", openCount);

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

  // Flushed so the command is synchronous for its caller: whoever opens a panel to reach something inside it finds
  // that content mounted when the dispatch returns, with no frame to wait for and no retry to write.
  useOnCommand(PANEL_SET_ACTIVE_COMMAND, ({ side, panelId }: IPanelSetActiveCommand) =>
    flushSync(() => (side === "left" ? leftSelection : rightSelection).onOpenPanel(panelId))
  );

  // Closing a side is toggling whatever it has open, which is what `usePanelSelection` already means by it.
  useOnCommand(PANEL_CLOSE_COMMAND, ({ side }: IPanelSideCommand) => {
    const selection: IPanelSelection = side === "left" ? leftSelection : rightSelection;

    if (selection.activePanelId) {
      selection.onTogglePanel(selection.activePanelId);
    }
  });

  useOnCommand(CLOSE_TOOL_COMMAND, () => navigate("/", { replace: true }));

  useOnQuery(TOOL_OPEN_QUERY, () => tool !== null);

  useOnQuery(
    PANEL_ACTIVE_QUERY,
    ({ side }: IPanelSideCommand) => (side === "left" ? leftSelection : rightSelection).activePanelId
  );

  return (
    <EditorToolbarHostContext.Provider value={toolbarHost}>
      <div
        data-testid={dataTestId}
        id={id}
        className={cn("flex h-full w-full flex-col flex-nowrap surface-frame", className)}
      >
        <ApplicationTitleBar toolbarRef={setToolbarHost} isBusy={isBusy} />

        <div className={"flex min-h-0 grow flex-nowrap"}>
          <ApplicationRail
            panels={leftPanels}
            activePanelId={leftSelection.activePanelId}
            onTogglePanel={leftSelection.onTogglePanel}
          />

          <div className={"flex min-h-0 min-w-0 grow overflow-hidden rounded-surface border border-divider"}>
            <ErrorBoundary resetKey={pathname} fallback={onError} onCaught={onCaught}>
              <Suspense key={toolPath} fallback={<ToolLoader />}>
                <ToolScope tool={tool}>
                  <ApplicationPanelSlot
                    side={"left"}
                    panel={leftSelection.activePanel}
                    width={leftSizing.width}
                    onResize={leftSizing.onResize}
                  />

                  <div className={"flex min-h-0 min-w-0 grow overflow-hidden"}>
                    <ErrorBoundary resetKey={pathname} fallback={onError} onCaught={onCaught}>
                      {children}
                    </ErrorBoundary>
                  </div>

                  <ApplicationPanelSlot
                    side={"right"}
                    panel={rightSelection.activePanel}
                    width={rightSizing.width}
                    onResize={rightSizing.onResize}
                  />
                </ToolScope>
              </Suspense>
            </ErrorBoundary>
          </div>

          <ApplicationPanelStripe
            side={"right"}
            panels={toolRightPanels}
            activePanelId={rightSelection.activePanelId}
            footer={
              <>
                {settingsService.isDevModeEnabled ? (
                  <RailButton
                    label={JOBS_PANEL.label}
                    icon={JOBS_PANEL.icon}
                    tooltipPlacement={"left"}
                    isSelected={rightSelection.activePanelId === JOBS_PANEL.id}
                    onClick={() => rightSelection.onTogglePanel(JOBS_PANEL.id)}
                  />
                ) : null}

                <RailButton
                  label={NOTIFICATIONS_PANEL.label}
                  icon={NOTIFICATIONS_PANEL.icon}
                  tooltipPlacement={"left"}
                  isSelected={rightSelection.activePanelId === NOTIFICATIONS_PANEL.id}
                  onClick={() => rightSelection.onTogglePanel(NOTIFICATIONS_PANEL.id)}
                />
              </>
            }
            onTogglePanel={rightSelection.onTogglePanel}
          />
        </div>

        <ApplicationStatusBar />
      </div>
    </EditorToolbarHostContext.Provider>
  );
}
