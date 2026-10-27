import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import {
  InfoOutlinedIcon,
  Inventory2Icon,
  LayersIcon,
  SpeedIcon,
  WarningAmberIcon as WarningIcon,
  WbCloudyIcon,
} from "@xrf/ui/icons";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement, ReactNode } from "react";

import { LevelViewerCaptureActions } from "@/applications/level-viewer/components/capture/LevelViewerCaptureActions";
import { LevelViewerCaptureCover } from "@/applications/level-viewer/components/capture/LevelViewerCaptureCover";
import { LevelViewerToolbar } from "@/applications/level-viewer/components/LevelViewerToolbar";
import { LevelHeaderPanel } from "@/applications/level-viewer/components/panels/LevelHeaderPanel";
import { LevelProblemsPanel } from "@/applications/level-viewer/components/panels/LevelProblemsPanel";
import { LevelRendererPanel } from "@/applications/level-viewer/components/panels/LevelRendererPanel";
import { LevelSpawnPanel } from "@/applications/level-viewer/components/panels/LevelSpawnPanel";
import { LevelSurfacesPanel } from "@/applications/level-viewer/components/panels/LevelSurfacesPanel";
import { LevelWeatherPanel } from "@/applications/level-viewer/components/panels/LevelWeatherPanel";
import { LevelPreviewCoordinates } from "@/applications/level-viewer/components/preview/LevelPreviewCoordinates";
import { LevelPreviewEmpty } from "@/applications/level-viewer/components/preview/LevelPreviewEmpty";
import { LevelPreviewMetrics } from "@/applications/level-viewer/components/preview/LevelPreviewMetrics";
import { LevelPreviewSpot } from "@/applications/level-viewer/components/preview/LevelPreviewSpot";
import { LevelViewerScreenshotActions } from "@/applications/level-viewer/components/screenshot/LevelViewerScreenshotActions";
import { LevelViewerScreenshotToast } from "@/applications/level-viewer/components/screenshot/LevelViewerScreenshotToast";
import { LevelViewerSnapshotActions } from "@/applications/level-viewer/components/snapshot/LevelViewerSnapshotActions";
import { ELevelViewerPanelId } from "@/applications/level-viewer/lib/panels/level-viewer-panel-id";
import { LEVEL_VIEWER_VIEW_MENU } from "@/applications/level-viewer/menus";
import { LevelPreviewCover } from "@/core/level/components/preview/LevelPreviewCover";
import { LevelPreviewPick } from "@/core/level/components/preview/LevelPreviewPick";
import { ILevelPreviewViewportProps, LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import { ILevelOverlay, LEVEL_OVERLAYS } from "@/core/level/lib/view/level-overlays";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { LevelViewService } from "@/core/level/services/level-view";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorMenus, useEditorPanels } from "@/core/shell/editor-shell";
import { IMenuContribution, TMenuEntry } from "@/core/shell/menu/lib";

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

  useEditorPanels(
    (): Array<IEditorPanel> => [
      {
        icon: <WbCloudyIcon />,
        id: ELevelViewerPanelId.WEATHER,
        isOpenByDefault: true,
        label: "Weather",
        render: () => <LevelWeatherPanel />,
        side: "left",
      },
      {
        icon: <Inventory2Icon />,
        id: ELevelViewerPanelId.SPAWN,
        label: "Spawn",
        render: () => <LevelSpawnPanel />,
        side: "left",
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

  useEditorMenus(
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
