import { describe, expect, it } from "@jest/globals";

import { filterByName } from "@/lib/search/filter-by-name";

describe("filterByName", () => {
  const names: Array<string> = ["wpn_ak74_idle", "wpn_ak74_reload", "wpn_pm_idle"];

  it("keeps the entries whose name carries the query, with case and surrounding space ignored", () => {
    expect(filterByName(names, "  AK74 ", (it) => it)).toEqual(["wpn_ak74_idle", "wpn_ak74_reload"]);
  });

  it("answers the same list for an empty query, so a memoized list stays the same", () => {
    expect(filterByName(names, " ", (it) => it)).toBe(names);
  });
});
