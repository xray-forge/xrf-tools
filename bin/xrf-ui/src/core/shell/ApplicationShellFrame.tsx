import { Box } from "@mui/material";
import { useInjection, useOnCommand, useOnQuery } from "@wirestate/react";
import { ReactElement, ReactNode, Suspense, useCallback, useState } from "react";
import { flushSync } from "react-dom";
import { useLocation } from "react-router-dom";

import { ErrorBoundary, IErrorBoundaryFallbackProps } from "@/core/error/components/ErrorBoundary";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { APPLICATION_SOURCE, IApplicationDescriptor } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { SettingsService } from "@/core/settings/services/settings";
import { ApplicationScope } from "@/core/shell/ApplicationScope";
import { useIsEditorBusy } from "@/core/shell/editor-lifecycle";
import { IEditorPanel, selectPanelsOnSide, useEditorPanelsRegistry } from "@/core/shell/editor-shell";
import { ApplicationCrash } from "@/core/shell/error/ApplicationCrash";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { EditorToolbarHostContext } from "@/core/shell/header/editor-toolbar-host";
import { ApplicationLoader } from "@/core/shell/loading/ApplicationLoader";
import { ApplicationPanelSlot } from "@/core/shell/panel/ApplicationPanelSlot";
import { ApplicationPanelStripe } from "@/core/shell/panel/ApplicationPanelStripe";
import { JOBS_PANEL } from "@/core/shell/panel/jobs/jobs-panel";
import { NOTIFICATIONS_PANEL } from "@/core/shell/panel/notifications/notification-panel";
import {
  IPanelSetActiveMessage,
  IPanelSideMessage,
  PANEL_ACTIVE_QUERY,
  PANEL_SET_ACTIVE_MESSAGE,
} from "@/core/shell/panel/panel-messages";
import { ApplicationRail, PanelStripeButton } from "@/core/shell/panel/rail";
import { IPanelSelection, usePanelSelection } from "@/core/shell/panel/use-panel-selection";
import { IPanelWidth, usePanelWidth } from "@/core/shell/panel/use-panel-width";
import { ApplicationTitleBar } from "@/core/shell/title-bar/ApplicationTitleBar";
import { mergeSx } from "@/core/theme/merge-sx";
import { getSurfaceSx } from "@/core/theme/surface";
import { RADIUS } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

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
  const application: Nullable<IApplicationDescriptor> = useCurrentApplication();
  const notify: TEmitNotification = useEmitNotification();
  const panels: ReadonlyArray<IEditorPanel> = useEditorPanelsRegistry();

  const { pathname } = useLocation();

  const isBusy: boolean = useIsEditorBusy();
  const settingsService: SettingsService = useInjection(SettingsService);

  // The element the routed content portals its toolbar into. Held here rather than in a provider of
  // its own: the frame hands it down and never reads it back.
  const [toolbarHost, setToolbarHost] = useState<Nullable<HTMLElement>>(null);

  const applicationPath: string = application?.path ?? "root";

  const leftPanels: Array<IEditorPanel> = selectPanelsOnSide(panels, "left");
  const applicationRightPanels: Array<IEditorPanel> = selectPanelsOnSide(panels, "right");
  // Registered rather than merely hidden, so a build without dev mode never mounts a panel that polls the backend.
  const rightPanels: Array<IEditorPanel> = [
    ...applicationRightPanels,
    ...(settingsService.isDevModeEnabled ? [JOBS_PANEL] : []),
    NOTIFICATIONS_PANEL,
  ];

  const leftSelection: IPanelSelection = usePanelSelection("left", leftPanels, application?.id ?? "root");
  const rightSelection: IPanelSelection = usePanelSelection("right", rightPanels, "global");

  const openCount: number = (leftSelection.activePanel ? 1 : 0) + (rightSelection.activePanel ? 1 : 0);
  const leftSizing: IPanelWidth = usePanelWidth("left", openCount);
  const rightSizing: IPanelWidth = usePanelWidth("right", openCount);

  const onError = useCallback((props: IErrorBoundaryFallbackProps) => <ApplicationCrash {...props} />, []);

  const onCaught = useCallback(
    (error: Error, componentStack: Nullable<string>) =>
      notify({
        details: componentStack ? `${error.message}\n${componentStack}` : error.message,
        severity: ENotificationSeverity.ERROR,
        source: application?.id ?? APPLICATION_SOURCE,
        title: "The interface crashed and was replaced",
      }),
    [application, notify]
  );

  // Flushed so the command is synchronous for its caller: whoever opens a panel to reach something inside it finds
  // that content mounted when the dispatch returns, with no frame to wait for and no retry to write.
  useOnCommand(PANEL_SET_ACTIVE_MESSAGE, ({ side, panelId }: IPanelSetActiveMessage) =>
    flushSync(() => (side === "left" ? leftSelection : rightSelection).onOpenPanel(panelId))
  );

  useOnQuery(
    PANEL_ACTIVE_QUERY,
    ({ side }: IPanelSideMessage) => (side === "left" ? leftSelection : rightSelection).activePanelId
  );

  return (
    <EditorToolbarHostContext.Provider value={toolbarHost}>
      <Box
        data-testid={dataTestId}
        id={id}
        className={className}
        sx={mergeSx(getSurfaceSx("frame"), {
          display: "flex",
          flexDirection: "column",
          width: "100%",
          height: "100%",
          flexWrap: "nowrap",
        })}
      >
        <ApplicationTitleBar toolbarRef={setToolbarHost} isBusy={isBusy} />

        <Box sx={{ display: "flex", flexGrow: 1, minHeight: 0, flexWrap: "nowrap" }}>
          <ApplicationRail
            panels={leftPanels}
            activePanelId={leftSelection.activePanelId}
            onTogglePanel={leftSelection.onTogglePanel}
          />

          <Box
            sx={{
              display: "flex",
              flexGrow: 1,
              minWidth: 0,
              minHeight: 0,
              overflow: "hidden",
              border: 1,
              borderColor: "divider",
              borderRadius: `${RADIUS.md}px`,
            }}
          >
            <ErrorBoundary resetKey={pathname} fallback={onError} onCaught={onCaught}>
              <Suspense key={applicationPath} fallback={<ApplicationLoader />}>
                <ApplicationScope application={application}>
                  <ApplicationPanelSlot
                    side={"left"}
                    panel={leftSelection.activePanel}
                    width={leftSizing.width}
                    onResize={leftSizing.onResize}
                  />

                  <Box sx={{ display: "flex", flexGrow: 1, minWidth: 0, minHeight: 0, overflow: "hidden" }}>
                    <ErrorBoundary resetKey={pathname} fallback={onError} onCaught={onCaught}>
                      {children}
                    </ErrorBoundary>
                  </Box>

                  <ApplicationPanelSlot
                    side={"right"}
                    panel={rightSelection.activePanel}
                    width={rightSizing.width}
                    onResize={rightSizing.onResize}
                  />
                </ApplicationScope>
              </Suspense>
            </ErrorBoundary>
          </Box>

          <ApplicationPanelStripe
            side={"right"}
            panels={applicationRightPanels}
            activePanelId={rightSelection.activePanelId}
            footer={
              <>
                {settingsService.isDevModeEnabled ? (
                  <PanelStripeButton
                    panel={JOBS_PANEL}
                    side={"right"}
                    isActive={rightSelection.activePanelId === JOBS_PANEL.id}
                    onTogglePanel={rightSelection.onTogglePanel}
                  />
                ) : null}

                <PanelStripeButton
                  panel={NOTIFICATIONS_PANEL}
                  side={"right"}
                  isActive={rightSelection.activePanelId === NOTIFICATIONS_PANEL.id}
                  onTogglePanel={rightSelection.onTogglePanel}
                />
              </>
            }
            onTogglePanel={rightSelection.onTogglePanel}
          />
        </Box>

        <ApplicationStatusBar />
      </Box>
    </EditorToolbarHostContext.Provider>
  );
}
