import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { ErrorState } from "@xrf/ui/layout/error-state";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

interface IRenderFailureCoverProps extends BaseComponentProps {
  /** Why the renderer stopped, or null while it draws. */
  failure: Nullable<string>;
}

/**
 * What covers a viewport whose renderer has stopped, saying why, rather than leaving its last frame standing.
 */
export function RenderFailureCover({
  "data-testid": dataTestId = "render-failure-cover",
  id,
  className,
  failure,
}: IRenderFailureCoverProps): Nullable<ReactElement> {
  if (failure === null) {
    return null;
  }

  return (
    <div data-testid={dataTestId} id={id} className={cn("absolute inset-0 z-20 bg-viewport-backdrop", className)}>
      <ErrorState title={"The renderer stopped"} description={failure} isOnViewport />
    </div>
  );
}
