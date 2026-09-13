import { describe, expect, it } from "@jest/globals";
import { act, renderHook, RenderHookResult } from "@testing-library/react";

import {
  IEquipmentGridSelection,
  useEquipmentGridSelection,
} from "@/applications/sprite-equipment-editor/components/sprite-view/use-equipment-grid-selection";
import { IEquipmentLayout, toEquipmentLayout } from "@/core/sprite-equipment/lib";
import { Nullable } from "@/lib/types/general";

/** A twenty by ten cell sheet of fifty pixel cells. */
const LAYOUT: IEquipmentLayout = toEquipmentLayout(1000, 500, 50, []);

function renderSelection(
  layout: Nullable<IEquipmentLayout> = LAYOUT
): RenderHookResult<IEquipmentGridSelection, unknown> {
  return renderHook(() => useEquipmentGridSelection(layout));
}

describe("useEquipmentGridSelection", () => {
  it("starts with nothing hovered or open", () => {
    const { result } = renderSelection();

    expect(result.current.hoveredCell).toBeNull();
    expect(result.current.selectedCell).toBeNull();
  });

  it("follows the pointer into cells", () => {
    const { result } = renderSelection();

    act(() => result.current.onPointerMove({ x: 275, y: 125 }));

    expect(result.current.hoveredCell).toEqual([2, 5]);

    act(() => result.current.onPointerMove(null));

    expect(result.current.hoveredCell).toBeNull();
  });

  it("keeps the same cell identity while the pointer crosses it", () => {
    const { result } = renderSelection();

    act(() => result.current.onPointerMove({ x: 10, y: 10 }));

    const first: Nullable<[number, number]> = result.current.hoveredCell;

    act(() => result.current.onPointerMove({ x: 40, y: 40 }));

    // Both points are in cell 0:0. A fresh tuple each move repaints the overlay on every mouse event.
    expect(result.current.hoveredCell).toBe(first);
  });

  it("opens a cell on click and closes it when the same cell is clicked again", () => {
    const { result } = renderSelection();

    act(() => result.current.onClick({ x: 275, y: 125 }));

    expect(result.current.selectedCell).toEqual([2, 5]);

    act(() => result.current.onClick({ x: 260, y: 110 }));

    // The second click lands in the same cell, so it closes rather than reselecting it.
    expect(result.current.selectedCell).toBeNull();
  });

  it("moves the open cell when a different one is clicked", () => {
    const { result } = renderSelection();

    act(() => result.current.onClick({ x: 275, y: 125 }));
    act(() => result.current.onClick({ x: 25, y: 25 }));

    expect(result.current.selectedCell).toEqual([0, 0]);
  });

  it("clears the open cell on request", () => {
    const { result } = renderSelection();

    act(() => result.current.onClick({ x: 275, y: 125 }));
    act(() => result.current.onClearSelection());

    expect(result.current.selectedCell).toBeNull();
  });

  it("answers nothing while no sheet is open", () => {
    const { result } = renderSelection(null);

    act(() => result.current.onPointerMove({ x: 275, y: 125 }));
    act(() => result.current.onClick({ x: 275, y: 125 }));

    expect(result.current.hoveredCell).toBeNull();
    expect(result.current.selectedCell).toBeNull();
  });
});
