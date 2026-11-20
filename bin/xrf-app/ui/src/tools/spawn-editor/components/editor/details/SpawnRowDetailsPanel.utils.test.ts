import { describe, expect, it } from "@jest/globals";

import { formatSpawnRowDetailsValue } from "./SpawnRowDetailsPanel.utils";

describe("formatSpawnRowDetailsValue", () => {
  it.each([null, undefined])("shows a dash for a missing value (%s)", (value) => {
    expect(formatSpawnRowDetailsValue(value)).toBe("-");
  });

  it.each([
    [0, "0"],
    [65535, "65535"],
    [false, "false"],
    ["", ""],
    ["esc_smart_stalker_1", "esc_smart_stalker_1"],
  ])("shows a plain value as it is (%p)", (value, expected) => {
    expect(formatSpawnRowDetailsValue(value)).toBe(expected);
  });

  it("says an empty list is empty", () => {
    expect(formatSpawnRowDetailsValue([])).toBe("empty");
  });

  it("joins a list's items, a missing one included", () => {
    expect(formatSpawnRowDetailsValue([1, null, "a"])).toBe("1, -, a");
  });

  it("names each field of an object", () => {
    expect(formatSpawnRowDetailsValue({ x: 12.5, y: 1.25, z: -30 })).toBe("x: 12.5, y: 1.25, z: -30");
  });

  it("brackets a nested list or object, so its members never read as the outer value's", () => {
    expect(
      formatSpawnRowDetailsValue({
        links: [
          [1, 1],
          [2, 1],
        ],
        position: { x: 1, y: 2 },
        id: 3,
      })
    ).toBe("links: [[1, 1], [2, 1]], position: { x: 1, y: 2 }, id: 3");
  });

  it("names an empty field rather than dropping it", () => {
    expect(formatSpawnRowDetailsValue({ clientData: [], parentId: null })).toBe("clientData: empty, parentId: -");
  });
});
