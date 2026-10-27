import { Nullable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

import { RenderFailureNotice } from "@/core/render/components/overlay/RenderFailureNotice";

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
      <RenderFailureNotice failure={failure} />
    </div>
  );
}
