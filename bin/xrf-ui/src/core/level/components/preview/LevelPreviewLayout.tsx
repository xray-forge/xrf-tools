import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { default as LayersIcon } from "@mui/icons-material/Layers";
import { default as LightModeIcon } from "@mui/icons-material/LightModeOutlined";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";
import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, ReactNode, useMemo } from "react";

import { LevelHeaderPanel } from "@/core/level/components/panels/LevelHeaderPanel";
import { LevelLightingPanel } from "@/core/level/components/panels/LevelLightingPanel";
import { LevelProblemsPanel } from "@/core/level/components/panels/LevelProblemsPanel";
import { LevelStreamPanel } from "@/core/level/components/panels/LevelStreamPanel";
import { LevelSurfacesPanel } from "@/core/level/components/panels/LevelSurfacesPanel";
import { LevelCameraAction } from "@/core/level/components/preview/LevelCameraAction";
import { LevelPreviewCoordinates } from "@/core/level/components/preview/LevelPreviewCoordinates";
import { LevelPreviewEmpty } from "@/core/level/components/preview/LevelPreviewEmpty";
import { LevelPreviewMetrics } from "@/core/level/components/preview/LevelPreviewMetrics";
import { LevelPreviewStatus } from "@/core/level/components/preview/LevelPreviewStatus";
import { LevelPreviewToolbar } from "@/core/level/components/preview/LevelPreviewToolbar";
import { ILevelPreviewViewportProps, LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import { ILevelStreamProgress, LevelViewService } from "@/core/level/services";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorPanels } from "@/core/shell/editor-shell";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewLayoutProps extends BaseComponentProps {
  /** What the open level is called. Its presence is what draws the file header over the viewport. */
  name?: Nullable<string>;
  subtitle?: ReactNode;
  /** How far through the sectors the camera asked for the loader is. */
  streaming: ILevelStreamProgress;
  /** Whether the level itself is being opened, which is a different wait from streaming its sectors. */
  isLoading?: boolean;
  error?: string;
  /** Draws the viewport, for a surface with no webgl context to give one - a test, or a headless render. */
  renderViewport?: (props: ILevelPreviewViewportProps) => ReactNode;
  onRetry?: () => void;
  onBack?: () => void;
  onDeselect?: Nullable<() => void>;
}

/**
 * The level preview chrome: toolbar, viewport, panel stripe and the two waits a level has.
 */
export function LevelPreviewLayout({
  "data-testid": dataTestId = "level-preview-layout",
  id = "level-preview-layout",
  className,
  name = null,
  subtitle,
  streaming,
  isLoading = false,
  error,
  renderViewport,
  onRetry,
  onBack,
  onDeselect = null,
}: ILevelPreviewLayoutProps): ReactElement {
  const viewService: LevelViewService = useInjection(LevelViewService);

  const isOpen: boolean = Boolean(name);
  const isStreaming: boolean = streaming.total > 0;

  const activity: Nullable<string> = useMemo(() => {
    if (isLoading) {
      return "Opening level";
    }

    if (isStreaming) {
      return `Streaming sector ${Math.min(streaming.loaded + 1, streaming.total)} of ${streaming.total}`;
    }

    return isOpen ? null : "No level open";
  }, [isLoading, isOpen, isStreaming, streaming]);

  useEditorPanels(
    (): Array<IEditorPanel> => [
      {
        icon: <InfoOutlinedIcon />,
        id: "level",
        isOpenByDefault: true,
        label: "Level",
        render: () => <LevelHeaderPanel />,
      },
      {
        icon: <SpeedIcon />,
        id: "streaming",
        label: "Streaming",
        render: () => <LevelStreamPanel />,
      },
      {
        icon: <LightModeIcon />,
        id: "lighting",
        label: "Lighting",
        render: () => <LevelLightingPanel lighting={viewService.lighting} onChange={viewService.setLighting} />,
      },
      {
        icon: <LayersIcon />,
        id: "surfaces",
        label: "Surfaces",
        render: () => <LevelSurfacesPanel />,
      },
      {
        icon: <WarningIcon />,
        id: "problems",
        label: "Problems",
        render: () => <LevelProblemsPanel />,
      },
    ],
    [viewService.lighting, viewService.setLighting]
  );

  return (
    <EditorLayout
      toolbar={
        <LevelPreviewToolbar
          subtitle={subtitle}
          options={viewService.options}
          actions={<LevelCameraAction camera={viewService.camera} onChange={viewService.setCamera} />}
          onChangeOptions={viewService.setOptions}
          onBack={onBack}
        />
      }
    >
      <div className={"flex min-h-0 min-w-0 grow flex-col"}>
        {name ? (
          <EditorFileHeader
            data-testid={"level-file-header"}
            name={name}
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

          {!isOpen && !isLoading ? <LevelPreviewEmpty error={error} onRetry={onRetry} /> : null}

          {isLoading ? (
            <div className={"pointer-events-none absolute inset-0 flex items-center justify-center"}>
              <DelayedProgress isOnViewport={true} label={"Opening level…"} />
            </div>
          ) : null}

          {!isLoading && isStreaming ? (
            <div className={"pointer-events-none absolute inset-x-0 bottom-0 flex justify-center pb-6"}>
              <DelayedProgress
                data-testid={"level-stream-progress"}
                isOnViewport={true}
                label={`Streaming sectors, ${streaming.loaded} of ${streaming.total}`}
              />
            </div>
          ) : null}
        </div>
      </div>

      <LevelPreviewStatus activity={activity} />
    </EditorLayout>
  );
}
