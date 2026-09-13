import { useCallback, useState } from "react";

import { IEquipmentLayout, isSameCell, TEquipmentCell, toCellAt } from "@/core/sprite-equipment/lib";
import { IPanZoomPoint } from "@/lib/media/pan-zoom";
import { Nullable } from "@/lib/types/general";

/** Which cells the pointer is over and which one is being explained, and the gestures that move them. */
export interface IEquipmentGridSelection {
  /** Cell under the pointer, or null while it is elsewhere. */
  hoveredCell: Nullable<TEquipmentCell>;
  /** Cell whose occupants are being shown, or null when none is. */
  selectedCell: Nullable<TEquipmentCell>;
  /** Takes a point on the sheet, in sheet pixels, or null once the pointer leaves. */
  onPointerMove: (point: Nullable<IPanZoomPoint>) => void;
  /** Takes a click on the sheet, in sheet pixels. */
  onClick: (point: IPanZoomPoint) => void;
  /** Stops explaining whatever was selected. */
  onClearSelection: () => void;
}

/**
 * Tracks which cell the pointer is over and which one is open.
 *
 * @param layout - The lattice points are read through, or null while nothing is open.
 * @returns The selection, and the gestures that move it.
 */
export function useEquipmentGridSelection(layout: Nullable<IEquipmentLayout>): IEquipmentGridSelection {
  const [hoveredCell, setHoveredCell] = useState<Nullable<TEquipmentCell>>(null);
  const [selectedCell, setSelectedCell] = useState<Nullable<TEquipmentCell>>(null);

  const toCell = useCallback(
    (point: Nullable<IPanZoomPoint>): Nullable<TEquipmentCell> =>
      layout && point ? toCellAt(layout.grid, point.x, point.y) : null,
    [layout]
  );

  const onPointerMove = useCallback(
    (point: Nullable<IPanZoomPoint>): void => {
      const cell: Nullable<TEquipmentCell> = toCell(point);

      // Compared rather than set on every move: the pointer crosses a cell many times before it leaves it, and each
      // of those would otherwise repaint the overlay.
      setHoveredCell((current: Nullable<TEquipmentCell>) => (isSameCell(current, cell) ? current : cell));
    },
    [toCell]
  );

  const onClick = useCallback(
    (point: IPanZoomPoint): void => {
      const cell: Nullable<TEquipmentCell> = toCell(point);

      // Clicking the open cell again closes it, so there is a way out that is not the close button.
      setSelectedCell((current: Nullable<TEquipmentCell>) => (isSameCell(current, cell) ? null : cell));
    },
    [toCell]
  );

  const onClearSelection = useCallback((): void => setSelectedCell(null), []);

  return { hoveredCell, onClearSelection, onClick, onPointerMove, selectedCell };
}
