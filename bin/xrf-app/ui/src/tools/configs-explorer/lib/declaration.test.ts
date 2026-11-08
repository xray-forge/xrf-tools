import { describe, expect, it } from "@jest/globals";
import { LtxDeclaration } from "@xrf/ipc/types/xrf-ltx-inspect";

import { describeDeclaration } from "@/tools/configs-explorer/lib/declaration";

/** A field line of `section`, written with `operation`. */
function getFieldOf(section: string | null, operation: string, value: string | null): LtxDeclaration {
  return {
    isPatch: operation !== "",
    isPhysical: true,
    kind: "field",
    line: 2,
    name: "ammo_class",
    operation,
    path: "configs\\mod_system_a.ltx",
    section,
    sectionOperation: section === null ? "" : "!",
    source: "gamedata",
    value,
  };
}

describe("describeDeclaration", () => {
  it("spells a field line under its header with both prefixes, as the file writes it", () => {
    expect(describeDeclaration(getFieldOf("wpn_ak74", ">", "ammo_c"))).toBe("![wpn_ak74] >ammo_class = ammo_c");
  });

  it("spells a bare key without an equals sign, since it wrote no value", () => {
    expect(describeDeclaration(getFieldOf("wpn_ak74", "!", null))).toBe("![wpn_ak74] !ammo_class");
  });

  it("spells a key before the first header as the bare line it is", () => {
    expect(describeDeclaration(getFieldOf(null, "", "ammo_c"))).toBe("ammo_class = ammo_c");
  });

  it("spells a header with the parents it names", () => {
    expect(
      describeDeclaration({
        isPatch: false,
        isPhysical: false,
        kind: "section",
        line: 1,
        parents: ["wpn_base", "wpn_sounds"],
        path: "configs\\items\\w_ak74.ltx",
        section: "wpn_ak74",
        sectionOperation: "",
        source: "database\\gamedata.db",
      })
    ).toBe("[wpn_ak74]:wpn_base, wpn_sounds");
  });
});
