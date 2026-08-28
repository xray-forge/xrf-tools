import { describe, expect, it, jest } from "@jest/globals";
import { MouseEvent } from "react";

import { withStoppedPropagation } from "@/lib/dom/event/propagation";

describe("withStoppedPropagation", () => {
  function mockEvent(): MouseEvent<HTMLElement> {
    return { stopPropagation: jest.fn() } as unknown as MouseEvent<HTMLElement>;
  }

  it("runs the action and stops the event", () => {
    const action = jest.fn();
    const event: MouseEvent<HTMLElement> = mockEvent();

    withStoppedPropagation(action)(event);

    expect(event.stopPropagation).toHaveBeenCalled();
    expect(action).toHaveBeenCalled();
  });

  it("stops the event before the action runs, so an action that throws still contains the click", () => {
    const event: MouseEvent<HTMLElement> = mockEvent();

    expect(() =>
      withStoppedPropagation(() => {
        throw new Error("action failed");
      })(event)
    ).toThrow("action failed");

    expect(event.stopPropagation).toHaveBeenCalled();
  });
});
