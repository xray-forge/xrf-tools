import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ILevelStreamProgress, LevelLoadService, LevelViewportService } from "@/core/level/services";
import { DelayedProgress } from "@/core/ui/layout/DelayedProgress";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewCoverProps extends BaseComponentProps {
  /** Whether the level itself is being opened, before any of it is read. */
  isLoading: boolean;
}

/**
 * What hides the viewport until a level has been drawn with everything it opens with, so it never assembles in view:
 * opaque, taking the pointer from the canvas under it, and faded out once the level is shown.
 */
export function LevelPreviewCover({
  "data-testid": dataTestId = "level-preview-cover",
  id,
  className,
  isLoading,
}: ILevelPreviewCoverProps): ReactElement {
  const loadService: LevelLoadService = useInjection(LevelLoadService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const streaming: ILevelStreamProgress = loadService.streaming;
  const isCovering: boolean = isLoading || !viewportService.isRevealed;

  let label: string = "Preparing the level…";

  if (isLoading) {
    label = "Opening level…";
  } else if (streaming.total > 0) {
    label = `Reading sectors, ${streaming.loaded} of ${streaming.total}`;
  }

  return (
    <div
      data-testid={dataTestId}
      id={id}
      aria-hidden={!isCovering}
      className={cn(
        "absolute inset-0 z-20 flex items-center justify-center bg-viewport-backdrop transition-opacity duration-300",
        isCovering ? "opacity-100" : "pointer-events-none opacity-0",
        className
      )}
    >
      {isCovering ? <DelayedProgress isOnViewport={true} label={label} /> : null}
    </div>
  );
}
