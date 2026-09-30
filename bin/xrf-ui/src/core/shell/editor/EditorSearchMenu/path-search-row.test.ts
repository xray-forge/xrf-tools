import { describe, expect, it } from "@jest/globals";

import { toPathSearchRow } from "./path-search-row";

describe("toPathSearchRow", () => {
  it("names a path by its last part, with its directory beneath", () => {
    expect(toPathSearchRow("meshes\\dynamics\\wpn_ak74.ogf")).toEqual({
      description: "meshes\\dynamics",
      id: "meshes\\dynamics\\wpn_ak74.ogf",
      label: "wpn_ak74.ogf",
    });
  });

  it("gives a path at the root no directory", () => {
    expect(toPathSearchRow("system.ltx")).toEqual({ description: undefined, id: "system.ltx", label: "system.ltx" });
  });
});
