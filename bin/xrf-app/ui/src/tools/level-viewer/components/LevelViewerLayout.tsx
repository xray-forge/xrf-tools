import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { EditorFileHeader } from "@xrf/ui/editor/file-header";
import { EditorLayout } from "@xrf/ui/editor/layout";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { AccountTreeIcon } from "@xrf/ui/icons/account-tree";
import { InfoOutlinedIcon } from "@xrf/ui/icons/info-outlined";
import { LayersIcon } from "@xrf/ui/icons/layers";
import { SpeedIcon } from "@xrf/ui/icons/speed";
import { TuneIcon } from "@xrf/ui/icons/tune";
import { WarningAmberIcon as WarningIcon } from "@xrf/ui/icons/warning-amber";
import { WbCloudyIcon } from "@xrf/ui/icons/wb-cloudy";
import { cn } from "@xrf/ui/theme";
import { ReactElement, ReactNode } from "react";

import { LevelPreviewCover } from "@/core/level/components/preview/LevelPreviewCover";
import { LevelPreviewPick } from "@/core/level/components/preview/LevelPreviewPick";
import { ILevelPreviewViewportProps, LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import { ILevelOverlay, LEVEL_OVERLAYS } from "@/core/level/lib/view/level-overlays";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";
import { useShellMenus, useShellPanels } from "@/core/shell/contributions";
import { IMenuContribution, TMenuEntry } from "@/core/shell/menu/lib";
import { IShellPanel } from "@/core/shell/panels/shell-panel";
import { LevelViewerCaptureActions } from "@/tools/level-viewer/components/capture/LevelViewerCaptureActions";
import { LevelViewerCaptureCover } from "@/tools/level-viewer/components/capture/LevelViewerCaptureCover";
import { LevelViewerToolbar } from "@/tools/level-viewer/components/LevelViewerToolbar";
import { LevelHeaderPanel } from "@/tools/level-viewer/components/panels/LevelHeaderPanel";
import { LevelInspectorPanel } from "@/tools/level-viewer/components/panels/LevelInspectorPanel";
import { LevelOutlinerPanel } from "@/tools/level-viewer/components/panels/LevelOutlinerPanel";
import { LevelProblemsPanel } from "@/tools/level-viewer/components/panels/LevelProblemsPanel";
import { LevelRendererPanel } from "@/tools/level-viewer/components/panels/LevelRendererPanel";
import { LevelSurfacesPanel } from "@/tools/level-viewer/components/panels/LevelSurfacesPanel";
import { LevelWeatherPanel } from "@/tools/level-viewer/components/panels/LevelWeatherPanel";
import { LevelPreviewCoordinates } from "@/tools/level-viewer/components/preview/LevelPreviewCoordinates";
import { LevelPreviewEmpty } from "@/tools/level-viewer/components/preview/LevelPreviewEmpty";
import { LevelPreviewMetrics } from "@/tools/level-viewer/components/preview/LevelPreviewMetrics";
import { LevelPreviewSpot } from "@/tools/level-viewer/components/preview/LevelPreviewSpot";
import { LevelViewerScreenshotActions } from "@/tools/level-viewer/components/screenshot/LevelViewerScreenshotActions";
import { LevelViewerScreenshotToast } from "@/tools/level-viewer/components/screenshot/LevelViewerScreenshotToast";
import { LevelViewerSnapshotActions } from "@/tools/level-viewer/components/snapshot/LevelViewerSnapshotActions";
import { ELevelViewerPanelId } from "@/tools/level-viewer/lib/panels/level-viewer-panel-id";
import { LEVEL_VIEWER_VIEW_MENU } from "@/tools/level-viewer/menus";

interface ILevelViewerLayoutProps extends BaseComponentProps {
  /** What the open level is called, which heads the viewport. Its presence is what offers stats and picks. */
  name?: Nullable<string>;
  /**
   * What the level being opened is called, which heads the viewport until it is open: the header comes in with the open
   * otherwise, and the viewport shrinks by it, which sizes every target the renderer draws into again.
   */
  pending?: Nullable<string>;
  /** Whether the level itself is being opened, which is a different wait from reading its sectors. */
  isLoading?: boolean;
  /** Why the last open failed, shown with `onRetry` while no level is open. */
  error?: string;
  /** Draws the viewport, for a surface with no GPU to give one - a test, or a headless render. */
  renderViewport?: (props: ILevelPreviewViewportProps) => ReactNode;
  onRetry?: () => void;
  onBack?: () => void;
  onDeselect?: Nullable<() => void>;
}

/**
 * The level viewer's chrome: toolbar, viewport, panel stripe and the two waits a level has.
 */
export function LevelViewerLayout({
  "data-testid": dataTestId = "level-viewer-layout",
  id = "level-viewer-layout",
  className,
  name = null,
  pending = null,
  isLoading = false,
  error,
  renderViewport,
  onRetry,
  onBack,
  onDeselect = null,
}: ILevelViewerLayoutProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);

  const viewOptions: ILevelViewOptions = viewService.options;

  const isOpen: boolean = Boolean(name);
  const heading: Nullable<string> = name ?? pending;

  useShellPanels(
    (): Array<IShellPanel> => [
      {
        icon: <WbCloudyIcon />,
        id: ELevelViewerPanelId.WEATHER,
        isOpenByDefault: true,
        label: "Weather",
        render: () => <LevelWeatherPanel />,
        side: "left",
      },
      {
        icon: <AccountTreeIcon />,
        id: ELevelViewerPanelId.OUTLINER,
        label: "Outliner",
        render: () => <LevelOutlinerPanel />,
        side: "left",
      },
      {
        icon: <TuneIcon />,
        id: ELevelViewerPanelId.INSPECTOR,
        label: "Inspector",
        render: () => <LevelInspectorPanel />,
      },
      {
        icon: <InfoOutlinedIcon />,
        id: ELevelViewerPanelId.LEVEL,
        isOpenByDefault: true,
        label: "Level",
        render: () => <LevelHeaderPanel />,
      },
      {
        icon: <SpeedIcon />,
        id: ELevelViewerPanelId.RENDERER,
        label: "Renderer",
        render: () => <LevelRendererPanel />,
      },
      {
        icon: <LayersIcon />,
        id: ELevelViewerPanelId.SURFACES,
        label: "Surfaces",
        render: () => <LevelSurfacesPanel />,
        side: "left",
      },
      {
        icon: <WarningIcon />,
        id: ELevelViewerPanelId.PROBLEMS,
        label: "Problems",
        render: () => <LevelProblemsPanel />,
      },
    ],
    []
  );

  useShellMenus(
    (): Array<IMenuContribution> => [
      {
        section: LEVEL_VIEWER_VIEW_MENU.sections.overlays,
        entries: LEVEL_OVERLAYS.map((it: ILevelOverlay): TMenuEntry => ({
          kind: "toggle",
          id: `level-viewer/toggle-${it.id}`,
          label: it.label,
          isChecked: viewOptions[it.option],
          onToggle: () => viewService.toggleOption(it.option),
        })),
      },
    ],
    [viewOptions, viewService]
  );

  return (
    <EditorLayout toolbar={<LevelViewerToolbar onBack={onBack} />}>
      <LevelViewerSnapshotActions />
      <LevelViewerScreenshotActions />
      <LevelViewerCaptureActions />

      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {heading ? (
          <EditorFileHeader
            data-testid={"level-file-header"}
            name={heading}
            closeLabel={"Close level"}
            closeDescription={"Clear the selection and close this level"}
            onClose={onDeselect ?? undefined}
          />
        ) : null}

        <div
          data-testid={dataTestId}
          id={id}
          className={cn("relative flex min-h-0 min-w-0 flex-1 overflow-hidden", className)}
        >
          {renderViewport ? renderViewport({}) : <LevelPreviewViewport />}

          {isOpen && viewOptions.isStatsVisible ? (
            <>
              <LevelPreviewMetrics />
              <LevelPreviewCoordinates />
            </>
          ) : null}

          {isOpen && viewOptions.isMetered ? <LevelPreviewSpot /> : null}

          {isOpen ? <LevelPreviewPick /> : null}

          {isOpen ? <LevelViewerScreenshotToast /> : null}

          {isOpen ? <LevelViewerCaptureCover /> : null}

          {!isOpen && !isLoading ? <LevelPreviewEmpty error={error} onRetry={onRetry} /> : null}

          {isOpen || isLoading ? <LevelPreviewCover isLoading={isLoading} /> : null}
        </div>
      </div>
    </EditorLayout>
  );
}
