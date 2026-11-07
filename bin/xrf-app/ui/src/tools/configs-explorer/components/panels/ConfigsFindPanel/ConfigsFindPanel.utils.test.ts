import { describe, expect, it } from "@jest/globals";
import { LtxDeclaration } from "@xrf/ipc/types/xrf-ltx-inspect";

import { toStatementText } from "./ConfigsFindPanel.utils";

/** A field line of `section`, written with `operation`. */
function field(section: string | null, operation: string, value: string | null): LtxDeclaration {
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

describe("toStatementText", () => {
  it("spells a field line under its header with both prefixes, as the file writes it", () => {
    expect(toStatementText(field("wpn_ak74", ">", "ammo_c"))).toBe("![wpn_ak74] >ammo_class = ammo_c");
  });

  it("spells a bare key without an equals sign, since it wrote no value", () => {
    expect(toStatementText(field("wpn_ak74", "!", null))).toBe("![wpn_ak74] !ammo_class");
  });

  it("spells a key before the first header as the bare line it is", () => {
    expect(toStatementText(field(null, "", "ammo_c"))).toBe("ammo_class = ammo_c");
  });
});
