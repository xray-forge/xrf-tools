import { describe, expect, it } from "@jest/globals";

import { wrap } from "#/wrap";

describe("wrap", () => {
  it("keeps a value already within the round", () => {
    expect(wrap(5, 24)).toBe(5);
  });

  it("counts a value past the round, or before it, from zero again", () => {
    expect(wrap(25, 24)).toBe(1);
    expect(wrap(-1, 24)).toBe(23);
    expect(wrap(-49, 24)).toBe(23);
  });

  it("reads a whole number of rounds as zero", () => {
    expect(wrap(48, 24)).toBe(0);
    expect(wrap(-24, 24)).toBe(0);
  });
});
