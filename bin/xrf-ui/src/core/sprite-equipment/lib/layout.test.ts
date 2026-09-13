import { describe, expect, it } from "@jest/globals";

import { IEquipmentLayout, IEquipmentSectionDescriptor, toEquipmentLayout } from "@/core/sprite-equipment/lib";
import { mockEquipmentDescriptor } from "@/fixtures/mocks/sprite.mocks";

/** A twenty by ten cell sheet, which every case here places rectangles on. */
function mockLayout(descriptors: Array<IEquipmentSectionDescriptor>): IEquipmentLayout {
  return toEquipmentLayout(1000, 500, 50, descriptors);
}

function getSectionsAt(layout: IEquipmentLayout, row: number, column: number): Array<string> {
  return layout.at([row, column]).map((it: IEquipmentSectionDescriptor) => it.section);
}

describe("toEquipmentLayout", () => {
  it("covers every cell a rectangle spans", () => {
    const layout: IEquipmentLayout = mockLayout([mockEquipmentDescriptor("wpn_ak74", { x: 2, y: 1, w: 3, h: 2 })]);

    expect(getSectionsAt(layout, 1, 2)).toEqual(["wpn_ak74"]);
    expect(getSectionsAt(layout, 2, 4)).toEqual(["wpn_ak74"]);
    expect(getSectionsAt(layout, 1, 5)).toEqual([]);
    expect(getSectionsAt(layout, 3, 2)).toEqual([]);
  });

  it("keeps every section sharing a slot, in declaration order", () => {
    // The `_nimble` and `_snag` variants inherit their base weapon's position, so several sections legitimately claim
    // one slot. A cell that answers only the last of them cannot explain what is drawn there.
    const slot = { x: 2, y: 1, w: 2, h: 2 };
    const layout: IEquipmentLayout = mockLayout([
      mockEquipmentDescriptor("wpn_ak74", slot),
      mockEquipmentDescriptor("wpn_ak74_nimble", slot),
      mockEquipmentDescriptor("wpn_ak74_snag", slot),
    ]);

    expect(getSectionsAt(layout, 1, 2)).toEqual(["wpn_ak74", "wpn_ak74_nimble", "wpn_ak74_snag"]);
  });

  it("answers the same empty array for every unclaimed cell", () => {
    const layout: IEquipmentLayout = mockLayout([mockEquipmentDescriptor("wpn_ak74")]);

    expect(layout.at([9, 9])).toBe(layout.at([8, 8]));
  });

  it("keeps rectangles that leave the sheet rather than discarding them", () => {
    const layout: IEquipmentLayout = mockLayout([
      mockEquipmentDescriptor("inside"),
      mockEquipmentDescriptor("over_the_right_edge", { x: 19, w: 3 }),
      mockEquipmentDescriptor("below_the_bottom_edge", { y: 9, h: 3 }),
    ]);

    expect(layout.outside.map((it: IEquipmentSectionDescriptor) => it.section)).toEqual([
      "over_the_right_edge",
      "below_the_bottom_edge",
    ]);
    // Still indexed, so clicking the part that does sit on the sheet explains what claims it.
    expect(getSectionsAt(layout, 0, 19)).toEqual(["over_the_right_edge"]);
    expect(getSectionsAt(layout, 0, 21)).toEqual(["over_the_right_edge"]);
  });

  it("builds a lattice that spans every rectangle it indexes", () => {
    // The guarantee the layout exists for. A lattice narrower than its own occupants folds one cell onto another, and
    // a cell then answers with a neighbour's sections instead of its own.
    const layout: IEquipmentLayout = mockLayout([
      mockEquipmentDescriptor("wide", { x: 25 }),
      mockEquipmentDescriptor("second_row", { x: 5, y: 1 }),
    ]);

    expect(layout.grid.columns).toBe(26);
    expect(getSectionsAt(layout, 0, 25)).toEqual(["wide"]);
    expect(getSectionsAt(layout, 1, 5)).toEqual(["second_row"]);
  });
});
