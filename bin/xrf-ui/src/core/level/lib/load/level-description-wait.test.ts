import { describe, expect, it } from "@jest/globals";
import { AsyncState } from "@xrf/async";

import { describeLevelDescriptionWait } from "./level-description-wait";

describe("describeLevelDescriptionWait", () => {
  it("says nothing once the renderer has described the level", () => {
    expect(describeLevelDescriptionWait(AsyncState.ready<void>(null), "what it could not draw")).toBeNull();
  });

  it("says the description is not read yet, being read, or could not be read", () => {
    expect(describeLevelDescriptionWait(AsyncState.idle(), "what it could not draw")).toBe(
      "Not read yet: the renderer says what it could not draw once the level is resident."
    );
    expect(describeLevelDescriptionWait(AsyncState.loading(), "what it could not draw")).toBe(
      "Reading what it could not draw…"
    );
    expect(
      describeLevelDescriptionWait(AsyncState.failed(new Error("The viewport is gone")), "what it could not draw")
    ).toBe("Could not read what it could not draw: The viewport is gone.");
  });
});
