import { describe, expect, it } from "@jest/globals";

import { flattenDottedRecord, unflattenDottedRecord } from "@/lib/object/dotted-record";

describe("dotted record", () => {
  const nested: Record<string, unknown> = {
    shadows: { cascades: [20, 40], contact: { steps: 16 } },
    tonemap: "enhanced",
  };
  const flat: Record<string, unknown> = {
    "shadows.cascades": [20, 40],
    "shadows.contact.steps": 16,
    tonemap: "enhanced",
  };

  it("puts every value at its dotted key, a run of values whole, and nests them back", () => {
    expect(flattenDottedRecord(nested)).toEqual(flat);
    expect(flattenDottedRecord(nested, "settings")).toEqual(
      Object.fromEntries(Object.entries(flat).map(([key, value]: [string, unknown]) => [`settings.${key}`, value]))
    );
    expect(unflattenDottedRecord(flat)).toEqual(nested);
  });

  it("keeps a nothing as nothing and a single value at its prefix", () => {
    expect(flattenDottedRecord(undefined, "a")).toEqual({});
    expect(flattenDottedRecord(null, "a")).toEqual({ a: null });
    expect(flattenDottedRecord({})).toEqual({});
  });
});
