import { useInjection } from "@wirestate/react";
import { RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { ErrorState } from "@xrf/ui/layout/error-state";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { describeLevelLoad } from "@/core/level/lib/load/level-load-progress";
import { LevelViewportService } from "@/core/level/services/level-viewport";

/** What the cover asks of the render service drawing under it. */
export interface ILevelPreviewCoverHost {
  /** Why the viewport draws nothing it was asked to, or null while it does. */
  readonly failure: Nullable<string>;
  /** Whether what is open is drawn with everything it opens with. */
  readonly isRevealed: boolean;
}

interface ILevelPreviewCoverProps extends BaseComponentProps {
  /** The render service drawing under the cover. */
  host: ILevelPreviewCoverHost;
  /** What is drawn, as the cover names it while it waits: a level, a scene. */
  subject?: string;
  /** What the load does once the sectors and textures are in, where it is not a level's spawned objects and the rest. */
  rest?: string;
  /** Whether what is drawn is itself being opened, before any of it is read. */
  isLoading: boolean;
}

/**
 * Hides the viewport, taking its pointer, until a level or a scene is drawn with everything it opens with, so it never
 * assembles in view. After a renderer failure the cover stays and says why.
 */
export function LevelPreviewCover({
  "data-testid": dataTestId = "level-preview-cover",
  id,
  className,
  host,
  subject = "level",
  rest,
  isLoading,
}: ILevelPreviewCoverProps): ReactElement {
  const viewportService: LevelViewportService = useInjection(LevelViewportService);

  const load: Nullable<RenderLoadReport> = viewportService.load;
  const failure: Nullable<string> = host.failure;
  const isCovering: boolean = failure !== null || isLoading || !host.isRevealed;

  let label: string = `Preparing the ${subject}…`;

  if (isLoading) {
    label = `Opening ${subject}…`;
  } else if (load && !load.isReady) {
    label = `${describeLevelLoad(load, rest)}…`;
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
      {failure !== null ? <ErrorState title={"The renderer stopped"} description={failure} isOnViewport /> : null}

      {failure === null && isCovering ? <DelayedProgress isOnViewport={true} label={label} /> : null}
    </div>
  );
}
