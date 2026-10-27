import { BaseComponentProps } from "@xrf/ui/element-types";
import { cn } from "@xrf/ui/theme/class-name";
import { ReactElement } from "react";

/**
 * Where the spot meter reads, marked at the viewport's centre: a small square, drawn in the difference blend so it reads
 * over any light.
 */
export function LevelPreviewSpot({
  "data-testid": dataTestId = "level-preview-spot",
  id,
  className,
}: BaseComponentProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      className={cn(
        "pointer-events-none absolute top-1/2 left-1/2 size-3 -translate-x-1/2 -translate-y-1/2 border",
        "border-viewport-text mix-blend-difference",
        className
      )}
    />
  );
}
