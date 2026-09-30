import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { LevelPreviewStatus } from "@/core/level/components/preview/LevelPreviewStatus";
import { ILevelSpawnReport, isLevelSpawnReading } from "@/core/level/lib/spawn";
import { ILevelStreamProgress, LevelLoadService, LevelViewportService } from "@/core/level/services";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";

interface ILevelPreviewActivityProps {
  /** Whether a level is open, which is what the status bar says when nothing is being read. */
  isOpen: boolean;
  /** Whether the level itself is being opened, which is a different wait from streaming its sectors. */
  isLoading: boolean;
}

/**
 * What the loader is doing: the streaming progress over the viewport, and the activity in the status bar, sectors
 * before spawned models. Its own observer, so a sector arriving redraws this and not the toolbar beside it.
 */
export function LevelPreviewActivity({ isOpen, isLoading }: ILevelPreviewActivityProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const streaming: ILevelStreamProgress = loadService.streaming;
  const isStreaming: boolean = loadService.isStreaming;
  const spawn: ILevelSpawnReport = loadService.spawnReport;

  let activity: Nullable<string> = isOpen ? null : "No level open";

  if (isLoading) {
    activity = "Opening level";
  } else if (isStreaming) {
    activity = `Streaming sector ${Math.min(streaming.loaded + 1, streaming.total)} of ${streaming.total}`;
  } else if (isLevelSpawnReading(spawn)) {
    activity = `Reading spawned models, ${spawn.read} of ${spawn.visuals}`;
  }

  return (
    <>
      {!isLoading && isStreaming && viewportService.isRevealed ? (
        <div className={"pointer-events-none absolute inset-x-0 bottom-0 flex justify-center pb-6"}>
          <DelayedProgress
            data-testid={"level-stream-progress"}
            isOnViewport={true}
            label={`Streaming sectors, ${streaming.loaded} of ${streaming.total}`}
          />
        </div>
      ) : null}

      <LevelPreviewStatus activity={activity} />
    </>
  );
}
