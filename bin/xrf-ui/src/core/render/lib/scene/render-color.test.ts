import { describe, expect, it } from "@jest/globals";

import { toRawColor } from "@/core/render/lib/scene/render-color";

describe("toRawColor", () => {
  it("reads each byte of a hex colour as a raw value", () => {
    expect(toRawColor(0xff8000)).toEqual([1, 128 / 255, 0]);
  });
});
