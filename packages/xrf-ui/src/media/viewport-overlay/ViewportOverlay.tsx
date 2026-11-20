import { ReactElement, ReactNode } from "react";

import { BaseComponentProps } from "#/element-types";
import { cn } from "#/theme";

/** Which corner of the viewport a readout sits in. */
export type TViewportOverlayCorner = "top-left" | "top-right" | "bottom-left" | "bottom-right";

const CORNER_POSITION: Record<TViewportOverlayCorner, string> = {
  "bottom-left": "bottom-2 left-2",
  "bottom-right": "bottom-2 right-2",
  "top-left": "top-2 left-2",
  "top-right": "top-2 right-2",
};

interface IViewportOverlayProps extends BaseComponentProps {
  corner: TViewportOverlayCorner;
  children: ReactNode;
}

/**
 * A readout laid over a scene, which is there to be glanced at and nothing else.
 */
export function ViewportOverlay({
  "data-testid": dataTestId = "viewport-overlay",
  id,
  className,
  corner,
  children,
}: IViewportOverlayProps): ReactElement {
  return (
    <div
      data-testid={dataTestId}
      id={id}
      aria-hidden={true}
      className={cn(
        "pointer-events-none absolute z-10 rounded-surface bg-viewport-backdrop/80 px-2 py-1 select-none",
        "font-mono text-xs leading-tight text-viewport-text",
        CORNER_POSITION[corner],
        className
      )}
    >
      {children}
    </div>
  );
}
