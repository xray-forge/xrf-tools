import { BaseComponentProps } from "@xrf/ui/element-types";
import { DelayedProgress } from "@xrf/ui/layout/delayed-progress";
import { EmptyState } from "@xrf/ui/layout/empty-state";
import { ErrorState } from "@xrf/ui/layout/error-state";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

interface ILevelEditorViewportStateProps extends BaseComponentProps {
  /** Whether a scene is being opened. */
  isLoading?: boolean;
  /** Why the last open failed, shown with `onRetry` while no scene is open. */
  error?: string;
  onRetry?: () => void;
}

/**
 * What stands where the scene will be drawn: the open under way, why it failed, or that the scene is not drawn.
 */
export function LevelEditorViewportState({
  "data-testid": dataTestId = "level-editor-viewport-state",
  id,
  className,
  isLoading = false,
  error,
  onRetry,
}: ILevelEditorViewportStateProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("flex min-h-0 min-w-0 flex-1 surface-content", className)}>
      {isLoading ? (
        <DelayedProgress label={"Opening scene"} />
      ) : error ? (
        <ErrorState title={"Could not open this scene"} description={error} onRetry={onRetry} />
      ) : (
        <EmptyState
          title={"The scene is not drawn yet"}
          description={"Its objects are listed in the Outliner and shown in the Inspector."}
        />
      )}
    </div>
  );
}
