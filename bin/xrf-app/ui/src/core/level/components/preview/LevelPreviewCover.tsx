import { useInjection } from "@wirestate/react";
import { RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/DelayedProgress";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

import { describeLevelLoad } from "@/core/level/lib/load/level-load-progress";
import { LevelRenderService } from "@/core/level/services/level-render";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { RenderFailureNotice } from "@/core/render/components/overlay/RenderFailureNotice";

interface ILevelPreviewCoverProps extends BaseComponentProps {
  /** Whether the level itself is being opened, before any of it is read. */
  isLoading: boolean;
}

/**
 * What hides the viewport until a level has been drawn with everything it opens with, so it never assembles in view:
 * opaque, taking the pointer from the canvas under it, and faded out once the level is shown. A renderer that failed
 * never shows it, so the cover stays and says why.
 */
export function LevelPreviewCover({
  "data-testid": dataTestId = "level-preview-cover",
  id,
  className,
  isLoading,
}: ILevelPreviewCoverProps): ReactElement {
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const load: Nullable<RenderLoadReport> = viewportService.load;
  const failure: Nullable<string> = renderService.failure;
  const isCovering: boolean = failure !== null || isLoading || !renderService.isRevealed;

  let label: string = "Preparing the level…";

  if (isLoading) {
    label = "Opening level…";
  } else if (load && !load.isReady) {
    label = `${describeLevelLoad(load)}…`;
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
      {failure !== null ? <RenderFailureNotice failure={failure} /> : null}

      {failure === null && isCovering ? <DelayedProgress isOnViewport={true} label={label} /> : null}
    </div>
  );
}
