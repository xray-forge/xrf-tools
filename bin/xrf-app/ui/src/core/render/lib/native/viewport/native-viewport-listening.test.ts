import { describe, expect, it, jest } from "@jest/globals";

import { NativeViewport } from "@/core/render/lib/native/viewport/native-viewport";
import { followListening } from "@/core/render/lib/native/viewport/native-viewport-listening";

/** A viewport counting how often it was told to listen. */
function mockViewport(): NativeViewport & { listen: jest.Mock } {
  return { listen: jest.fn() } as unknown as NativeViewport & { listen: jest.Mock };
}

describe("followListening", () => {
  it("hears the first pane attached, then the one pressed in, again as the window takes the focus", () => {
    const first = mockViewport();
    const second = mockViewport();
    const firstElement: HTMLElement = document.createElement("div");
    const secondElement: HTMLElement = document.createElement("div");

    const stopFirst = followListening(first, firstElement);
    const stopSecond = followListening(second, secondElement);

    expect(first.listen).toHaveBeenCalledTimes(1);
    expect(second.listen).not.toHaveBeenCalled();

    secondElement.dispatchEvent(new Event("pointerdown"));
    window.dispatchEvent(new Event("focus"));

    expect(second.listen).toHaveBeenCalledTimes(2);
    expect(first.listen).toHaveBeenCalledTimes(1);

    stopSecond();
    window.dispatchEvent(new Event("focus"));

    expect(second.listen).toHaveBeenCalledTimes(2);

    stopFirst();
  });
});
