import { Theme, useTheme } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement, useCallback, useLayoutEffect, useMemo, useRef } from "react";

import {
  IEquipmentGridPalette,
  paintEquipmentGrid,
  prepareCanvas,
  toEquipmentGridPalette,
} from "@/applications/sprite-equipment-editor/lib/scene";
import { IEquipmentLayout, TEquipmentCell } from "@/core/sprite-equipment/lib";
import { IImageViewportView } from "@/core/ui/media/ImageViewport";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { toPanZoomTransform } from "@/lib/media/pan-zoom";

interface IEquipmentGridCanvasProps extends BaseComponentProps {
  view: IImageViewportView;
  layout: IEquipmentLayout;
  isGridVisible: boolean;
  isOccupancyVisible: boolean;
  hoveredCell: Nullable<TEquipmentCell>;
  selectedCell: Nullable<TEquipmentCell>;
}

/**
 * The lattice, drawn over the sheet as one canvas.
 */
export function EquipmentGridCanvas({
  "data-testid": dataTestId = "equipment-grid-canvas",
  id,
  className,
  view,
  layout,
  isGridVisible,
  isOccupancyVisible,
  hoveredCell,
  selectedCell,
}: IEquipmentGridCanvasProps): ReactElement {
  const theme: Theme = useTheme();
  const canvasRef = useRef<Nullable<HTMLCanvasElement>>(null);

  const palette: IEquipmentGridPalette = useMemo(() => toEquipmentGridPalette(theme), [theme]);

  const draw = useCallback((): void => {
    const context: Nullable<CanvasRenderingContext2D> = prepareCanvas(canvasRef.current, view.viewport);

    if (!context) {
      return;
    }

    paintEquipmentGrid(context, {
      hoveredCell,
      isGridVisible,
      isOccupancyVisible,
      layout,
      palette,
      selectedCell,
      transform: toPanZoomTransform(view.controller.get(), view.content, view.viewport),
      viewport: view.viewport,
    });
  }, [hoveredCell, isGridVisible, isOccupancyVisible, layout, palette, selectedCell, view]);

  // Layout rather than passive, so the overlay is painted in the same frame that placed the picture under it and the
  // two never disagree for a frame after a resize or a reopen.
  useLayoutEffect(() => {
    draw();

    return view.controller.subscribe(draw);
  }, [draw, view.controller]);

  return (
    <canvas
      ref={canvasRef}
      data-testid={dataTestId}
      aria-hidden={true}
      id={id}
      className={cn("absolute top-0 left-0 h-full w-full", className)}
    />
  );
}
