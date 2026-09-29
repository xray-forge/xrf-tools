import { describe, expect, it } from "@jest/globals";

import { isSameDefinition } from "#/scene/same-definition";

describe("isSameDefinition", () => {
  it("takes copies of the same definition as the same, however deep", () => {
    const definition = { drop: { indices: [0, 1, 2], positions: new Float32Array([1, 2, 3]) }, streak: "fx\\fx_rain" };

    expect(isSameDefinition(definition, structuredClone(definition))).toBe(true);
    expect(isSameDefinition(null, null)).toBe(true);
  });

  it("tells apart a value, a length, a key or a kind of array changed anywhere", () => {
    const definition = { drop: { indices: [0, 1, 2] }, streak: "fx\\fx_rain" };

    expect(isSameDefinition(definition, { ...definition, streak: "fx\\fx_snow" })).toBe(false);
    expect(isSameDefinition(definition, { ...definition, drop: { indices: [0, 1] } })).toBe(false);
    expect(isSameDefinition(definition, { ...definition, drop: null })).toBe(false);
    expect(isSameDefinition(definition, { drop: definition.drop })).toBe(false);
    expect(isSameDefinition([1, 2], new Float32Array([1, 2]))).toBe(false);
    expect(isSameDefinition(definition, null)).toBe(false);
  });
});
