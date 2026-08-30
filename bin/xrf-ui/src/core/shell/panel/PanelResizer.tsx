import { Box } from "@mui/material";
import { PointerEvent, ReactElement, useCallback, useRef } from "react";

import { TEditorPanelSide } from "@/core/shell/panel/context";

export interface IPanelResizerProps {
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

  const onPointerUp = useCallback((event: PointerEvent<HTMLDivElement>) => {
    event.currentTarget.releasePointerCapture(event.pointerId);
  }, []);

  return (
    <Box
      aria-hidden={true}
      sx={{
        position: "absolute",
        top: 0,
        bottom: 0,
        // Centred on the border it straddles, so the target is symmetric around the line people aim at.
        ...(side === "left" ? { right: -3 } : { left: -3 }),
        width: 7,
        cursor: "col-resize",
        zIndex: 2,
        // Only shows while pointed at; at rest the panel's own border is all there is to see.
        "&:hover::after": {
          content: '""',
          position: "absolute",
          top: 0,
          bottom: 0,
          left: 3,
          width: "1px",
        },
      }}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={onPointerUp}
    />
  );
}
