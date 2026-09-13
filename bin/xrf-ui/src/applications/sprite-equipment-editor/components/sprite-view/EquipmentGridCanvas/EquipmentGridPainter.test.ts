import { describe, expect, it, jest } from "@jest/globals";

import { IEquipmentGridPalette } from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentGridCanvas/equipment-grid-palette";
import {
  IEquipmentGridFrame,
  paintEquipmentGrid,
} from "@/applications/sprite-equipment-editor/components/sprite-view/EquipmentGridCanvas/EquipmentGridPainter";
import { toEquipmentLayout } from "@/core/sprite-equipment/lib";
import { mockEquipmentDescriptor } from "@/fixtures/mocks/sprite.mocks";

/** Named rather than themed, so an assertion says which mark it is looking at. */
const PALETTE: IEquipmentGridPalette = {
  hover: "hover",
  line: "line",
  occupied: "occupied",
  outside: "outside",
  outsideEdge: "outside-edge",
  selected: "selected",
  selectedEdge: "selected-edge",
};

/** A context that records what it was asked to do, since jsdom draws nothing. */
function mockContext(): CanvasRenderingContext2D {
  return {
    beginPath: jest.fn(),
    fillRect: jest.fn(),
    lineTo: jest.fn(),
    moveTo: jest.fn(),
    stroke: jest.fn(),
    strokeRect: jest.fn(),
  } as unknown as CanvasRenderingContext2D;
}

/**
 * One frame over a twenty by ten cell sheet, drawn at one to one with no camera offset.
 *
 * @param overrides - What this case varies.
 * @returns The frame to paint.
 */
function mockFrame(overrides: Partial<IEquipmentGridFrame> = {}): IEquipmentGridFrame {
  return {
    hoveredCell: null,
    isGridVisible: true,
    layout: toEquipmentLayout(1000, 500, 50, []),
    palette: PALETTE,
    selectedCell: null,
    transform: { offsetX: 0, offsetY: 0, scale: 1 },
    viewport: { width: 1000, height: 500 },
    ...overrides,
  };
}

describe("paintEquipmentGrid", () => {
  it("shades one rectangle per claim, in sheet position", () => {
    const context: CanvasRenderingContext2D = mockContext();

    paintEquipmentGrid(
      context,
      mockFrame({
        layout: toEquipmentLayout(1000, 500, 50, [
          mockEquipmentDescriptor("wpn_ak74", { x: 2, y: 1, w: 3, h: 2 }),
          mockEquipmentDescriptor("wpn_pm", { x: 6, y: 0 }),
        ]),
      })
    );

    expect(context.fillRect).toHaveBeenCalledWith(100, 50, 150, 100);
    expect(context.fillRect).toHaveBeenCalledWith(300, 0, 50, 50);
  });

  it("outlines a claim that leaves the picture, and shades the region it reaches into", () => {
    const context: CanvasRenderingContext2D = mockContext();

    paintEquipmentGrid(
      context,
      mockFrame({
        layout: toEquipmentLayout(1000, 500, 50, [mockEquipmentDescriptor("over_the_edge", { x: 21 })]),
      })
    );

    // The strip past the right edge of a twenty column sheet, and the rectangle sitting out in it.
    expect(context.fillRect).toHaveBeenCalledWith(1000, 0, 100, 500);
    expect(context.strokeRect).toHaveBeenCalledWith(1050.5, 0.5, 49, 49);
  });

  it("draws no lattice when the grid is hidden", () => {
    const context: CanvasRenderingContext2D = mockContext();

    paintEquipmentGrid(context, mockFrame({ isGridVisible: false }));

    expect(context.stroke).not.toHaveBeenCalled();
  });

  it("draws no lattice once a cell is narrower than the lines around it", () => {
    const context: CanvasRenderingContext2D = mockContext();

    // Fifty pixel cells at a twelfth of their size are four pixels across, which is the floor.
    paintEquipmentGrid(context, mockFrame({ transform: { offsetX: 0, offsetY: 0, scale: 0.05 } }));

    expect(context.stroke).not.toHaveBeenCalled();
  });

  it("draws only the lattice lines the viewport can see", () => {
    const context: CanvasRenderingContext2D = mockContext();

    paintEquipmentGrid(
      context,
      mockFrame({
        layout: toEquipmentLayout(100000, 100000, 50, []),
        viewport: { width: 200, height: 100 },
      })
    );

    // Five columns and three rows of boundary fit a 200x100 pane, not the two thousand the sheet holds.
    expect((context.moveTo as jest.Mock).mock.calls).toHaveLength(5 + 3);
  });

  it("marks the hovered and selected cells", () => {
    const context: CanvasRenderingContext2D = mockContext();

    paintEquipmentGrid(context, mockFrame({ hoveredCell: [1, 2], selectedCell: [3, 4] }));

    expect(context.fillRect).toHaveBeenCalledWith(100, 50, 50, 50);
    expect(context.fillRect).toHaveBeenCalledWith(200, 150, 50, 50);
    // The open cell is outlined as well as shaded, so it stays distinct from the one merely under the pointer.
    expect(context.strokeRect).toHaveBeenCalledWith(200.5, 150.5, 49, 49);
  });
});
