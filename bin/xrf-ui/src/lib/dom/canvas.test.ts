import { afterEach, describe, expect, it } from "@jest/globals";

import { canRenderOffscreen } from "@/lib/dom/canvas";

function mockTransferSupport(isSupported: boolean): void {
  if (isSupported) {
    (HTMLCanvasElement.prototype as Partial<HTMLCanvasElement>).transferControlToOffscreen = () =>
      ({}) as OffscreenCanvas;
  } else {
    delete (HTMLCanvasElement.prototype as Partial<HTMLCanvasElement>).transferControlToOffscreen;
  }
}

describe("canRenderOffscreen", () => {
  afterEach(() => mockTransferSupport(false));

  // Whichever answer this gives decides which thread a viewport draws on, so both answers are worth pinning.
  it("answers for the canvas the document would make", () => {
    mockTransferSupport(true);

    expect(canRenderOffscreen()).toBe(true);
  });

  it("refuses a browser that cannot hand a canvas away", () => {
    mockTransferSupport(false);

    expect(canRenderOffscreen()).toBe(false);
  });
});
