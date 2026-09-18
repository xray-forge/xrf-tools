import { default as InfoOutlinedIcon } from "@mui/icons-material/InfoOutlined";
import { default as SpeedIcon } from "@mui/icons-material/Speed";
import { ReactElement, ReactNode, useMemo, useState } from "react";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { LevelHeaderPanel } from "@/core/level/components/panels/LevelHeaderPanel";
import { LevelStreamPanel } from "@/core/level/components/panels/LevelStreamPanel";
import { LevelPreviewEmpty } from "@/core/level/components/preview/LevelPreviewEmpty";
import { LevelPreviewToolbar } from "@/core/level/components/preview/LevelPreviewToolbar";
import { ILevelPreviewViewportProps, LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import {
  DEFAULT_LEVEL_SECTOR_VIEW_OPTIONS,
  ILevelSectorViewOptions,
} from "@/core/level/components/scene/LevelPreviewSectors";
import { ILevelPoint } from "@/core/level/lib/level-residency";
import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { EMPTY_LEVEL_STATS, ILevelStats } from "@/core/level/lib/level-stats";
import { ILevelStreamProgress } from "@/core/level/services";
import { EditorFileHeader } from "@/core/shell/editor/EditorFileHeader";
import { EditorLayout } from "@/core/shell/editor/EditorLayout";
import { IEditorPanel, useEditorPanels, useEditorStatus } from "@/core/shell/editor-shell";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";
import { Nullable } from "@/lib/types/general";

interface ILevelPreviewLayoutProps extends BaseComponentProps {
  /** Resident sectors the viewport draws. */
  sectors: ReadonlyMap<number, ILoadedSector>;
  /** The level's extent, which frames the camera once when a level opens. */
  bounds: Nullable<VisualBounds>;
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
  const [options, setOptions] = useState<ILevelSectorViewOptions>(DEFAULT_LEVEL_SECTOR_VIEW_OPTIONS);
  const [stats, setStats] = useState<ILevelStats>(EMPTY_LEVEL_STATS);

  const isOpen: boolean = Boolean(name);
  const isStreaming: boolean = streaming.total > 0;

  const status: Array<string> = useMemo(() => {
    if (isLoading) {
      return ["Opening level"];
    }

    if (isStreaming) {
      return [`Streaming sector ${Math.min(streaming.loaded + 1, streaming.total)} of ${streaming.total}`];
    }

    return isOpen
      ? [`${stats.sectors} sectors`, `${stats.draws} draws`, `${stats.triangles} triangles`, formatBytes(stats.bytes)]
      : ["No level open"];
  }, [isLoading, isOpen, isStreaming, stats, streaming]);

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
        render: () => <LevelStreamPanel stats={stats} />,
      },
    ],
    [stats]
  );

  useEditorStatus(status);

  return (
    <EditorLayout
      toolbar={
        <LevelPreviewToolbar subtitle={subtitle} options={options} onChangeOptions={setOptions} onBack={onBack} />
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
            renderViewport({ bounds, onCameraMoved, onStats: setStats, options, sectors })
          ) : (
            <LevelPreviewViewport
              sectors={sectors}
              bounds={bounds}
              options={options}
              onCameraMoved={onCameraMoved}
              onStats={setStats}
            />
          )}

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
    </EditorLayout>
  );
}
