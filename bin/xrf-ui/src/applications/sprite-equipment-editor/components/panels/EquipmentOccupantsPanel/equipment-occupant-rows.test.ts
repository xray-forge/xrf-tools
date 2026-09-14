import { describe, expect, it } from "@jest/globals";

import {
  IEquipmentOccupantRow,
  toEquipmentOccupantRows,
} from "@/applications/sprite-equipment-editor/components/panels/EquipmentOccupantsPanel/equipment-occupant-rows";
import { toEquipmentLayout } from "@/core/sprite-equipment/lib";
import { mockEquipmentOccupant } from "@/fixtures/mocks/sprite.mocks";

const LAYOUT = toEquipmentLayout(1000, 500, 50, [
  mockEquipmentOccupant("wpn_ak74", { x: 2, y: 1, origin: "items\\weapons\\w_ak74.ltx" }),
  mockEquipmentOccupant("wpn_pm", { x: 4, y: 0, origin: "items\\weapons\\w_pm.ltx" }),
  mockEquipmentOccupant("af_medusa", { x: 6, y: 2, origin: "items\\artefacts.ltx" }),
]);

function getSections(rows: Array<IEquipmentOccupantRow>): Array<string> {
  return rows.map((row: IEquipmentOccupantRow) => row.occupant.section);
}

describe("toEquipmentOccupantRows", () => {
  it("lists everything in declaration order when nothing is typed", () => {
    expect(getSections(toEquipmentOccupantRows(LAYOUT, ""))).toEqual(["wpn_ak74", "wpn_pm", "af_medusa"]);
  });

  it("reveals a row at the top left of its rectangle", () => {
    // Row first, the way the lattice is indexed, while the occupant carries x before y.
    expect(toEquipmentOccupantRows(LAYOUT, "wpn_ak74")[0].cell).toEqual([1, 2]);
  });

  it("matches a section by name, ignoring case", () => {
    expect(getSections(toEquipmentOccupantRows(LAYOUT, "AK74"))).toEqual(["wpn_ak74"]);
  });

  it("matches everything one config declared", () => {
    // Finding a file's whole contribution is the other question this list answers, and a section name cannot ask it.
    expect(getSections(toEquipmentOccupantRows(LAYOUT, "artefacts.ltx"))).toEqual(["af_medusa"]);
  });

  it("answers nothing while no sheet is open", () => {
    expect(toEquipmentOccupantRows(null, "")).toEqual([]);
  });
});
