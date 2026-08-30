import { Box } from "@mui/material";
import { ReactElement, ReactNode, useCallback, useState } from "react";
import { useLocation } from "react-router-dom";

import { ErrorBoundary, IErrorBoundaryFallbackProps } from "@/core/error/components/ErrorBoundary";
import { ENotificationSeverity, TEmitNotification, useEmitNotification } from "@/core/notifications/lib";
import { APPLICATION_SOURCE, IApplicationDescriptor } from "@/core/routing/application";
import { useCurrentApplication } from "@/core/routing/current-application.context";
import { ApplicationScope } from "@/core/shell/ApplicationScope";
import { useIsEditorBusy } from "@/core/shell/EditorBusyContext";
import { ApplicationCrash } from "@/core/shell/error/ApplicationCrash";
import { ApplicationStatusBar } from "@/core/shell/footer/ApplicationStatusBar";
import { EditorToolbarHostContext } from "@/core/shell/header/editor-toolbar-host";
import { ApplicationPanelSlot } from "@/core/shell/panel/ApplicationPanelSlot";
import { ApplicationPanelStripe } from "@/core/shell/panel/ApplicationPanelStripe";
import { ApplicationRail } from "@/core/shell/panel/ApplicationRail";
import { IEditorPanel, selectPanelsOnSide, useEditorPanelsRegistry } from "@/core/shell/panel/context";
import { NOTIFICATIONS_PANEL } from "@/core/shell/panel/notifications/notification-panel";
import { PanelStripeButton } from "@/core/shell/panel/PanelStripeButton";
import { IPanelSelection, usePanelSelection } from "@/core/shell/panel/use-panel-selection";
import { IPanelWidth, usePanelWidth } from "@/core/shell/panel/use-panel-width";
import { ApplicationTitleBar } from "@/core/shell/title-bar/ApplicationTitleBar";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

export interface IApplicationShellFrameProps extends BaseComponentProps {
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

  // The element the routed content portals its toolbar into. Held here rather than in a provider of
  // its own: the frame hands it down and never reads it back.
  const [toolbarHost, setToolbarHost] = useState<Nullable<HTMLElement>>(null);

  const applicationPath: string = application?.path ?? "root";

  const leftPanels: Array<IEditorPanel> = selectPanelsOnSide(panels, "left");
  const applicationRightPanels: Array<IEditorPanel> = selectPanelsOnSide(panels, "right");
  const rightPanels: Array<IEditorPanel> = [...applicationRightPanels, NOTIFICATIONS_PANEL];

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

  return (
    <EditorToolbarHostContext.Provider value={toolbarHost}>
      <Box
        data-testid={dataTestId}
        id={id}
        className={className}
        sx={{ display: "flex", flexDirection: "column", width: "100%", height: "100%", flexWrap: "nowrap" }}
      >
        <ApplicationTitleBar toolbarRef={setToolbarHost} isBusy={isBusy} />

        <Box sx={{ display: "flex", flexGrow: 1, minHeight: 0, flexWrap: "nowrap" }}>
          <ApplicationRail
            panels={leftPanels}
            activePanelId={leftSelection.activePanelId}
            onTogglePanel={leftSelection.onTogglePanel}
          />

          <ApplicationScope key={applicationPath} application={application}>
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

          <ApplicationPanelStripe
            side={"right"}
            panels={applicationRightPanels}
            activePanelId={rightSelection.activePanelId}
            footer={
              <PanelStripeButton
                panel={NOTIFICATIONS_PANEL}
                side={"right"}
                isActive={rightSelection.activePanelId === NOTIFICATIONS_PANEL.id}
                onTogglePanel={rightSelection.onTogglePanel}
              />
            }
            onTogglePanel={rightSelection.onTogglePanel}
          />
        </Box>

        <ApplicationStatusBar />
      </Box>
    </EditorToolbarHostContext.Provider>
  );
}
