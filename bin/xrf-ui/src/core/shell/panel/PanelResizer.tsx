import { PointerEvent, ReactElement, useCallback, useRef } from "react";

import { TEditorPanelSide } from "@/core/shell/editor-shell";
import { cn } from "@/lib/dom/dom-name";

interface IPanelResizerProps {
  side: TEditorPanelSide;
  width: number;
  onResize: (width: number) => void;
}

/**
 * The grab handle between a panel and the content.
 *
 * Positioned over the panel's own border rather than laid out beside it: as a flex child it cost four
 * real pixels and read as a gap. Pointer capture rather than window listeners, so the drag keeps
 * following the cursor once it leaves the strip.
 */
export function PanelResizer({ side, width, onResize }: IPanelResizerProps): ReactElement {
  const origin = useRef<{ x: number; width: number }>({ x: 0, width });

  const onPointerDown = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      origin.current = { x: event.clientX, width };
      event.currentTarget.setPointerCapture(event.pointerId);
    },
    [width]
  );

  const onPointerMove = useCallback(
    (event: PointerEvent<HTMLDivElement>) => {
      if (!event.currentTarget.hasPointerCapture(event.pointerId)) {
        return;
      }

      // The left panel grows as the cursor moves right; the right panel grows as it moves left.
      const delta: number = (event.clientX - origin.current.x) * (side === "left" ? 1 : -1);

      onResize(origin.current.width + delta);
    },
    [onResize, side]
  );

  return (
    <div
      aria-hidden={true}
      className={cn(
        "absolute top-0 bottom-0 z-2 w-1.75 cursor-col-resize",
        "after:absolute after:inset-y-0 after:left-0.75 after:w-px after:bg-transparent hover:after:bg-primary",
        side === "left" ? "-right-0.75" : "-left-0.75"
      )}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
    />
  );
}
