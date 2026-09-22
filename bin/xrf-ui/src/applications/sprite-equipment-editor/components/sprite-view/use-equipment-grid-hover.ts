import { Nullable } from "@xrf/types";
import { useCallback, useState } from "react";

import { IEquipmentLayout, isSameCell, TEquipmentCell, toCellAt } from "@/core/sprite-equipment/lib";
import { IPanZoomPoint } from "@/lib/media/pan-zoom";

/** The cell under the pointer, and the move that tracks it. */
export interface IEquipmentGridHover {
  hoveredCell: Nullable<TEquipmentCell>;
  /** Takes a point on the sheet, in sheet pixels, or null once the pointer leaves. */
  onPointerMove: (point: Nullable<IPanZoomPoint>) => void;
}

/**
 * Tracks which cell the pointer is over.
 *
 * @param layout - The lattice points are read through, or null while nothing is open.
 * @returns The hovered cell and the gesture that moves it.
 */
export function useEquipmentGridHover(layout: Nullable<IEquipmentLayout>): IEquipmentGridHover {
  const [hoveredCell, setHoveredCell] = useState<Nullable<TEquipmentCell>>(null);

  const onPointerMove = useCallback(
    (point: Nullable<IPanZoomPoint>): void => {
      const cell: Nullable<TEquipmentCell> = layout && point ? toCellAt(layout.grid, point.x, point.y) : null;

      // Compared rather than set on every move: the pointer crosses a cell many times before it leaves it, and each
      // of those would otherwise repaint the overlay.
      setHoveredCell((current: Nullable<TEquipmentCell>) => (isSameCell(current, cell) ? current : cell));
    },
    [layout]
  );

  return { hoveredCell, onPointerMove };
}
