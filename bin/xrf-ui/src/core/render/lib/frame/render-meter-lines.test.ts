import { describe, expect, it } from "@jest/globals";
import { ABSENT_VALUE } from "@xrf/format";
import { RenderMeterReading } from "@xrf/ipc/types/xrf-renderer";

import { formatRenderLuminance, toRenderMeterLines } from "@/core/render/lib/frame/render-meter-lines";

const UNREAD: RenderMeterReading = {
  albedo: null,
  diffuse: null,
  distance: null,
  ev100: null,
  exposure: null,
  gloss: null,
  luminance: null,
  specular: null,
  stops: null,
};

describe("render meter lines", () => {
  it("reads the light at the spot, a surface's under it where one lies there", () => {
    expect(
      toRenderMeterLines({
        ...UNREAD,
        albedo: 0.18,
        distance: 12.34,
        exposure: 1.5,
        gloss: 0.5,
        luminance: 0.4321,
        stops: -0.5,
      })
    ).toEqual(["Meter -0.50 EV · L 0.432 · ×1.50", "albedo 0.180 · gloss 0.50 · 12.3 m"]);
    expect(toRenderMeterLines({ ...UNREAD, ev100: 14.96, luminance: 12345, stops: 0.25 })).toEqual([
      "Meter +0.25 EV · L 12345 cd/m² · EV100 15.0",
    ]);
  });

  it("shows what the meter did not read as absent, never as a reading of nothing", () => {
    expect(toRenderMeterLines(UNREAD)).toEqual([`Meter ${ABSENT_VALUE} EV · L ${ABSENT_VALUE} · ×${ABSENT_VALUE}`]);
    expect(formatRenderLuminance(Number.NaN)).toBe(ABSENT_VALUE);
  });
});
