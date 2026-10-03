import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { RenderLoadReport } from "@/core/ipc/types/xrf-renderer";
import { LevelPreviewStatus } from "@/core/level/components/preview/LevelPreviewStatus";
import { describeLevelLoad } from "@/core/level/lib/load/level-load-progress";
import { ILevelSpawnReport, isLevelSpawnReading } from "@/core/level/lib/spawn";
import { LevelLoadService, LevelViewportService } from "@/core/level/services";

interface ILevelPreviewActivityProps {
  /** Whether a level is open, which is what the status bar says when nothing is being read. */
  isOpen: boolean;
  /** Whether the level itself is being opened, which is a different wait from reading what it holds. */
  isLoading: boolean;
}

/**
 * What the loader is doing, in the status bar: the renderer's read of the level, then the spawned objects' listing.
 * Its own observer, so a sector arriving redraws this and not the toolbar beside it.
 */
export function LevelPreviewActivity({ isOpen, isLoading }: ILevelPreviewActivityProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const load: Nullable<RenderLoadReport> = viewportService.load;
  const spawn: ILevelSpawnReport = loadService.spawnReport;

  let activity: Nullable<string> = isOpen ? null : "No level open";

  if (isLoading) {
    activity = "Opening level";
  } else if (load && !load.isReady) {
    activity = describeLevelLoad(load);
  } else if (isLevelSpawnReading(spawn)) {
    activity = "Listing spawned objects";
  }

  return <LevelPreviewStatus activity={activity} />;
}
