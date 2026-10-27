import { BaseComponentProps } from "@xrf/ui/element-types";
import { EmptyState } from "@xrf/ui/layout/EmptyState";
import { ErrorState } from "@xrf/ui/layout/ErrorState";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

interface IVisualPreviewEmptyProps extends BaseComponentProps {
  /** Why the last open failed, or absent when nothing is open and nothing went wrong. */
  error?: string;
  /** Reads the failed open's source again. Absent where the surface has no attempt to repeat. */
  onRetry?: () => void;
}

/**
 * What covers the viewport while it holds no model: nothing has been opened, or the last open failed.
 */
export function VisualPreviewEmpty({
  "data-testid": dataTestId,
  id,
  className,
  error,
  onRetry,
}: IVisualPreviewEmptyProps): ReactElement {
  return (
    <div data-testid={dataTestId} id={id} className={cn("absolute inset-0 flex surface-content", className)}>
      {error ? (
        <ErrorState title={"Could not open this visual"} description={error} onRetry={onRetry} />
      ) : (
        <EmptyState title={"No visual open"} description={"Pick a model from the tree to preview it."} />
      )}
    </div>
  );
}
