import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { default as Inventory2Icon } from "@mui/icons-material/Inventory2";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";
import { default as WbCloudyIcon } from "@mui/icons-material/WbCloudy";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode } from "react";

import { LevelViewerToolbar } from "@/applications/level-viewer/components/LevelViewerToolbar";
import { ELevelViewerPanelId } from "@/applications/level-viewer/lib/level-viewer-panel-id";
import { LevelHeaderPanel } from "@/core/level/components/panels/LevelHeaderPanel";
import { LevelProblemsPanel } from "@/core/level/components/panels/LevelProblemsPanel";
import { LevelRendererPanel } from "@/core/level/components/panels/LevelRendererPanel";
import { LevelSpawnPanel } from "@/core/level/components/panels/LevelSpawnPanel";
import { LevelSurfacesPanel } from "@/core/level/components/panels/LevelSurfacesPanel";
import { LevelWeatherPanel } from "@/core/level/components/panels/LevelWeatherPanel";
import { LevelPreviewCoordinates } from "@/core/level/components/preview/LevelPreviewCoordinates";
import { LevelPreviewCover } from "@/core/level/components/preview/LevelPreviewCover";
import { LevelPreviewEmpty } from "@/core/level/components/preview/LevelPreviewEmpty";
import { LevelPreviewMetrics } from "@/core/level/components/preview/LevelPreviewMetrics";
import { LevelPreviewPick } from "@/core/level/components/preview/LevelPreviewPick";
import { ILevelPreviewViewportProps, LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import { LevelViewService } from "@/core/level/services";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorPanels } from "@/core/shell/editor-shell";
import { cn } from "@/core/theme/class-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelViewerLayoutProps extends BaseComponentProps {
  /** What the open level is called, which heads the viewport. Its presence is what offers stats and picks. */
  name?: Nullable<string>;
  /**
   * What the level being opened is called, which heads the viewport until it is open: the header comes in with the open
   * otherwise, and the viewport shrinks by it, which sizes every target the renderer draws into again.
   */
  pending?: Nullable<string>;
  subtitle?: ReactNode;
  /** Whether the level itself is being opened, which is a different wait from streaming its sectors. */
  isLoading?: boolean;
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
  subtitle,
  isLoading = false,
  error,
  renderViewport,
  onRetry,
  onBack,
  onDeselect = null,
}: ILevelViewerLayoutProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);

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

  return (
    <EditorLayout toolbar={<LevelViewerToolbar subtitle={subtitle} onBack={onBack} />}>
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

          {isOpen && viewService.options.isStatsVisible ? (
            <>
              <LevelPreviewMetrics />
              <LevelPreviewCoordinates />
            </>
          ) : null}

          {isOpen ? <LevelPreviewPick /> : null}

          {!isOpen && !isLoading ? <LevelPreviewEmpty error={error} onRetry={onRetry} /> : null}

          {isOpen || isLoading ? <LevelPreviewCover isLoading={isLoading} /> : null}
        </div>
      </div>
    </EditorLayout>
  );
}
