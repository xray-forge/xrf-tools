import { ReactElement } from "react";

import { EmptyState } from "@/core/ui/layout/EmptyState";
import { ErrorState } from "@/core/ui/layout/ErrorState";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelPreviewEmptyProps extends BaseComponentProps {
  /** Why the last open failed, or absent when nothing is open and nothing went wrong. */
  error?: string;
  onRetry?: () => void;
}

/**
 * What covers the viewport while it draws no level: nothing has been opened, or the last open failed.
 */
export function LevelPreviewEmpty({
  "data-testid": dataTestId = "level-preview-empty",
  id,
  className,
  error,
  onRetry,
}: ILevelPreviewEmptyProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("absolute inset-0 flex surface-content", className)}>
      {error ? (
        <ErrorState title={"Could not open this level"} description={error} onRetry={onRetry} />
      ) : (
        <EmptyState title={"No level open"} description={"Pick a level from a game root to fly it."} />
      )}
    </div>
  );
}
