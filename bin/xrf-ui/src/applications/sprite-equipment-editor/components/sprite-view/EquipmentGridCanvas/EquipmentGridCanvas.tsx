import { Box, Theme, useTheme } from "@mui/material";
import { ReactElement, useCallback, useLayoutEffect, useMemo, useRef } from "react";

import { IEquipmentLayout, TEquipmentCell } from "@/core/sprite-equipment/lib";
import { IImageViewportView } from "@/core/ui/media/ImageViewport";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { toPanZoomTransform } from "@/lib/media/pan-zoom";
import { Nullable } from "@/lib/types/general";

import { IEquipmentGridPalette, toEquipmentGridPalette } from "./equipment-grid-palette";
import { prepareCanvas } from "./EquipmentGridCanvas.utils";
import { paintEquipmentGrid } from "./EquipmentGridPainter";

interface IEquipmentGridCanvasProps extends BaseComponentProps {
  view: IImageViewportView;
  layout: IEquipmentLayout;
  isGridVisible: boolean;
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
      layout,
      palette,
      selectedCell,
      transform: toPanZoomTransform(view.controller.get(), view.content, view.viewport),
      viewport: view.viewport,
    });
  }, [hoveredCell, isGridVisible, layout, palette, selectedCell, view]);

  // Layout rather than passive, so the overlay is painted in the same frame that placed the picture under it and the
  // two never disagree for a frame after a resize or a reopen.
  useLayoutEffect(() => {
    draw();

    return view.controller.subscribe(draw);
  }, [draw, view.controller]);

  return (
    <Box
      ref={canvasRef}
      data-testid={dataTestId}
      id={id}
      className={className}
      component={"canvas"}
      aria-hidden={true}
      sx={{ position: "absolute", left: 0, top: 0, width: "100%", height: "100%" }}
    />
  );
}
