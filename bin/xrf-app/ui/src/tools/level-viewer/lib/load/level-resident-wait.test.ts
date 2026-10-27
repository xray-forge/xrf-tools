import { describe, expect, it } from "@jest/globals";
import { AsyncState } from "@xrf/async";

import { EMPTY_LEVEL_RESIDENT } from "@/core/level/lib/render/level-resident";

import { describeLevelResidentWait } from "./level-resident-wait";

describe("describeLevelResidentWait", () => {
  it("says nothing once the renderer has described the level", () => {
    expect(describeLevelResidentWait(AsyncState.ready(EMPTY_LEVEL_RESIDENT), "what it could not draw")).toBeNull();
  });

  it("says the description is not read yet, being read, or could not be read", () => {
    expect(describeLevelResidentWait(AsyncState.idle(), "what it could not draw")).toBe(
      "Not read yet: the renderer says what it could not draw once the level is resident."
    );
    expect(describeLevelResidentWait(AsyncState.loading(), "what it could not draw")).toBe(
      "Reading what it could not draw…"
    );
    expect(
      describeLevelResidentWait(AsyncState.failed(new Error("The viewport is gone")), "what it could not draw")
    ).toBe("Could not read what it could not draw: The viewport is gone.");
  });
});
