import { describe, expect, it } from "@jest/globals";

import { assertExhaustive } from "@/lib/types/exhaustive";

describe("assertExhaustive", () => {
  it("names what arrived where nothing should have", () => {
    // Only reachable from a value the declarations do not hold, which is why it throws rather than returns.
    expect(() => assertExhaustive("shaders" as never)).toThrow('Unhandled case: "shaders"');
  });
});
