import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { default as LightModeIcon } from "@mui/icons-material/LightModeOutlined";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { default as WarningIcon } from "@mui/icons-material/WarningAmber";
import { useInjection } from "@wirestate/react";
import { ReactElement, ReactNode, useMemo, useState } from "react";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { LevelHeaderPanel } from "@/core/level/components/panels/LevelHeaderPanel";
import { LevelLightingPanel } from "@/core/level/components/panels/LevelLightingPanel";
import { LevelProblemsPanel } from "@/core/level/components/panels/LevelProblemsPanel";
import { LevelStreamPanel } from "@/core/level/components/panels/LevelStreamPanel";
import { LevelCameraAction } from "@/core/level/components/preview/LevelCameraAction";
import { LevelPreviewCoordinates } from "@/core/level/components/preview/LevelPreviewCoordinates";
import { LevelPreviewEmpty } from "@/core/level/components/preview/LevelPreviewEmpty";
import { LevelPreviewMetrics } from "@/core/level/components/preview/LevelPreviewMetrics";
import { LevelPreviewStatus } from "@/core/level/components/preview/LevelPreviewStatus";
import { LevelPreviewToolbar } from "@/core/level/components/preview/LevelPreviewToolbar";
import { ILevelPreviewViewportProps, LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import { DEFAULT_LEVEL_CAMERA_OPTIONS, ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { DEFAULT_LEVEL_VIEW_OPTIONS, ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelStreamProgress, LevelViewportService } from "@/core/level/services";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorPanels } from "@/core/shell/editor-shell";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { Nullable } from "@/lib/types/general";

interface ILevelPreviewLayoutProps extends BaseComponentProps {
  /** Resident sectors the viewport draws. */
  sectors: ReadonlyMap<number, ILoadedSector>;
  /** The level's extent, which frames the camera once when a level opens. */
  bounds: Nullable<VisualBounds>;
  /** Where surfaces take their textures from, owned by the loader. */
  textures?: Nullable<ILevelTextureLookup>;
  /** What the open level is called. Its presence is what draws the file header over the viewport. */
  name?: Nullable<string>;
  subtitle?: ReactNode;
  /** How far through the sectors the camera asked for the loader is. */
  streaming: ILevelStreamProgress;
  /** Whether the level itself is being opened, which is a different wait from streaming its sectors. */
  isLoading?: boolean;
  error?: string;
  onCameraMoved: (point: ILevelPoint) => void;
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
  sectors,
  bounds,
  textures = null,
  name = null,
  subtitle,
  streaming,
  isLoading = false,
  error,
  onCameraMoved,
  renderViewport,
  onRetry,
  onBack,
  onDeselect = null,
}: ILevelPreviewLayoutProps): ReactElement {
  const [options, setOptions] = useState<ILevelViewOptions>(DEFAULT_LEVEL_VIEW_OPTIONS);
  const [lighting, setLighting] = useState<ILevelLighting>(DEFAULT_LEVEL_LIGHTING);
  const [camera, setCamera] = useState<ILevelCameraOptions>(DEFAULT_LEVEL_CAMERA_OPTIONS);
  const viewport: LevelViewportService = useInjection(LevelViewportService);

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
        render: () => <LevelLightingPanel lighting={lighting} onChange={setLighting} />,
      },
      {
        icon: <WarningIcon />,
        id: "problems",
        label: "Problems",
        render: () => <LevelProblemsPanel />,
      },
    ],
    [lighting]
  );

  return (
    <EditorLayout
      toolbar={
        <LevelPreviewToolbar
          subtitle={subtitle}
          options={options}
          onChangeOptions={setOptions}
          onBack={onBack}
          actions={<LevelCameraAction camera={camera} onChange={setCamera} />}
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
          {renderViewport ? (
            renderViewport({
              bounds,
              camera,
              lighting,
              onCameraMoved,
              onReport: viewport.report,
              options,
              sectors,
              textures,
            })
          ) : (
            <LevelPreviewViewport
              sectors={sectors}
              bounds={bounds}
              textures={textures}
              options={options}
              lighting={lighting}
              camera={camera}
              onCameraMoved={onCameraMoved}
              onReport={viewport.report}
            />
          )}

          {isOpen ? (
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
