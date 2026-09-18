import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback, useState } from "react";

import { LevelPreviewViewport } from "@/core/level/components/preview/LevelPreviewViewport";
import { ILevelPoint } from "@/core/level/lib/level-residency";
import { EMPTY_LEVEL_STATS, ILevelStats } from "@/core/level/lib/level-stats";
import { LevelLoadService } from "@/core/level/services";
import { BaseComponentProps } from "@/lib/dom/element-types";

import { LevelViewerOpenForm } from "./components/LevelViewerOpenForm";
import { LevelViewerStats } from "./components/LevelViewerStats";

/**
 * Fly a compiled level, streaming its sectors as the camera reaches them.
 */
export function LevelViewerApplication({
  "data-testid": dataTestId = "level-viewer-application",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);

  const [stats, setStats] = useState<ILevelStats>(EMPTY_LEVEL_STATS);

  const onCameraMoved = useCallback((point: ILevelPoint) => void loadService.stream(point), [loadService]);

  if (!loadService.level.value) {
    return <LevelViewerOpenForm />;
  }

  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={className}
      style={{ display: "flex", flexDirection: "column", height: "100%", position: "relative" }}
    >
      <LevelPreviewViewport
        sectors={loadService.sectors}
        bounds={loadService.level.value.selected.value.bounds}
        onCameraMoved={onCameraMoved}
        onStats={setStats}
      />

      <LevelViewerStats stats={stats} />
    </div>
  );
}
