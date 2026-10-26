import { describe, expect, it } from "@jest/globals";

import { clampPanelWidth } from "@/core/shell/panel/panel-width";

describe("clampPanelWidth", () => {
  it("leaves a width the window has room for alone", () => {
    expect(clampPanelWidth(300, 1600, 1)).toBe(300);
  });

  it("keeps the pixel maximum when half the window is wider than it", () => {
    expect(clampPanelWidth(900, 1600, 1)).toBe(640);
  });

  it("caps a lone panel at half the window when that is the smaller bound", () => {
    // The narrowest supported window: the ratio binds here, which the fixed maximum alone never did.
    expect(clampPanelWidth(640, 900, 1)).toBe(450);
  });

  it("splits the budget so two open panels cannot take the whole window", () => {
    expect(clampPanelWidth(640, 900, 2)).toBe(225);
    expect(clampPanelWidth(640, 1280, 2)).toBe(320);
  });

  it("holds the floor when the budget share falls below a usable panel", () => {
    // A window this narrow cannot lay out either way; unusable chrome is the worse of the two outcomes.
    expect(clampPanelWidth(640, 600, 2)).toBe(200);
  });

  it("treats a closed frame as one share rather than dividing by zero", () => {
    expect(clampPanelWidth(640, 1600, 0)).toBe(640);
  });
});
